//! The audio readers for MP4 and Matroska against the audio fixtures, checked against ffprobe.

use std::path::{Path, PathBuf};

use player_core::video::{AudioCodec, read_audio_samples};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/audio")
        .join(name)
}

#[test]
fn audio_tracks_give_their_codec_format_and_frames() {
    let cases = [
        ("aac_stereo.mp4", AudioCodec::Aac, 2, 95),
        ("aac_stereo.mkv", AudioCodec::Aac, 2, 95),
        ("ac3_51.mp4", AudioCodec::Ac3, 6, 63),
        ("ac3_51.mkv", AudioCodec::Ac3, 6, 63),
        ("eac3_51.mp4", AudioCodec::Eac3, 6, 63),
        ("eac3_stereo.mkv", AudioCodec::Eac3, 2, 63),
    ];
    for (name, codec, channels, frames) in cases {
        let path = fixture(name);
        let track = read_audio_samples(&path)
            .unwrap_or_else(|error| panic!("{name}: {error}"))
            .unwrap_or_else(|| panic!("{name}: no audio track"));
        assert_eq!(track.codec, codec, "{name}");
        assert_eq!(track.sample_rate, 48000, "{name}");
        assert_eq!(track.channels, channels, "{name}");
        assert_eq!(track.samples.len(), frames, "{name}: frame count");
        let length = std::fs::metadata(&path).unwrap().len();
        for sample in &track.samples {
            assert!(sample.size > 0, "{name}: empty frame");
            assert!(
                sample.offset + u64::from(sample.size) <= length,
                "{name}: frame outside the file"
            );
        }
    }
}

#[test]
fn aac_tracks_carry_their_audio_specific_config() {
    for name in ["aac_stereo.mp4", "aac_stereo.mkv"] {
        let track = read_audio_samples(&fixture(name)).unwrap().unwrap();
        // The first five bits are the audio object type, 2 for AAC LC.
        assert!(!track.config.is_empty(), "{name}: no config");
        assert_eq!(track.config[0] >> 3, 2, "{name}: AAC LC");
    }
}
