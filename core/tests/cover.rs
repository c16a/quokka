use std::path::Path;

use player_core::read_cover;

/// Reads the embedded cover of a real file: `PLAYER_TEST_FILE=/path/song.flac cargo test -p player-core -- --ignored`.
#[test]
#[ignore]
fn reads_cover_from_env() {
    let path = std::env::var("PLAYER_TEST_FILE").expect("set PLAYER_TEST_FILE");
    let cover = read_cover(Path::new(&path));
    println!("cover bytes: {:?}", cover.as_ref().map(Vec::len));
    assert!(cover.is_some());
}
