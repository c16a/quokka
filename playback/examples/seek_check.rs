//! Check that seeking a movie moves its picture and its sound to the same place.
//! Run it with `cargo run -p quokka-playback --example seek_check -- <file with video and sound>`.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use quokka_playback::{FrameVideo, Player};

fn run_for(video: &mut FrameVideo, seconds: f64) {
    let until = Instant::now() + Duration::from_secs_f64(seconds);
    while Instant::now() < until {
        video.poll();
        sleep(Duration::from_millis(5));
    }
}

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        println!("usage: seek_check <file>");
        return;
    };
    let mut video = FrameVideo::new();
    if let Err(error) = video.load(Path::new(&path)) {
        println!("{path}: {error}");
        return;
    }
    video.seek(Duration::from_secs_f64(1.0)).expect("seek");
    run_for(&mut video, 0.1);
    let paused_at = video.position().map_or(0.0, |p| p.as_secs_f64());
    let shown_paused = video.shown();
    video.play().expect("play");
    run_for(&mut video, 0.5);
    let played_to = video.position().map_or(0.0, |p| p.as_secs_f64());
    println!(
        "after seeking to 1.0 s: paused clock {paused_at:.3} s, {shown_paused} frames shown; \
         half a second of play later the clock is {played_to:.3} s, {} frames shown",
        video.shown()
    );
    video.pause().expect("pause");
    video.seek(Duration::from_secs_f64(0.2)).expect("seek back");
    run_for(&mut video, 0.1);
    println!(
        "seek back to 0.2 s: paused clock {:.3} s",
        video.position().map_or(0.0, |p| p.as_secs_f64())
    );
}
