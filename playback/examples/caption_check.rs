//! Check the captions a movie shows at chosen times. Run it with
//! `cargo run -p player-playback --example caption_check -- <file with subtitles>`.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use player_playback::{FrameVideo, Player};

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        println!("usage: caption_check <file>");
        return;
    };
    let mut video = FrameVideo::new();
    if let Err(error) = video.load(Path::new(&path)) {
        println!("{path}: {error}");
        return;
    }
    for target in [0.05, 0.5, 1.2, 1.95] {
        video.seek(Duration::from_secs_f64(target)).expect("seek");
        let until = Instant::now() + Duration::from_millis(60);
        while Instant::now() < until {
            video.poll();
            sleep(Duration::from_millis(5));
        }
        println!("at {target:.2} s: {:?}", video.caption());
    }
    video.toggle_subtitles();
    println!("subtitles hidden, at 0.5 s: {:?}", video.caption());
}
