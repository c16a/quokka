//! Every frame of an AV1 file comes out of the decoder. The decoder runs frames in parallel, so
//! frames can still be in flight at the end of the file, and those must be drained rather than
//! dropped.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use quokka_playback::decode::player::FramePlayer;

fn count_frames(path: &Path) -> (usize, usize) {
    let mut player = FramePlayer::open(path).expect("the file opens");
    let expected = player.frame_count;
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut count = 0;
    loop {
        match player.next_frame().expect("no decode error") {
            Some(_) => count += 1,
            None if player.finished() => break,
            None => {
                assert!(Instant::now() < deadline, "decoding did not finish");
                sleep(Duration::from_millis(1));
            }
        }
    }
    (count, expected)
}

#[test]
fn av1_keeps_every_frame() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests/fixtures/video");
    for name in ["av1_8_420.mkv", "av1_10_420.mkv", "av1_8_420.mp4"] {
        let (count, expected) = count_frames(&fixtures.join(name));
        assert_eq!(count, expected, "frames out of {name}");
    }
}
