//! Needs a real audio output. Run with `just test`'s binary and `--ignored`.

use std::path::PathBuf;
use std::time::Duration;

use player_playback::{Player, RodioPlayer};

/// Write one second of silence as a 16-bit mono WAV file at `rate`.
fn silent_wav(path: &PathBuf, rate: u32) {
    let samples = rate as usize;
    let data_len = (samples * 2) as u32;
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.resize(out.len() + samples * 2, 0);
    std::fs::write(path, out).unwrap();
}

#[test]
#[ignore = "needs an audio output device"]
fn output_follows_the_track_rate() {
    let dir = std::env::temp_dir();
    let (a, b) = (dir.join("rate-44100.wav"), dir.join("rate-48000.wav"));
    silent_wav(&a, 44_100);
    silent_wav(&b, 48_000);

    let mut player = RodioPlayer::open(None).unwrap();
    player.load(&a).unwrap();
    assert_eq!(
        player.output_rate(),
        44_100,
        "device should run at the track's rate"
    );
    player.load(&b).unwrap();
    assert_eq!(player.output_rate(), 48_000);
    player.stop().unwrap();
    let _ = (
        std::fs::remove_file(a),
        std::fs::remove_file(b),
        Duration::ZERO,
    );
}
