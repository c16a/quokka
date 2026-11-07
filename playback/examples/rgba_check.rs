//! Write the first decoded frame of a file as RGBA bytes, to compare with another converter's
//! output. Run it with `cargo run -p quokka-playback --example rgba_check -- <file> <output>`.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use quokka_playback::decode::player::FramePlayer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, file, output] = args.as_slice() else {
        println!("usage: rgba_check <file> <output>");
        return;
    };
    let mut frames = FramePlayer::open(Path::new(file)).expect("open");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match frames.next_frame() {
            Ok(Some(frame)) => {
                let rgba = frame.image.to_rgba();
                std::fs::write(output, &rgba.pixels).expect("write");
                println!(
                    "first frame {}x{} written to {output}",
                    rgba.width, rgba.height
                );
                return;
            }
            Ok(None) if Instant::now() < deadline => sleep(Duration::from_millis(5)),
            _ => {
                println!("no frame");
                return;
            }
        }
    }
}
