//! Times the library queries the UI runs most often, on a database filled by `library_fixture`.
//! Each query runs several times and the average is printed.

use std::path::Path;
use std::time::Instant;

use quokka_core::{AlbumFilter, AlbumSort, Library, SongQuery};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = std::env::args()
        .nth(1)
        .ok_or("usage: bench_queries <database>")?;
    let library = Library::open(Path::new(&db))?;
    println!("{} rows", library.count()?);

    let albums = library.albums_by(AlbumSort::Name, &AlbumFilter::default())?;
    println!("{} albums", albums.len());
    let names: Vec<Option<String>> = albums.iter().map(|a| a.album.clone()).collect();

    time("albums (by name)", 5, || {
        library
            .albums_by(AlbumSort::Name, &AlbumFilter::default())
            .map(|a| a.len())
    })?;
    time("album_tracks, every album", 1, || {
        let mut total = 0;
        for name in &names {
            total += library.album_tracks(name.as_deref())?.len();
        }
        Ok(total)
    })?;
    time("album_cover, every album", 1, || {
        let mut found = 0;
        for name in &names {
            found += usize::from(library.album_cover(name.as_deref())?.is_some());
        }
        Ok(found)
    })?;
    time("search words (FTS)", 20, || {
        library.search("track 1", 100).map(|v| v.len())
    })?;
    time("search songs by title", 20, || {
        library
            .search_songs(&SongQuery::parse("track 1"), 200)
            .map(|v| v.len())
    })?;
    Ok(())
}

/// Runs `query` `runs` times and prints the average time per run.
fn time(
    label: &str,
    runs: u32,
    query: impl Fn() -> quokka_core::Result<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut result = 0;
    let start = Instant::now();
    for _ in 0..runs {
        result = query()?;
    }
    let average = start.elapsed() / runs;
    println!("{label:<28} {average:>12.3?}  (result {result})");
    Ok(())
}
