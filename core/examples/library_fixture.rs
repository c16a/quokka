use std::error::Error;
use std::path::Path;

use quokka_core::Library;
use rusqlite::{Connection, params};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let [_, db, albums, tracks_per_album, movies] = args.as_slice() else {
        return Err("usage: library_fixture <db> <albums> <tracks_per_album> <movies>".into());
    };
    let albums = parse_count("albums", albums)?;
    let tracks_per_album = parse_count("tracks_per_album", tracks_per_album)?;
    let movies = parse_count("movies", movies)?;
    let db = Path::new(db);

    // Opening the library creates the schema when the file is new.
    drop(Library::open(db)?);

    let mut conn = Connection::open(db)?;
    let added = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let tx = conn.transaction()?;
    {
        let mut audio = tx.prepare(
            "INSERT INTO media
                (path, kind, size, mtime, title, artist, album_artist, album, genre, track, year, codec, added)
             VALUES (?1, 'audio', 1000000, 1, ?2, ?3, ?3, ?4, ?5, ?6, ?7, 'FLAC', ?8)",
        )?;
        for a in 1..=albums {
            let artist = format!("Artist {}", a % 300);
            let album = format!("Album {a}");
            let genre = format!("Genre {}", a % 20);
            let year = 1960 + (a % 60) as i64;
            for t in 1..=tracks_per_album {
                let track = t as i64;
                audio.execute(params![
                    format!("/fixture/album-{a}/track-{t}.flac"),
                    format!("Track {t}"),
                    artist,
                    album,
                    genre,
                    track,
                    year,
                    added,
                ])?;
            }
        }
    }
    {
        let mut movie = tx.prepare(
            "INSERT INTO media (path, kind, size, mtime, title, processed)
             VALUES (?1, 'video', 1000000000, 1, ?2, 1)",
        )?;
        for m in 1..=movies {
            movie.execute(params![
                format!("/fixture/movies/movie-{m}.mkv"),
                format!("Movie {m}"),
            ])?;
        }
    }
    tx.commit()?;

    let audio_rows: i64 = conn.query_row(
        "SELECT COUNT(*) FROM media WHERE kind = 'audio'",
        [],
        |row| row.get(0),
    )?;
    let video_rows: i64 = conn.query_row(
        "SELECT COUNT(*) FROM media WHERE kind = 'video'",
        [],
        |row| row.get(0),
    )?;
    println!("audio rows: {audio_rows}");
    println!("video rows: {video_rows}");
    eprintln!("warning: these paths do not exist on disk, so cover reads return nothing");
    Ok(())
}

fn parse_count(name: &str, value: &str) -> Result<u64, Box<dyn Error>> {
    value
        .parse()
        .map_err(|e| format!("{name} must be a whole number, got {value:?}: {e}").into())
}
