use std::fs;
use std::path::{Path, PathBuf};

use quokka_core::matching::{self, MATCH_THRESHOLD};
use quokka_core::tmdb;

fn fixture(name: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path).unwrap()
}

#[test]
fn a_search_response_gives_ids_titles_and_years() {
    let hits = tmdb::parse_search(&fixture("tmdb_search_matrix.json")).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, 603);
    assert_eq!(hits[0].title, "The Matrix");
    assert_eq!(hits[0].year, Some(1999));
}

#[test]
fn a_movie_response_gives_details_and_credits() {
    let info = tmdb::parse_movie(&fixture("tmdb_movie_matrix.json")).unwrap();
    assert_eq!(info.id, 603);
    assert_eq!(info.year, Some(1999));
    assert_eq!(info.runtime_minutes, Some(136));
    assert_eq!(info.genres, vec!["Action", "Science Fiction"]);
    assert_eq!(info.tagline.as_deref(), Some("Welcome to the Real World."));
    assert_eq!(info.poster_path.as_deref(), Some("/sample-poster.jpg"));
    assert_eq!(info.directors, vec!["Lana Wachowski", "Lilly Wachowski"]);
    assert_eq!(info.writers, vec!["Lana Wachowski"]);
}

#[test]
fn cast_is_in_billing_order_with_characters() {
    let info = tmdb::parse_movie(&fixture("tmdb_movie_matrix.json")).unwrap();
    let names: Vec<(&str, &str)> = info
        .cast
        .iter()
        .map(|c| (c.name.as_str(), c.character.as_str()))
        .collect();
    assert_eq!(
        names,
        vec![
            ("Keanu Reeves", "Neo"),
            ("Carrie-Anne Moss", "Trinity"),
            ("Laurence Fishburne", "Morpheus"),
        ]
    );
}

#[test]
fn a_file_name_matches_its_tmdb_movie() {
    let hits = tmdb::parse_search(&fixture("tmdb_search_matrix.json")).unwrap();
    let candidates = tmdb::candidates(&hits);
    let parsed = matching::parse_file_name("The.Matrix.1999.1080p.BluRay.x264");
    let best = &matching::best_matches(&parsed, &candidates, 1)[0];
    assert_eq!(best.candidate.id, "603");
    assert!(best.score >= MATCH_THRESHOLD);
}

#[test]
fn the_year_picks_the_right_sequel() {
    let hits = tmdb::parse_search(&fixture("tmdb_search_matrix.json")).unwrap();
    let candidates = tmdb::candidates(&hits);
    let parsed = matching::parse_file_name("The.Matrix.Reloaded.2003.1080p");
    let best = &matching::best_matches(&parsed, &candidates, 1)[0];
    assert_eq!(best.candidate.id, "604");
}

#[test]
fn a_client_without_a_key_says_so() {
    // SAFETY: this test is the only one in this binary that touches the variable.
    unsafe { std::env::remove_var("TMDB_API_KEY") };
    let error = tmdb::Tmdb::from_env().err().expect("no key is an error");
    assert!(error.to_string().contains("TMDB_API_KEY"));
}

#[test]
fn people_keep_their_tmdb_photos() {
    let info = tmdb::parse_movie(&fixture("tmdb_movie_matrix.json")).unwrap();
    assert_eq!(
        info.cast[0].profile_path.as_deref(),
        Some("/sample-keanu.jpg")
    );
    assert_eq!(info.cast[1].profile_path, None);
    assert_eq!(
        info.photos.get("Lana Wachowski").map(String::as_str),
        Some("/sample-lana.jpg")
    );
}
