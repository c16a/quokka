//! Downloads the real station index. Needs a network connection, so it is ignored by default.

#[test]
#[ignore = "downloads the radio-browser.info station index"]
fn the_whole_station_index_downloads() {
    let stations = player_core::fetch_all_stations().unwrap();
    println!("stations: {}", stations.len());
    assert!(stations.len() > 1000, "more than the first page");
    assert!(
        stations
            .iter()
            .all(|s| !s.uuid.is_empty() && !s.url.is_empty())
    );
}
