use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use player_playback::SymphoniaSource;
use rodio::Source;

/// Write a 16-bit PCM WAV of `seconds` of silence at 8 kHz, mono.
fn write_wav(path: &Path, seconds: u32) {
    let rate: u32 = 8_000;
    let data_len = rate * seconds * 2;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.resize(out.len() + data_len as usize, 0);
    fs::write(path, out).unwrap();
}

fn temp_wav(name: &str, seconds: u32) -> PathBuf {
    let path = std::env::temp_dir().join(format!("player-test-{name}.wav"));
    write_wav(&path, seconds);
    path
}

#[test]
fn decodes_wav_with_expected_format_and_length() {
    let path = temp_wav("length", 2);
    let source = SymphoniaSource::open(&path, Default::default()).unwrap();
    assert_eq!(source.sample_rate().get(), 8_000);
    assert_eq!(source.channels().get(), 1);
    let total = source.total_duration().unwrap();
    assert!(
        (total.as_secs_f64() - 2.0).abs() < 0.01,
        "duration was {total:?}"
    );
    assert_eq!(source.count(), 16_000);
    fs::remove_file(path).ok();
}

#[test]
fn seeks_into_the_middle_and_plays_the_rest() {
    let path = temp_wav("seek", 2);
    let mut source = SymphoniaSource::open(&path, Default::default()).unwrap();
    source.try_seek(Duration::from_secs(1)).unwrap();
    let remaining = source.count();
    // Roughly one second of samples should be left after seeking to 1 s of a 2 s file.
    assert!(
        (7_000..=9_000).contains(&remaining),
        "remaining samples: {remaining}"
    );
    fs::remove_file(path).ok();
}

#[test]
fn unsupported_file_is_an_error_not_a_panic() {
    let path = std::env::temp_dir().join("player-test-not-audio.mp3");
    fs::write(&path, b"this is not audio").unwrap();
    assert!(SymphoniaSource::open(&path, Default::default()).is_err());
    fs::remove_file(path).ok();
}

/// Decode a real file: `PLAYER_TEST_FILE=/path/to/song.mp3 cargo test -p player-playback -- --ignored`.
#[test]
#[ignore]
fn decodes_a_real_file_from_env() {
    let path = std::env::var("PLAYER_TEST_FILE").expect("set PLAYER_TEST_FILE");
    let source = SymphoniaSource::open(Path::new(&path), Default::default()).unwrap();
    let rate = source.sample_rate().get();
    let channels = source.channels().get();
    let samples = source.count();
    assert!(samples > 0);
    println!("{rate} Hz, {channels} ch, {samples} samples");
}
