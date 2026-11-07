use std::path::{Path, PathBuf};

use player_core::matching::{self, MATCH_THRESHOLD, ParsedName};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn parsed(title: &str, year: Option<i64>) -> ParsedName {
    ParsedName {
        title: title.into(),
        year,
    }
}

#[test]
fn release_names_are_cleaned_to_a_title_and_year() {
    assert_eq!(
        matching::parse_file_name("The.Matrix.1999.1080p.BluRay.x264-GROUP"),
        parsed("The Matrix", Some(1999))
    );
    assert_eq!(
        matching::parse_file_name("Spider-Man.2002.2160p.WEB-DL.DV.HDR.AV1"),
        parsed("Spider Man", Some(2002))
    );
    assert_eq!(
        matching::parse_file_name("Schindler's List (1993) [1080p]"),
        parsed("Schindlers List", Some(1993))
    );
}

#[test]
fn a_year_at_the_start_of_a_title_is_part_of_the_title() {
    assert_eq!(
        matching::parse_file_name("2001 A Space Odyssey (1968)"),
        parsed("2001 A Space Odyssey", Some(1968))
    );
}

#[test]
fn technical_tags_without_a_year_leave_just_the_title() {
    assert_eq!(
        matching::parse_file_name("Sparks-5994fps-AV1-10bit-3840x2160-5967kbps"),
        parsed("Sparks", None)
    );
}

#[test]
fn similarity_ignores_case_and_punctuation() {
    assert_eq!(matching::similarity("The Matrix", "the matrix!"), 1.0);
    assert!(matching::similarity("Spider Man", "Spider-Man: Homecoming") < 0.6);
    // One transposed letter costs two edits, so the score is 1 - 2/9.
    assert!(matching::similarity("Inception", "Inceptoin") > 0.75);
}

#[test]
fn the_imdb_sample_loads_movies_only_and_keeps_original_titles() {
    let candidates = matching::read_imdb_movies(&fixture("title.basics.sample.tsv.gz")).unwrap();
    let ids: Vec<&str> = candidates.iter().map(|c| c.id.as_str()).collect();
    // The short film and the adult title are left out.
    assert!(!ids.contains(&"tt0000001"));
    assert!(!ids.contains(&"tt0000009"));
    assert!(ids.contains(&"tt0133093"));
    assert!(candidates.iter().any(|c| c.title == "Schindler's List"));
}

#[test]
fn a_file_name_matches_its_movie() {
    let candidates = matching::read_imdb_movies(&fixture("title.basics.sample.tsv.gz")).unwrap();

    let matrix = matching::parse_file_name("The.Matrix.1999.1080p.BluRay.x264");
    let best = &matching::best_matches(&matrix, &candidates, 1)[0];
    assert_eq!(best.candidate.id, "tt0133093");
    assert!(best.score >= MATCH_THRESHOLD);

    let homecoming = matching::parse_file_name("Spider-Man.Homecoming.2017.1080p");
    let best = &matching::best_matches(&homecoming, &candidates, 1)[0];
    assert_eq!(best.candidate.id, "tt2250912");

    let spider_man = matching::parse_file_name("Spider-Man.2002.2160p.WEB-DL");
    let best = &matching::best_matches(&spider_man, &candidates, 1)[0];
    assert_eq!(best.candidate.id, "tt0145487");
    assert!(best.score >= MATCH_THRESHOLD);
}

#[test]
fn a_year_mismatch_pulls_the_score_down() {
    let candidates = matching::read_imdb_movies(&fixture("title.basics.sample.tsv.gz")).unwrap();
    let right_year =
        matching::best_matches(&parsed("Inception", Some(2010)), &candidates, 1)[0].score;
    let wrong_year =
        matching::best_matches(&parsed("Inception", Some(1985)), &candidates, 1)[0].score;
    assert!(right_year > wrong_year);
}

#[test]
fn an_unknown_title_stays_below_the_threshold() {
    let candidates = matching::read_imdb_movies(&fixture("title.basics.sample.tsv.gz")).unwrap();
    let best = &matching::best_matches(&parsed("Sparks", None), &candidates, 1)[0];
    assert!(best.score < MATCH_THRESHOLD, "scored {}", best.score);
}
