//! Direct3D 11 for the hardware decoders. A decoder that Media Foundation drives through a DXGI device
//! manager gives its frames as GPU textures, which the CPU cannot read. This file makes the device the
//! decoder draws on, and copies each frame into a staging texture the CPU can map.

use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_FLAG, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11CreateDevice, ID3D11Device,
    ID3D11DeviceContext, ID3D11Multithread, ID3D11Resource, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;
use windows::Win32::Media::MediaFoundation::{IMFDXGIDeviceManager, MFCreateDXGIDeviceManager};
use windows::core::Interface;

use super::DecodeError;

const BACKEND: &str = "Media Foundation";

/// The staging texture, kept between frames while the picture size stays the same.
struct Staging {
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    texture: ID3D11Texture2D,
}

/// A hardware Direct3D device, and the DXGI device manager that hands it to a decoder.
pub(super) struct Device {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    manager: IMFDXGIDeviceManager,
    staging: Option<Staging>,
}

impl Device {
    /// Create the device with video support, and wrap it in a device manager.
    pub(super) fn create() -> Result<Self, DecodeError> {
        let flags = D3D11_CREATE_DEVICE_VIDEO_SUPPORT | D3D11_CREATE_DEVICE_BGRA_SUPPORT;
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        // SAFETY: the out-pointers are valid for the call. A default adapter is used when none is given.
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                flags,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device as *mut Option<ID3D11Device>),
                None,
                Some(&mut context as *mut Option<ID3D11DeviceContext>),
            )
        }
        .map_err(|error| step("D3D11CreateDevice", error))?;
        let device = device.ok_or_else(|| failed("D3D11CreateDevice gave no device"))?;
        let context = context.ok_or_else(|| failed("D3D11CreateDevice gave no context"))?;

        // The decoder and this file both use the device, so it must be safe across threads.
        let multithread: ID3D11Multithread = device
            .cast()
            .map_err(|error| step("ID3D11Multithread", error))?;
        // SAFETY: the interface is alive for the call. The result is only a flag from before the change.
        let _ = unsafe { multithread.SetMultithreadProtected(true) };

        let mut token = 0u32;
        let mut manager: Option<IMFDXGIDeviceManager> = None;
        // SAFETY: the token and the manager pointer are valid for the call.
        unsafe {
            MFCreateDXGIDeviceManager(
                &mut token as *mut u32,
                &mut manager as *mut Option<IMFDXGIDeviceManager>,
            )
        }
        .map_err(|error| step("MFCreateDXGIDeviceManager", error))?;
        let manager = manager.ok_or_else(|| failed("MFCreateDXGIDeviceManager gave no manager"))?;
        // SAFETY: the device and the manager are alive, and the token is the one just issued.
        unsafe { manager.ResetDevice(&device, token) }
            .map_err(|error| step("IMFDXGIDeviceManager::ResetDevice", error))?;

        Ok(Device {
            device,
            context,
            manager,
            staging: None,
        })
    }

    /// The manager's interface pointer, for `MFT_MESSAGE_SET_D3D_MANAGER`. It stays valid for as long
    /// as this device lives.
    pub(super) fn manager_pointer(&self) -> usize {
        self.manager.as_raw() as usize
    }

    /// Copy one subresource of `texture` into system memory. Returns the bytes, the row pitch in bytes,
    /// and the texture's height in rows. For a semi-planar format the chroma rows follow the luma rows.
    pub(super) fn read(
        &mut self,
        texture: &ID3D11Texture2D,
        subresource: u32,
    ) -> Result<(Vec<u8>, usize, usize), DecodeError> {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: the texture is alive, and the description is written in full.
        unsafe { texture.GetDesc(&mut desc) };
        let staging = self.staging_texture(&desc)?;
        let source: ID3D11Resource = texture
            .cast()
            .map_err(|error| step("ID3D11Texture2D::cast", error))?;
        let target: ID3D11Resource = staging
            .cast()
            .map_err(|error| step("ID3D11Texture2D::cast", error))?;
        // SAFETY: both resources are on this device, and the subresource is the one the decoder wrote.
        unsafe {
            self.context
                .CopySubresourceRegion(&target, 0, 0, 0, 0, &source, subresource, None)
        };

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: the staging texture is readable by the CPU, and it is unmapped below.
        unsafe {
            self.context
                .Map(&target, 0, D3D11_MAP_READ, 0, Some(&mut mapped as *mut _))
        }
        .map_err(|error| step("ID3D11DeviceContext::Map", error))?;
        let height = desc.Height as usize;
        let pitch = mapped.RowPitch as usize;
        // Luma rows, then half as many chroma rows.
        let rows = height + height.div_ceil(2);
        // SAFETY: the mapped memory holds `pitch` bytes for each row the staging texture has, and it
        // is copied before the unmap.
        let bytes =
            unsafe { std::slice::from_raw_parts(mapped.pData as *const u8, pitch * rows) }.to_vec();
        // SAFETY: the resource was mapped above.
        unsafe { self.context.Unmap(&target, 0) };
        Ok((bytes, pitch, height))
    }

    /// A staging texture for the given description, reused while the size and format stay the same.
    fn staging_texture(
        &mut self,
        source: &D3D11_TEXTURE2D_DESC,
    ) -> Result<ID3D11Texture2D, DecodeError> {
        if let Some(staging) = &self.staging
            && staging.width == source.Width
            && staging.height == source.Height
            && staging.format == source.Format
        {
            return Ok(staging.texture.clone());
        }
        let desc = D3D11_TEXTURE2D_DESC {
            Width: source.Width,
            Height: source.Height,
            MipLevels: 1,
            ArraySize: 1,
            Format: source.Format,
            SampleDesc: source.SampleDesc,
            Usage: D3D11_USAGE_STAGING,
            BindFlags: D3D11_BIND_FLAG(0).0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut texture: Option<ID3D11Texture2D> = None;
        // SAFETY: the description and the out-pointer are valid for the call.
        unsafe {
            self.device
                .CreateTexture2D(&desc, None, Some(&mut texture as *mut _))
        }
        .map_err(|error| step("ID3D11Device::CreateTexture2D", error))?;
        let texture = texture.ok_or_else(|| failed("CreateTexture2D gave no texture"))?;
        self.staging = Some(Staging {
            width: source.Width,
            height: source.Height,
            format: source.Format,
            texture: texture.clone(),
        });
        Ok(texture)
    }
}

/// A failure from one named Direct3D call, with its HRESULT.
fn step(name: &str, error: windows::core::Error) -> DecodeError {
    DecodeError::Failed {
        backend: BACKEND,
        reason: format!("{name} returned {:#010X}", error.code().0 as u32),
    }
}

fn failed(reason: &str) -> DecodeError {
    DecodeError::Failed {
        backend: BACKEND,
        reason: reason.into(),
    }
}
