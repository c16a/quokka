//! Match each movie in a library database to a TMDb movie and print the details, so the matches
//! can be checked by eye. Nothing is written to the database.
//!
//! Usage: `TMDB_API_KEY=... just match-tmdb ./library.db`

use std::error::Error;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use quokka_core::Library;
use quokka_core::matching::{self, MATCH_THRESHOLD};
use quokka_core::tmdb::{self, Tmdb};

/// TMDb allows about 40 requests every 10 seconds. Pausing between movies keeps well under that.
const PAUSE: Duration = Duration::from_millis(300);

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let [_, db] = args.as_slice() else {
        return Err("usage: match-tmdb <db>   (set TMDB_API_KEY first)".into());
    };
    let tmdb = Tmdb::from_env()?;
    let library = Library::open(Path::new(db))?;
    let movies = library.movies()?;

    let mut matched = 0;
    for movie in &movies {
        let Some(stem) = movie.path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let parsed = matching::parse_file_name(stem);
        println!("file:   {stem}");
        println!(
            "parsed: {} ({})",
            parsed.title,
            parsed.year.map_or("no year".to_owned(), |y| y.to_string())
        );

        let mut hits = tmdb.search(&parsed.title, parsed.year)?;
        if hits.is_empty() && parsed.year.is_some() {
            hits = tmdb.search(&parsed.title, None)?;
        }
        let candidates = tmdb::candidates(&hits);
        let best = matching::best_matches(&parsed, &candidates, 1);
        match best.first() {
            Some(hit) if hit.score >= MATCH_THRESHOLD => {
                matched += 1;
                let info = tmdb.movie(hit.candidate.id.parse()?)?;
                println!(
                    "  match {:.2}  {} ({})  tmdb {}",
                    hit.score,
                    info.title,
                    info.year.map_or("?".to_owned(), |y| y.to_string()),
                    info.id
                );
                println!(
                    "        {} min · {}",
                    info.runtime_minutes
                        .map_or("?".to_owned(), |m| m.to_string()),
                    info.genres.join(", ")
                );
                if !info.directors.is_empty() {
                    println!("        directed by {}", info.directors.join(", "));
                }
                let cast: Vec<String> = info
                    .cast
                    .iter()
                    .take(5)
                    .map(|c| format!("{} as {}", c.name, c.character))
                    .collect();
                if !cast.is_empty() {
                    println!("        cast: {}", cast.join("; "));
                }
                if let Some(tagline) = &info.tagline {
                    println!("        \"{tagline}\"");
                }
            }
            Some(hit) => println!(
                "  no confident match (best {:.2}: {} {})",
                hit.score, hit.candidate.title, hit.candidate.id
            ),
            None => println!("  no results"),
        }
        println!();
        sleep(PAUSE);
    }
    println!(
        "{matched} of {} movies matched at a score of {MATCH_THRESHOLD} or more",
        movies.len()
    );
    Ok(())
}
