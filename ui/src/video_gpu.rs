//! Draws video frames on the GPU. A frame's Y'CbCr planes are uploaded as they are, and a shader
//! converts them to RGB in a texture the size of the video. The window then draws that texture
//! scaled to its place, bilinearly filtered as before. The CPU no longer computes every pixel, and
//! the upload is 1.5 bytes a pixel rather than 4.

use std::collections::VecDeque;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use eframe::egui;
use eframe::egui_wgpu::{self, wgpu};
use quokka_playback::decode::frames::{Layout, Yuv, coefficients};

#[cfg(target_os = "macos")]
use crate::metal_video;

const CONVERT: &str = include_str!("video_convert.wgsl");
const DRAW: &str = include_str!("video_draw.wgsl");

/// The conversion parameters: sixteen 32-bit words.
const PARAMS_BYTES: u64 = 64;

/// Planes of earlier frames kept alive after they are replaced. The GPU may still be reading them
/// when the next frame arrives, and a hardware buffer must not be reused until it has finished.
const RETIRED_PLANES: usize = 4;

/// The GPU state for video: the two pipelines, and the textures for the frame on screen.
pub struct VideoGpu {
    convert: wgpu::RenderPipeline,
    draw: wgpu::RenderPipeline,
    convert_layout: wgpu::BindGroupLayout,
    draw_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params: wgpu::Buffer,
    planes: Option<Planes>,
    target: Option<Target>,
    /// The frame count and deinterlace setting last converted, so an unchanged frame is not converted
    /// again. The planes are uploaded once per frame, whatever the setting.
    uploaded: Option<(u64, bool)>,
    /// Hardware frames are read in place when this is set; `None` on devices that cannot.
    #[cfg(target_os = "macos")]
    importer: Option<metal_video::Importer>,
    #[cfg(target_os = "macos")]
    retired: VecDeque<Planes>,
}

/// The sample planes of a frame, as textures, and the bind group that reads them.
struct Planes {
    width: u32,
    height: u32,
    layout: Layout,
    /// The hardware buffer the planes read, when the frame was not copied into them.
    #[cfg(target_os = "macos")]
    imported: Option<metal_video::Imported>,
    y: wgpu::Texture,
    /// Cb, or the interleaved CbCr plane.
    chroma_a: wgpu::Texture,
    /// Cr, or a one-texel stand-in when Cb and Cr are interleaved.
    chroma_b: wgpu::Texture,
    bind: wgpu::BindGroup,
}

impl Planes {
    /// Whether the planes read a hardware buffer in place, rather than textures uploaded to.
    fn is_imported(&self) -> bool {
        #[cfg(target_os = "macos")]
        return self.imported.is_some();
        #[cfg(not(target_os = "macos"))]
        return false;
    }
}

/// The converted frame: RGBA at the size of the video, and the bind group that draws it.
struct Target {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    draw: wgpu::BindGroup,
}

impl VideoGpu {
    /// The pipelines for drawing into a surface of `target_format`.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let entry = |binding: u32, ty: wgpu::BindingType| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        };
        let samples = wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        };
        let convert_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("video convert"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(1, samples),
                entry(2, samples),
                entry(3, samples),
            ],
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("video draw"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
            ],
        });

        let convert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("video convert"),
            source: wgpu::ShaderSource::Wgsl(CONVERT.into()),
        });
        let draw_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("video draw"),
            source: wgpu::ShaderSource::Wgsl(DRAW.into()),
        });

        let convert_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("video convert"),
                bind_group_layouts: &[Some(&convert_layout)],
                immediate_size: 0,
            });
        let convert = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("video convert"),
            layout: Some(&convert_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &convert_shader,
                entry_point: Some("vs_full"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &convert_shader,
                entry_point: Some("fs_convert"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let draw_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("video draw"),
            bind_group_layouts: &[Some(&draw_layout)],
            immediate_size: 0,
        });
        // egui writes to an sRGB surface as linear light and expects gamma values, so the shader
        // converts to linear when the surface is sRGB.
        let srgb = if target_format.is_srgb() { 1.0 } else { 0.0 };
        let draw = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("video draw"),
            layout: Some(&draw_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &draw_shader,
                entry_point: Some("vs_draw"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &draw_shader,
                entry_point: Some("fs_draw"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[("srgb_target", srgb)],
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("video"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("video parameters"),
            size: PARAMS_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            convert,
            draw,
            convert_layout,
            draw_layout,
            sampler,
            params,
            planes: None,
            target: None,
            uploaded: None,
            #[cfg(target_os = "macos")]
            importer: metal_video::Importer::new(device),
            #[cfg(target_os = "macos")]
            retired: VecDeque::new(),
        }
    }

    /// Upload `frame` and convert it to the target, unless `shown` and `deinterlace` are the ones
    /// already converted. A change of `deinterlace` alone converts the planes on the GPU again
    /// without uploading them. The conversion is recorded into `encoder`, which runs before the
    /// window is drawn.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        frame: &Yuv,
        shown: u64,
        deinterlace: bool,
    ) {
        if self.uploaded == Some((shown, deinterlace)) {
            return;
        }
        let new_frame = self.uploaded.is_none_or(|(uploaded, _)| uploaded != shown);
        #[cfg(target_os = "macos")]
        let zero_copy = new_frame && self.import_frame(device, frame);
        #[cfg(not(target_os = "macos"))]
        let zero_copy = false;
        let samples = if !new_frame || zero_copy {
            None
        } else {
            let samples = frame.sample_bytes();
            if samples.len() < frame.layout.total_bytes(frame.width, frame.height) {
                return;
            }
            Some(samples)
        };
        self.uploaded = Some((shown, deinterlace));
        if samples.is_some() {
            self.ensure_planes(device, frame);
        }
        self.ensure_target(device, frame.width, frame.height);
        let (Some(planes), Some(target)) = (self.planes.as_ref(), self.target.as_ref()) else {
            return;
        };

        if let Some(samples) = &samples {
            upload(queue, planes, frame, samples);
        }
        queue.write_buffer(&self.params, 0, &params_bytes(frame, deinterlace));

        let view = target
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("video convert"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.convert);
        pass.set_bind_group(0, &planes.bind, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Draw the converted frame into a render pass. `viewport` is where the video goes, as
    /// `(x, y, width, height)` in pixels, and `scissor` clips it, in the same form. Nothing is drawn
    /// before the first frame is converted.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, viewport: [f32; 4], scissor: [u32; 4]) {
        let Some(target) = self.target.as_ref() else {
            return;
        };
        pass.set_pipeline(&self.draw);
        pass.set_bind_group(0, &target.draw, &[]);
        pass.set_viewport(viewport[0], viewport[1], viewport[2], viewport[3], 0.0, 1.0);
        pass.set_scissor_rect(scissor[0], scissor[1], scissor[2], scissor[3]);
        pass.draw(0..3, 0..1);
    }

    /// The RGBA texture holding the converted frame, once one has been converted.
    pub fn rgba_texture(&self) -> Option<&wgpu::Texture> {
        self.target.as_ref().map(|target| &target.texture)
    }

    /// Read a hardware frame's planes in place. Returns `false` when the frame has no hardware
    /// buffer or the device cannot import it, and the frame then takes the upload path.
    #[cfg(target_os = "macos")]
    fn import_frame(&mut self, device: &wgpu::Device, frame: &Yuv) -> bool {
        let (Some(importer), Some(buffer)) = (self.importer.as_mut(), frame.platform.as_ref())
        else {
            return false;
        };
        let Some(imported) =
            importer.import(device, buffer, frame.layout, frame.width, frame.height)
        else {
            return false;
        };
        let bytes = frame.layout.bytes();
        let stand_in = texture(
            device,
            1,
            1,
            sample_format(bytes, 1),
            wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let luma = imported.luma.clone();
        let chroma = imported.chroma.clone();
        let bind = self.bind_group(device, &luma, &chroma, &stand_in);
        let planes = Planes {
            width: frame.width,
            height: frame.height,
            layout: frame.layout,
            imported: Some(imported),
            y: luma,
            chroma_a: chroma,
            chroma_b: stand_in,
            bind,
        };
        self.replace_planes(planes);
        true
    }

    /// Make `planes` the planes on screen. The ones they replace are kept a few frames longer, so
    /// the GPU has finished reading them before their hardware buffers are released.
    fn replace_planes(&mut self, planes: Planes) {
        if let Some(old) = self.planes.replace(planes) {
            #[cfg(target_os = "macos")]
            {
                self.retired.push_back(old);
                while self.retired.len() > RETIRED_PLANES {
                    self.retired.pop_front();
                }
            }
            #[cfg(not(target_os = "macos"))]
            drop(old);
        }
    }

    fn bind_group(
        &self,
        device: &wgpu::Device,
        y: &wgpu::Texture,
        a: &wgpu::Texture,
        b: &wgpu::Texture,
    ) -> wgpu::BindGroup {
        let y_view = y.create_view(&wgpu::TextureViewDescriptor::default());
        let a_view = a.create_view(&wgpu::TextureViewDescriptor::default());
        let b_view = b.create_view(&wgpu::TextureViewDescriptor::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("video planes"),
            layout: &self.convert_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&y_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&b_view),
                },
            ],
        })
    }

    fn ensure_planes(&mut self, device: &wgpu::Device, frame: &Yuv) {
        let fits = self.planes.as_ref().is_some_and(|planes| {
            planes.width == frame.width
                && planes.height == frame.height
                && planes.layout == frame.layout
                && !planes.is_imported()
        });
        if fits {
            return;
        }
        let layout = frame.layout;
        let (cw, ch) = layout.chroma_size(frame.width as usize, frame.height as usize);
        let (cw, ch) = (cw as u32, ch as u32);
        let bytes = layout.bytes();
        let y = texture(
            device,
            frame.width,
            frame.height,
            sample_format(bytes, 1),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let chroma_a = texture(
            device,
            cw,
            ch,
            sample_format(bytes, if layout.biplanar { 2 } else { 1 }),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let (b_width, b_height) = if layout.biplanar { (1, 1) } else { (cw, ch) };
        let chroma_b = texture(
            device,
            b_width,
            b_height,
            sample_format(bytes, 1),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let y_view = y.create_view(&wgpu::TextureViewDescriptor::default());
        let a_view = chroma_a.create_view(&wgpu::TextureViewDescriptor::default());
        let b_view = chroma_b.create_view(&wgpu::TextureViewDescriptor::default());
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("video planes"),
            layout: &self.convert_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&y_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&b_view),
                },
            ],
        });
        let planes = Planes {
            width: frame.width,
            height: frame.height,
            layout,
            #[cfg(target_os = "macos")]
            imported: None,
            y,
            chroma_a,
            chroma_b,
            bind,
        };
        self.replace_planes(planes);
    }

    fn ensure_target(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self
            .target
            .as_ref()
            .is_some_and(|target| target.width == width && target.height == height)
        {
            return;
        }
        let texture = texture(
            device,
            width,
            height,
            wgpu::TextureFormat::Rgba8Unorm,
            // COPY_SRC lets tests read the converted pixels back.
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let draw = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("video frame"),
            layout: &self.draw_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.target = Some(Target {
            width,
            height,
            texture,
            draw,
        });
    }
}

/// A frame to draw in the window. The conversion runs when the frame changes, and the window then
/// draws the result.
pub struct VideoPaint {
    pub frame: Arc<Yuv>,
    /// Which frame this is, so an unchanged frame is not uploaded again.
    pub shown: u64,
    /// Whether odd rows are blended with their neighbours, for an interlaced picture.
    pub deinterlace: bool,
}

impl egui_wgpu::CallbackTrait for VideoPaint {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(gpu) = resources.get_mut::<VideoGpu>() {
            gpu.prepare(
                device,
                queue,
                encoder,
                &self.frame,
                self.shown,
                self.deinterlace,
            );
        }
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        if let Some(gpu) = resources.get::<VideoGpu>() {
            let viewport = info.viewport_in_pixels();
            let clip = info.clip_rect_in_pixels();
            gpu.draw(
                pass,
                [
                    viewport.left_px as f32,
                    viewport.top_px as f32,
                    viewport.width_px as f32,
                    viewport.height_px as f32,
                ],
                [
                    clip.left_px.max(0) as u32,
                    clip.top_px.max(0) as u32,
                    clip.width_px.max(0) as u32,
                    clip.height_px.max(0) as u32,
                ],
            );
        }
    }
}

/// Add the video pipelines to the window's renderer. Call once, when the window is created.
pub fn install(render_state: &egui_wgpu::RenderState) {
    let gpu = VideoGpu::new(&render_state.device, render_state.target_format);
    render_state.renderer.write().callback_resources.insert(gpu);
}

/// A device with no window, for tests and benchmarks. `None` when the machine has no GPU.
pub fn offscreen_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter =
        block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
    block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

/// Wait for a future that does not need the async runtime. The native GPU calls are synchronous,
/// so these futures finish when first polled.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        std::thread::yield_now();
    }
}

/// The texture format for a sample of `bytes` bytes, with one or two channels.
fn sample_format(bytes: usize, channels: u32) -> wgpu::TextureFormat {
    match (bytes, channels) {
        (1, 1) => wgpu::TextureFormat::R8Uint,
        (1, _) => wgpu::TextureFormat::Rg8Uint,
        (_, 1) => wgpu::TextureFormat::R16Uint,
        _ => wgpu::TextureFormat::Rg16Uint,
    }
}

fn texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("video"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

/// Upload the frame's planes. Each plane is one texture, with its rows as they lie in memory.
/// Copy a frame's samples into `planes`. `samples` holds the frame's planes one after another.
fn upload(queue: &wgpu::Queue, planes: &Planes, frame: &Yuv, samples: &[u8]) {
    let layout = frame.layout;
    let (w, h) = (frame.width as usize, frame.height as usize);
    let (cw, ch) = layout.chroma_size(w, h);
    let bytes = layout.bytes();
    let luma_len = w * h * bytes;
    let chroma_len = cw * ch * bytes;
    write_plane(
        queue,
        &planes.y,
        &samples[..luma_len],
        frame.width,
        frame.height,
        w * bytes,
    );
    if layout.biplanar {
        write_plane(
            queue,
            &planes.chroma_a,
            &samples[luma_len..luma_len + 2 * chroma_len],
            cw as u32,
            ch as u32,
            cw * 2 * bytes,
        );
    } else {
        write_plane(
            queue,
            &planes.chroma_a,
            &samples[luma_len..luma_len + chroma_len],
            cw as u32,
            ch as u32,
            cw * bytes,
        );
        write_plane(
            queue,
            &planes.chroma_b,
            &samples[luma_len + chroma_len..luma_len + 2 * chroma_len],
            cw as u32,
            ch as u32,
            cw * bytes,
        );
    }
}

fn write_plane(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    data: &[u8],
    width: u32,
    height: u32,
    bytes_per_row: usize,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row as u32),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

/// The conversion parameters for a frame, as the shader declares them: sixteen 32-bit words.
fn params_bytes(frame: &Yuv, deinterlace: bool) -> [u8; PARAMS_BYTES as usize] {
    let c = coefficients(frame.matrix, frame.full_range);
    let words: [u32; 16] = [
        frame.width,
        frame.height,
        frame.layout.shift_x,
        frame.layout.shift_y,
        frame.layout.sample_shift(),
        u32::from(frame.layout.biplanar),
        c.luma_offset.to_bits(),
        c.luma_scale.to_bits(),
        c.chroma_scale.to_bits(),
        c.r_from_cr.to_bits(),
        c.g_from_cb.to_bits(),
        c.g_from_cr.to_bits(),
        c.b_from_cb.to_bits(),
        u32::from(deinterlace),
        0,
        0,
    ];
    let mut bytes = [0u8; PARAMS_BYTES as usize];
    for (i, word) in words.iter().enumerate() {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}
