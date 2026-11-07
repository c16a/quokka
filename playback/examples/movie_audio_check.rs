//! Check that a movie's sound clock holds still while paused, and runs again after resuming.
//! Run it with `cargo run -p player-playback --example movie_audio_check -- <file with sound>`.

use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use player_playback::movie_audio::MovieAudio;

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        println!("usage: movie_audio_check <file with sound>");
        return;
    };
    let audio = match MovieAudio::open(Path::new(&path)) {
        Ok(Some(audio)) => audio,
        Ok(None) => {
            println!("{path}: no sound this reads");
            return;
        }
        Err(error) => {
            println!("{path}: {error}");
            return;
        }
    };
    audio.play();
    sleep(Duration::from_millis(600));
    let playing = audio.clock();
    audio.pause();
    sleep(Duration::from_millis(100));
    let paused_start = audio.clock();
    sleep(Duration::from_millis(900));
    let paused_end = audio.clock();
    audio.play();
    sleep(Duration::from_millis(400));
    let resumed = audio.clock();
    for tick in 0..40 {
        if audio.finished() {
            println!(
                "finished after {} ms with clock {:.3} s",
                tick * 100,
                audio.clock()
            );
            return;
        }
        sleep(Duration::from_millis(100));
    }
    println!("not finished after 4 s; clock {:.3} s", audio.clock());
    println!(
        "playing at {playing:.3} s; paused at {paused_start:.3} s, still {paused_end:.3} s after 0.9 s; resumed to {resumed:.3} s"
    );
}
