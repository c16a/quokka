use std::path::Path;

use quokka_core::MediaKind;

#[test]
fn unsupported_formats_are_not_indexed() {
    for name in ["a.wma", "a.ape", "a.wv"] {
        assert_eq!(MediaKind::from_path(Path::new(name)), None, "{name}");
    }
}

#[test]
fn supported_formats_are_indexed() {
    for name in [
        "a.mp3", "a.flac", "a.m4a", "a.ogg", "a.opus", "a.wav", "a.aiff", "a.mka",
    ] {
        assert_eq!(
            MediaKind::from_path(Path::new(name)),
            Some(MediaKind::Audio),
            "{name}"
        );
    }
}
