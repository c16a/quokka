use std::path::Path;

use quokka_core::read_cover;

/// Reads the embedded cover of a real file: `QUOKKA_TEST_FILE=/path/song.flac cargo test -p quokka-core -- --ignored`.
#[test]
#[ignore]
fn reads_cover_from_env() {
    let path = std::env::var("QUOKKA_TEST_FILE").expect("set QUOKKA_TEST_FILE");
    let cover = read_cover(Path::new(&path));
    println!("cover bytes: {:?}", cover.as_ref().map(Vec::len));
    assert!(cover.is_some());
}
