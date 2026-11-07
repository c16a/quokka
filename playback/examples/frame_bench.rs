//! Measure how fast a file's frames decode and convert, from the start of the file. Run it on a
//! release build: `cargo run --release -p quokka-playback --example frame_bench -- <file> <seconds>`.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use quokka_playback::decode::player::FramePlayer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, file, seconds] = args.as_slice() else {
        println!("usage: frame_bench <file> <seconds>");
        return;
    };
    let seconds: f64 = seconds.parse().expect("seconds");
    let mut frames = FramePlayer::open(Path::new(file)).expect("open");
    let start = Instant::now();
    let mut count = 0usize;
    let mut video_seconds = 0.0;
    while start.elapsed().as_secs_f64() < seconds {
        match frames.next_frame() {
            Ok(Some(frame)) => {
                count += 1;
                video_seconds = frame.time;
            }
            Ok(None) if frames.finished() => break,
            Ok(None) => sleep(Duration::from_millis(1)),
            Err(error) => {
                println!("{file}: {error}");
                return;
            }
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "{file}: {count} frames in {elapsed:.2} s wall time, {:.1} frames per second; reached {video_seconds:.2} s of video",
        count as f64 / elapsed
    );
}
