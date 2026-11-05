use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use player_playback::{Player, PlayerState, RodioPlayer};

/// Needs an audio output device, so it only runs on demand:
/// `PLAYER_TEST_FILE=/path/to/song.flac cargo test -p player-playback -- --ignored`.
#[test]
#[ignore]
fn plays_seeks_and_pauses_a_real_file() {
    let path = std::env::var("PLAYER_TEST_FILE").expect("set PLAYER_TEST_FILE");
    let mut player = RodioPlayer::new().expect("open audio output");

    player.load(Path::new(&path)).unwrap();
    assert_eq!(player.state(), PlayerState::Paused);
    assert!(player.duration().is_some());

    player.play().unwrap();
    assert_eq!(player.state(), PlayerState::Playing);
    sleep(Duration::from_millis(1500));
    let before_seek = player.position().unwrap();
    assert!(
        before_seek >= Duration::from_secs(1),
        "position: {before_seek:?}"
    );

    player.seek(Duration::from_secs(30)).unwrap();
    sleep(Duration::from_millis(300));
    let after_seek = player.position().unwrap();
    assert!(
        after_seek >= Duration::from_secs(29) && after_seek < Duration::from_secs(32),
        "position after seek: {after_seek:?}"
    );

    player.pause().unwrap();
    assert_eq!(player.state(), PlayerState::Paused);
    player.stop().unwrap();
    assert_eq!(player.state(), PlayerState::Idle);
}
