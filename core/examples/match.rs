//! Match each movie in a library database to a title in an IMDb `title.basics` file, and print the
//! best candidates with their scores, so the matches can be checked by eye.
//!
//! Usage: `just match-titles ./library.db ~/Downloads/title.basics.tsv.gz`

use std::error::Error;
use std::path::Path;

use quokka_core::Library;
use quokka_core::matching::{self, MATCH_THRESHOLD};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let [_, db, titles] = args.as_slice() else {
        return Err("usage: match <db> <title.basics.tsv[.gz]>".into());
    };
    let library = Library::open(Path::new(db))?;
    let candidates = matching::read_imdb_movies(Path::new(titles))?;
    eprintln!("{} movie titles loaded", candidates.len());

    let mut matched = 0;
    let movies = library.movies()?;
    for movie in &movies {
        let Some(stem) = movie.path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let parsed = matching::parse_file_name(stem);
        let best = matching::best_matches(&parsed, &candidates, 3);
        let year = |y: Option<i64>| y.map_or("?".to_owned(), |y| y.to_string());
        println!("file:   {stem}");
        println!("parsed: {} ({})", parsed.title, year(parsed.year));
        for (rank, hit) in best.iter().enumerate() {
            let flag = if rank == 0 && hit.score >= MATCH_THRESHOLD {
                matched += 1;
                "match"
            } else {
                "     "
            };
            println!(
                "  {flag} {:.2}  {} ({})  {}",
                hit.score,
                hit.candidate.title,
                year(hit.candidate.year),
                hit.candidate.id
            );
        }
        println!();
    }
    println!(
        "{matched} of {} movies matched at a score of {MATCH_THRESHOLD} or more",
        movies.len()
    );
    Ok(())
}
