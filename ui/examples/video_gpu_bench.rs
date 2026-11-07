//! Compare the cost of getting a decoded frame onto the screen: the old path, which converts to RGBA
//! on the CPU and uploads RGBA, and the GPU path, which uploads the planes and converts on the GPU.
//! Each timed frame waits for the GPU, so the numbers include the GPU's work.
//! Run it on a release build: `cargo run --release -p quokka-ui --example video_gpu_bench -- <file> <frames> <rounds>`.

#[cfg(target_os = "macos")]
fn main() {
    use std::path::Path;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use eframe::egui;
    use eframe::egui_wgpu::wgpu;
    use quokka_playback::decode::frames::Yuv;
    use quokka_playback::decode::player::FramePlayer;
    use quokka_ui::video_gpu::{VideoGpu, offscreen_device};

    let args: Vec<String> = std::env::args().collect();
    let [_, file, frames, rounds] = args.as_slice() else {
        println!("usage: video_gpu_bench <file> <frames> <rounds>");
        return;
    };
    let frames: usize = frames.parse().expect("frames");
    let rounds: usize = rounds.parse().expect("rounds");

    // Decode the frames first, so the timings below leave decoding out.
    let mut player = FramePlayer::open(Path::new(file)).expect("open");
    let mut decoded: Vec<Yuv> = Vec::with_capacity(frames);
    let deadline = Instant::now() + Duration::from_secs(300);
    while decoded.len() < frames {
        match player.next_frame().expect("decode") {
            Some(frame) => decoded.push(frame.image),
            None if player.finished() => break,
            None => {
                assert!(Instant::now() < deadline, "decoding did not finish");
                sleep(Duration::from_millis(1));
            }
        }
    }
    let (width, height) = (decoded[0].width, decoded[0].height);
    println!(
        "{} frames of {width}x{height}, {rounds} rounds",
        decoded.len()
    );

    let Some((device, queue)) = offscreen_device() else {
        println!("no GPU on this machine");
        return;
    };

    // The old path: convert on the CPU, make an egui image, and upload it as RGBA.
    let rgba_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("old path"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let old_path = |frame: &Yuv| {
        let rgba = frame.to_rgba();
        let color = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &rgba.pixels,
        );
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &rgba_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            color.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        queue.submit([encoder.finish()]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    };

    // The GPU path: upload the planes, convert them on the GPU, and wait for the GPU.
    let mut gpu = VideoGpu::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    let mut shown = 0u64;
    let mut new_path = |frame: &Yuv| {
        shown += 1;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        gpu.prepare(&device, &queue, &mut encoder, frame, shown);
        queue.submit([encoder.finish()]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    };

    let mut old_total = Duration::ZERO;
    let mut new_total = Duration::ZERO;
    let mut timed = 0u32;
    // One round first to warm up the caches and the GPU, then the timed rounds.
    for round in 0..=rounds {
        for frame in &decoded {
            let start = Instant::now();
            old_path(frame);
            let old = start.elapsed();
            let start = Instant::now();
            new_path(frame);
            let new = start.elapsed();
            if round > 0 {
                old_total += old;
                new_total += new;
                timed += 1;
            }
        }
    }
    let old_ms = old_total.as_secs_f64() * 1000.0 / f64::from(timed);
    let new_ms = new_total.as_secs_f64() * 1000.0 / f64::from(timed);
    println!(
        "old path (CPU convert, RGBA upload): {old_ms:.2} ms a frame, {:.1} frames a second",
        1000.0 / old_ms
    );
    println!(
        "GPU path (plane upload, GPU convert): {new_ms:.2} ms a frame, {:.1} frames a second",
        1000.0 / new_ms
    );
    println!("speedup of the UI-side work: {:.2}x", old_ms / new_ms);
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("the GPU path is for macOS");
}
