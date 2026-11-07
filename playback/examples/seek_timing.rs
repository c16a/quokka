//! Time a seek in a movie. The file's tables are read once, as the app does when a movie loads, and
//! then each seek opens the decoders at a new position. Run it with
//! `cargo run --release -p player-playback --example seek_timing -- <file> <seconds> <seconds>`.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use player_playback::decode::player::{FramePlayer, VideoSource};
use player_playback::movie_audio::AudioSource;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, file, first, second] = args.as_slice() else {
        println!("usage: seek_timing <file> <seconds> <seconds>");
        return;
    };
    let path = Path::new(file);

    let t = Instant::now();
    let video = VideoSource::read(path).expect("video");
    let audio = AudioSource::read(path).expect("audio");
    println!("load (read the tables once): {:?}", t.elapsed());

    for target in [first, second] {
        let target: f64 = target.parse().expect("seconds");
        let t = Instant::now();
        let mut frames: FramePlayer = video.open_at(target).expect("open frames");
        let opened = t.elapsed();
        let mut shown = None;
        while shown.is_none() {
            match frames.next_frame() {
                Ok(Some(frame)) => shown = Some(frame.time),
                Ok(None) => sleep(Duration::from_millis(1)),
                Err(e) => panic!("{e}"),
            }
        }
        let frame_time = t.elapsed();
        let t = Instant::now();
        let _sound = audio
            .as_ref()
            .map(|a| a.open_at(target).expect("open audio"));
        println!(
            "seek to {target}s: decoder {opened:?}, first picture {frame_time:?} (at {:.2}s), sound {:?}",
            shown.unwrap(),
            t.elapsed()
        );
    }
}
