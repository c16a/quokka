//! Time each step of loading a movie, as the Video window does: read the video and audio tables,
//! open the first frames, read the subtitles, then open the sound. Run it with
//! `cargo run [--release] -p player-playback --example load_timing -- <file>`.

use std::path::Path;
use std::time::Instant;

use player_playback::decode::player::VideoSource;
use player_playback::movie_audio::AudioSource;

fn main() {
    let Some(file) = std::env::args().nth(1) else {
        println!("usage: load_timing <file>");
        return;
    };
    let path = Path::new(&file);

    let t = Instant::now();
    let video = VideoSource::read(path).expect("video");
    println!("video tables (read_samples + inspect): {:?}", t.elapsed());

    let t = Instant::now();
    let audio = AudioSource::read(path).expect("audio");
    println!("audio tables (read_audio_samples): {:?}", t.elapsed());

    let t = Instant::now();
    let _frames = video.open_at(0.0).expect("frames");
    println!("video decoder open + first sample: {:?}", t.elapsed());

    let t = Instant::now();
    let _cues = player_core::video::read_subtitles(path).unwrap_or_default();
    println!("subtitles: {:?}", t.elapsed());

    let t = Instant::now();
    let _sound = audio.as_ref().map(|a| a.open_at(0.0).expect("sound"));
    println!(
        "sound (audio decoder + output device, on the UI thread): {:?}",
        t.elapsed()
    );
}
