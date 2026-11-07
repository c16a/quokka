//! Connects to a live internet radio stream. Needs a network connection, so it is ignored by
//! default. Set QUOKKA_TEST_STREAM to a stream address to run it.

use rodio::Source;

#[test]
#[ignore = "needs a live stream: set QUOKKA_TEST_STREAM"]
fn a_live_stream_decodes() {
    let url = std::env::var("QUOKKA_TEST_STREAM").expect("QUOKKA_TEST_STREAM");
    let source = quokka_playback::open_stream(&url).unwrap();
    assert_eq!(source.total_duration(), None, "a live stream has no length");
    let samples: Vec<f32> = source.take(200_000).collect();
    assert!(samples.len() >= 100_000, "got {} samples", samples.len());
    assert!(samples.iter().any(|s| s.abs() > 0.0001), "some sound");
}
