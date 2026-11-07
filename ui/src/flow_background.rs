//! The animated background of a movie page, drawn on the GPU. The shader takes the time each frame,
//! so the strands drift without any work on the CPU.

use eframe::egui;
use eframe::egui_wgpu::{self, wgpu};

const SHADER: &str = include_str!("flow.wgsl");

/// The shader's parameters: four vectors of four 32-bit floats.
const PARAMS_BYTES: u64 = 64;

/// The GPU state for the background: its pipeline, and the buffer its parameters are written to.
pub struct FlowGpu {
    pipeline: wgpu::RenderPipeline,
    bind: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    /// Whether the surface is sRGB, which the shader must then write in linear light.
    srgb: bool,
}

/// The background to draw in one window area.
pub struct FlowBackground {
    /// The cover's dominant colour. Its lighter and deeper tints are the strands.
    pub base: egui::Color32,
    /// A per-movie offset of the noise, so each movie has its own pattern.
    pub seed: f32,
    /// Seconds since the app started.
    pub time: f32,
    /// The area's width over its height, so the strands are not stretched.
    pub aspect: f32,
}

impl FlowBackground {
    fn params(&self, srgb: bool) -> [u8; PARAMS_BYTES as usize] {
        let base = [
            f32::from(self.base.r()) / 255.0,
            f32::from(self.base.g()) / 255.0,
            f32::from(self.base.b()) / 255.0,
        ];
        let light: Vec<f32> = base.iter().map(|c| c + (1.0 - c) * 0.45).collect();
        let deep: Vec<f32> = base.iter().map(|c| c * 0.88).collect();
        let words = [
            base[0],
            base[1],
            base[2],
            1.0,
            light[0],
            light[1],
            light[2],
            1.0,
            deep[0],
            deep[1],
            deep[2],
            1.0,
            self.time,
            self.seed,
            self.aspect,
            if srgb { 1.0 } else { 0.0 },
        ];
        let mut bytes = [0_u8; PARAMS_BYTES as usize];
        for (index, word) in words.iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
}

impl egui_wgpu::CallbackTrait for FlowBackground {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(gpu) = resources.get::<FlowGpu>() {
            queue.write_buffer(&gpu.buffer, 0, &self.params(gpu.srgb));
        }
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let Some(gpu) = resources.get::<FlowGpu>() else {
            return;
        };
        let viewport = info.viewport_in_pixels();
        let clip = info.clip_rect_in_pixels();
        pass.set_pipeline(&gpu.pipeline);
        pass.set_bind_group(0, &gpu.bind, &[]);
        pass.set_viewport(
            viewport.left_px as f32,
            viewport.top_px as f32,
            viewport.width_px as f32,
            viewport.height_px as f32,
            0.0,
            1.0,
        );
        pass.set_scissor_rect(
            clip.left_px.max(0) as u32,
            clip.top_px.max(0) as u32,
            clip.width_px.max(0) as u32,
            clip.height_px.max(0) as u32,
        );
        pass.draw(0..3, 0..1);
    }
}

/// Add the background pipeline to the window's renderer. Call once, when the window is created.
pub fn install(render_state: &egui_wgpu::RenderState) {
    let device = &render_state.device;
    let format = render_state.target_format;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("flow background"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("flow background"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("flow background"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("flow background"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_flow"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_flow"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flow background parameters"),
        size: PARAMS_BYTES,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("flow background"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(FlowGpu {
            pipeline,
            bind,
            buffer,
            srgb: format.is_srgb(),
        });
}
