#![cfg(target_os = "macos")]
//! The GPU conversion gives the same pixels as the CPU conversion, to within one step of rounding.
//! Each test skips, with a note, when the machine has no GPU to run it on.

use std::path::Path;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use eframe::egui_wgpu::wgpu;
use quokka_core::video::Matrix;
use quokka_playback::decode::frames::{Layout, Yuv};
use quokka_playback::decode::player::FramePlayer;
use quokka_ui::video_gpu::{VideoGpu, offscreen_device};

/// A frame of pseudo-random samples in `layout`, so every chroma block and the clamping at the
/// ends of the range are exercised.
fn synthetic(width: u32, height: u32, layout: Layout, matrix: Matrix, full_range: bool) -> Yuv {
    let len = layout.total_bytes(width, height);
    let max = (1u32 << layout.bits) - 1;
    let mut seed = 0x2545_f491 ^ width ^ (height << 8);
    let mut samples = Vec::with_capacity(len + 2);
    while samples.len() < len {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let value = (seed >> 8) % (max + 1);
        // Samples above 8 bits sit in the high bits of their 16-bit words when they are aligned high.
        let stored = if layout.high_aligned {
            value << (16 - u32::from(layout.bits))
        } else {
            value
        };
        if layout.bytes() == 2 {
            samples.extend_from_slice(&(stored as u16).to_le_bytes());
        } else {
            samples.push(stored as u8);
        }
    }
    samples.truncate(len);
    Yuv {
        width,
        height,
        layout,
        matrix,
        full_range,
        samples,
        #[cfg(target_os = "macos")]
        platform: None,
    }
}

fn layout(biplanar: bool, bits: u8, high_aligned: bool, shift_x: u32, shift_y: u32) -> Layout {
    Layout {
        biplanar,
        bits,
        high_aligned,
        shift_x,
        shift_y,
    }
}

/// Convert `frame` on the GPU and read the RGBA pixels back.
fn convert_on_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu: &mut VideoGpu,
    frame: &Yuv,
    shown: u64,
) -> Vec<u8> {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    gpu.prepare(device, queue, &mut encoder, frame, shown);
    let texture = gpu.rgba_texture().expect("the frame was converted");
    let (w, h) = (frame.width, frame.height);
    // Rows read back must be a multiple of 256 bytes wide.
    let row = (w * 4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(row) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the device waits for the copy");
    receiver
        .recv()
        .expect("the map finishes")
        .expect("the buffer maps");
    let mapped = slice
        .get_mapped_range()
        .expect("the mapped range is readable");
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for line in mapped.chunks(row as usize).take(h as usize) {
        pixels.extend_from_slice(&line[..(w * 4) as usize]);
    }
    drop(mapped);
    buffer.unmap();
    pixels
}

/// Check that the GPU conversion of `frame` matches the CPU conversion.
fn assert_matches_cpu(name: &str, frame: &Yuv) {
    let Some((device, queue)) = offscreen_device() else {
        eprintln!("no GPU here: skipped {name}");
        return;
    };
    let mut gpu = VideoGpu::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    let on_gpu = convert_on_gpu(&device, &queue, &mut gpu, frame, 1);
    let on_cpu = frame.to_rgba().pixels;
    assert_eq!(on_gpu.len(), on_cpu.len(), "{name}: size");
    let worst = on_gpu
        .iter()
        .zip(&on_cpu)
        .map(|(a, b)| (i16::from(*a) - i16::from(*b)).abs())
        .max()
        .unwrap_or(0);
    assert!(worst <= 1, "{name}: largest difference is {worst}");
}

#[test]
fn eight_bit_planar_matches_cpu() {
    let frame = synthetic(7, 5, layout(false, 8, false, 1, 1), Matrix::Bt601, false);
    assert_matches_cpu("I420 7x5", &frame);
}

#[test]
fn eight_bit_biplanar_matches_cpu() {
    let frame = synthetic(33, 17, layout(true, 8, false, 1, 1), Matrix::Bt709, false);
    assert_matches_cpu("420v 33x17", &frame);
}

#[test]
fn eight_bit_4_2_2_matches_cpu() {
    let frame = synthetic(9, 5, layout(false, 8, false, 1, 0), Matrix::Bt709, true);
    assert_matches_cpu("I422 9x5 full", &frame);
}

#[test]
fn eight_bit_4_4_4_full_range_bt2020_matches_cpu() {
    let frame = synthetic(9, 9, layout(true, 8, false, 0, 0), Matrix::Bt2020, true);
    assert_matches_cpu("444v 9x9 full bt2020", &frame);
}

#[test]
fn ten_bit_high_aligned_matches_cpu() {
    let frame = synthetic(40, 22, layout(true, 10, true, 1, 1), Matrix::Bt709, false);
    assert_matches_cpu("x420 40x22", &frame);
}

#[test]
fn ten_bit_low_aligned_planar_matches_cpu() {
    let frame = synthetic(16, 9, layout(false, 10, false, 1, 1), Matrix::Bt709, true);
    assert_matches_cpu("I010 16x9 full", &frame);
}

#[test]
fn twelve_bit_planar_matches_cpu() {
    let frame = synthetic(12, 8, layout(false, 12, false, 1, 1), Matrix::Bt601, false);
    assert_matches_cpu("I012 12x8", &frame);
}

#[test]
fn decoded_frame_matches_cpu() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests/fixtures/video/av1_8_420.mkv");
    let mut frames = FramePlayer::open(&path).expect("the fixture opens");
    let deadline = Instant::now() + Duration::from_secs(60);
    let frame = loop {
        match frames.next_frame().expect("no decode error") {
            Some(frame) => break frame,
            None => {
                assert!(Instant::now() < deadline, "no frame decoded");
                sleep(Duration::from_millis(1));
            }
        }
    };
    assert_matches_cpu("first frame of av1_8_420.mkv", &frame.image);
}

/// Draw the converted frame into a target of `format`, at the frame's size, and read it back. The
/// target matches the frame pixel for pixel, so the sampling lands exactly on the texels.
fn draw_into(format: wgpu::TextureFormat, frame: &Yuv) -> Option<Vec<u8>> {
    let (device, queue) = offscreen_device()?;
    let mut gpu = VideoGpu::new(&device, format);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    gpu.prepare(&device, &queue, &mut encoder, frame, 1);
    let (w, h) = (frame.width, frame.height);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("draw target"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("draw"),
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
        gpu.draw(&mut pass, [0.0, 0.0, w as f32, h as f32], [0, 0, w, h]);
    }
    let row = (w * 4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(row) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the device waits for the copy");
    receiver
        .recv()
        .expect("the map finishes")
        .expect("the buffer maps");
    let mapped = slice
        .get_mapped_range()
        .expect("the mapped range is readable");
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for line in mapped.chunks(row as usize).take(h as usize) {
        pixels.extend_from_slice(&line[..(w * 4) as usize]);
    }
    drop(mapped);
    buffer.unmap();
    Some(pixels)
}

#[test]
fn drawing_into_a_linear_target_matches_cpu() {
    let frame = synthetic(20, 12, layout(true, 8, false, 1, 1), Matrix::Bt709, false);
    let Some(drawn) = draw_into(wgpu::TextureFormat::Rgba8Unorm, &frame) else {
        eprintln!("no GPU here: skipped the linear draw");
        return;
    };
    let worst = drawn
        .iter()
        .zip(frame.to_rgba().pixels.iter())
        .map(|(a, b)| (i16::from(*a) - i16::from(*b)).abs())
        .max()
        .unwrap_or(0);
    assert!(worst <= 1, "largest difference is {worst}");
}

#[test]
fn drawing_into_an_srgb_target_keeps_the_colours() {
    // egui draws into an sRGB surface, where the hardware encodes what the shader writes. The
    // shader undoes the encoding first, so the stored bytes come back as the CPU's bytes.
    let frame = synthetic(20, 12, layout(true, 8, false, 1, 1), Matrix::Bt709, false);
    let Some(drawn) = draw_into(wgpu::TextureFormat::Rgba8UnormSrgb, &frame) else {
        eprintln!("no GPU here: skipped the sRGB draw");
        return;
    };
    let worst = drawn
        .iter()
        .zip(frame.to_rgba().pixels.iter())
        .map(|(a, b)| (i16::from(*a) - i16::from(*b)).abs())
        .max()
        .unwrap_or(0);
    assert!(worst <= 2, "largest difference is {worst}");
}
