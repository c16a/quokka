//! Hands a hardware-decoded frame's planes to the GPU in place. VideoToolbox decodes into Core
//! Video buffers that Metal can read, so each plane becomes a wgpu texture over the same memory and
//! the frame is never copied through the CPU. macOS only.

use std::ffi::c_void;
use std::ptr;

use eframe::egui_wgpu::wgpu;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLDevice, MTLTexture, MTLTextureType};
use quokka_playback::decode::frames::Layout;
use quokka_playback::decode::videotoolbox::PixelBuffer;

type CVMetalTextureCacheRef = *mut c_void;
type CVMetalTextureRef = *mut c_void;

#[link(name = "CoreVideo", kind = "framework")]
unsafe extern "C" {
    fn CVMetalTextureCacheCreate(
        allocator: *const c_void,
        cache_attributes: *const c_void,
        metal_device: *const c_void,
        texture_attributes: *const c_void,
        cache_out: *mut CVMetalTextureCacheRef,
    ) -> i32;
    fn CVMetalTextureCacheCreateTextureFromImage(
        allocator: *const c_void,
        cache: CVMetalTextureCacheRef,
        source_image: *const c_void,
        texture_attributes: *const c_void,
        pixel_format: u64,
        width: usize,
        height: usize,
        plane_index: usize,
        texture_out: *mut CVMetalTextureRef,
    ) -> i32;
    fn CVMetalTextureGetTexture(texture: CVMetalTextureRef) -> *mut c_void;
    fn CFRelease(object: *const c_void);
}

/// A Core Video Metal texture, which wraps one plane. Released when dropped.
struct CvTexture(CVMetalTextureRef);

// SAFETY: the texture is only read, and releasing a Core Foundation object is allowed from any
// thread.
unsafe impl Send for CvTexture {}
unsafe impl Sync for CvTexture {}

impl Drop for CvTexture {
    fn drop(&mut self) {
        // SAFETY: we hold the one reference that `CVMetalTextureCacheCreateTextureFromImage` gave us.
        unsafe { CFRelease(self.0) }
    }
}

/// The Core Video cache that turns decoded buffers into Metal textures. Made once, for the
/// device the window draws with.
pub struct Importer {
    cache: CVMetalTextureCacheRef,
}

// SAFETY: the cache is only used from the render thread, through `&mut self`, so it is never used
// from two threads at once. Moving it between threads is allowed.
unsafe impl Send for Importer {}
unsafe impl Sync for Importer {}

impl Drop for Importer {
    fn drop(&mut self) {
        // SAFETY: the cache was created by us and is released exactly once, here.
        unsafe { CFRelease(self.cache) }
    }
}

/// The two planes of one frame, as wgpu textures over the decoder's memory. Keeps the Core Video
/// textures and the buffer alive for as long as the planes are in use.
pub struct Imported {
    pub luma: wgpu::Texture,
    pub chroma: wgpu::Texture,
    _cv: [CvTexture; 2],
    _buffer: PixelBuffer,
}

impl Importer {
    /// An importer for the Metal device under `device`. `None` when the window does not draw with
    /// Metal, in which case frames take the CPU path.
    pub fn new(device: &wgpu::Device) -> Option<Self> {
        // SAFETY: the device is not used through the HAL beyond reading its Metal device handle.
        let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }?;
        let metal: &Retained<ProtocolObject<dyn MTLDevice>> = hal.raw_device();
        let mut cache: CVMetalTextureCacheRef = ptr::null_mut();
        // SAFETY: the Metal device is alive for the call, and the output pointer is valid.
        let status = unsafe {
            CVMetalTextureCacheCreate(
                ptr::null(),
                ptr::null(),
                Retained::as_ptr(metal).cast(),
                ptr::null(),
                &mut cache,
            )
        };
        (status == 0 && !cache.is_null()).then_some(Self { cache })
    }

    /// The planes of a biplanar frame in `buffer`, as textures over its memory. `None` for planar
    /// frames, which come from software decoders, and when Core Video refuses a plane.
    pub fn import(
        &mut self,
        device: &wgpu::Device,
        buffer: &PixelBuffer,
        layout: Layout,
        width: u32,
        height: u32,
    ) -> Option<Imported> {
        if !layout.biplanar {
            return None;
        }
        let (cw, ch) = layout.chroma_size(width as usize, height as usize);
        let bytes = layout.bytes();
        let (luma_cv, luma) =
            self.plane(device, buffer, 0, width, height, plane_format(bytes, 1))?;
        let (chroma_cv, chroma) = self.plane(
            device,
            buffer,
            1,
            cw as u32,
            ch as u32,
            plane_format(bytes, 2),
        )?;
        Some(Imported {
            luma,
            chroma,
            _cv: [luma_cv, chroma_cv],
            _buffer: buffer.clone(),
        })
    }

    /// One plane of `buffer` as a wgpu texture over the same memory.
    fn plane(
        &mut self,
        device: &wgpu::Device,
        buffer: &PixelBuffer,
        index: usize,
        width: u32,
        height: u32,
        (metal_format, format): (u64, wgpu::TextureFormat),
    ) -> Option<(CvTexture, wgpu::Texture)> {
        let mut cv: CVMetalTextureRef = ptr::null_mut();
        // SAFETY: the cache and the buffer are alive, and the output pointer is valid.
        let status = unsafe {
            CVMetalTextureCacheCreateTextureFromImage(
                ptr::null(),
                self.cache,
                buffer.as_raw(),
                ptr::null(),
                metal_format,
                width as usize,
                height as usize,
                index,
                &mut cv,
            )
        };
        if status != 0 || cv.is_null() {
            return None;
        }
        let cv = CvTexture(cv);
        // SAFETY: the Core Video texture owns the Metal texture, so retaining it gives us our own
        // reference, which the wrapped texture releases when it is dropped.
        let raw = unsafe {
            Retained::retain(
                CVMetalTextureGetTexture(cv.0).cast::<ProtocolObject<dyn MTLTexture>>(),
            )
        }?;
        // SAFETY: the Metal texture has the format, size and type given here, as Core Video made it.
        let hal = unsafe {
            wgpu::hal::metal::Device::texture_from_raw(
                raw,
                format,
                MTLTextureType::Type2D,
                1,
                1,
                wgpu::hal::CopyExtent {
                    width,
                    height,
                    depth: 1,
                },
                None,
            )
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("video plane"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        // SAFETY: the texture was made on this device, from the same Metal device, and its
        // description matches what the HAL texture holds.
        let texture = unsafe {
            device.create_texture_from_hal::<wgpu::hal::api::Metal>(
                hal,
                &desc,
                wgpu::TextureUses::RESOURCE,
            )
        };
        Some((cv, texture))
    }
}

/// The Metal format of a plane with `bytes` bytes a sample and `channels` channels, and the wgpu
/// format that reads it as unsigned integers, as the conversion shader expects.
fn plane_format(bytes: usize, channels: u32) -> (u64, wgpu::TextureFormat) {
    match (bytes, channels) {
        (1, 1) => (13, wgpu::TextureFormat::R8Uint),
        (1, _) => (33, wgpu::TextureFormat::Rg8Uint),
        (_, 1) => (23, wgpu::TextureFormat::R16Uint),
        _ => (63, wgpu::TextureFormat::Rg16Uint),
    }
}
