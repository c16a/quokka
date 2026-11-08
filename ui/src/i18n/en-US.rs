//! American English, the UI's source text. Every string is here, and every other language falls
//! back to it.

use super::Translation;

pub struct EnUs;

impl Translation for EnUs {
    fn language_tags(&self) -> &'static [&'static str] {
        &["en-US", "en"]
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Name")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Year")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Date added")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Albums")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("No albums match these filters.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("No albums yet. Choose your library folder in Settings.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("All genres")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("All years")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("All albums")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("Unknown album")
    }
    fn play(&self) -> Option<&'static str> {
        Some("Play")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("Shuffle")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("Remove from favourites")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("Add to favourites")
    }
    fn album(&self) -> Option<&'static str> {
        Some("ALBUM")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("No artists yet. Choose your library folder in Settings.")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("Artists")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("All artists")
    }
    // format.rs
    fn mono(&self) -> Option<&'static str> {
        Some("Mono")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("Stereo")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("Add a folder to get started.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("Syncing the radio station list…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("Radio station sync stopped unexpectedly.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("the scan stopped unexpectedly")
    }
    fn video(&self) -> Option<&'static str> {
        Some("Video")
    }
    fn search(&self) -> Option<&'static str> {
        Some("Search")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("Recent")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("Playlists")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("Radio")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("Movies")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("Settings")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("Scanning…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("Saved the TMDb key.")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("Hide sidebar")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("Show sidebar")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("No movies yet. Choose your library folder in Settings.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("MOVIE")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Length unknown")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Start over")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("Wrong movie? Choose another")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Remove details")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("File information")
    }
    fn source(&self) -> Option<&'static str> {
        Some("SOURCE")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("Finding details…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Find details on TMDb")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("Add your TMDb API key in Settings to see the description and cast.")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("Synopsis")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("Cast")
    }
    fn director(&self) -> Option<&'static str> {
        Some("Director")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("Writer")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Crew")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("Which movie is this?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("Pick the one your file is. Nothing is saved until you pick.")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("None of these")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("Left unmatched.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("This one")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("the lookup stopped unexpectedly")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("No TMDb details for this movie.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Unknown")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Container")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("Codec")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("Colour depth")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("Chroma subsampling")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("Dynamic range")
    }
    fn hdr(&self) -> Option<&'static str> {
        Some("HDR")
    }
    fn sdr(&self) -> Option<&'static str> {
        Some("SDR")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("Resolution")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("Aspect ratio")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Frame rate")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Length")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("File size")
    }
    fn mp4(&self) -> Option<&'static str> {
        Some("MP4")
    }
    fn m4v(&self) -> Option<&'static str> {
        Some("M4V")
    }
    fn matroska_mkv(&self) -> Option<&'static str> {
        Some("Matroska (MKV)")
    }
    fn webm(&self) -> Option<&'static str> {
        Some("WebM")
    }
    fn quicktime_mov(&self) -> Option<&'static str> {
        Some("QuickTime (MOV)")
    }
    fn avi(&self) -> Option<&'static str> {
        Some("AVI")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("MPEG transport stream")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("MPEG program stream")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("Raw AV1 stream (OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("no audio output device available")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("Could not connect to the station.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("The station keeps dropping the connection. Press play to try again.")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pause")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("Connecting…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("Paused")
    }
    fn live(&self) -> Option<&'static str> {
        Some("Live")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("No audio output device available")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("Back 10 seconds (Left arrow)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("Pause (Space)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("Play (Space)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("Forward 10 seconds (Right arrow)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("Exit full screen (Escape)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Full screen (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Unmute")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Mute")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("Loading video…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("Previous track")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("Next track")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("Repeat")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("Nothing playing")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Output")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("System default")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("Up next")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Lyrics")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some("Playback speed. Click to change. Speeds other than 1× also change the pitch.")
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("Track info")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("New playlist name")
    }
    fn create(&self) -> Option<&'static str> {
        Some("Create")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("No playlists yet. Add tracks from an album or the search results.")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Rename")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Delete playlist")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("All playlists")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Save")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("Cancel")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("Play playlist")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("Shuffle playlist")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("Up Next")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("Nothing queued")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("Clear")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("Tracks you queue will appear here.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("Play next")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("Add to queue")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("Add to playlist")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("No playlists yet")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("No stations yet.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Sync now")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("Syncing…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("Your stations")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Find a station below and tune in. It will appear here.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Find stations")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Sync the station list to search it.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("No stations match.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("Country: any")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("Any country")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("Codec: any")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("Any codec")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("Bitrate: any")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("Any bitrate")
    }
    fn text_64_kbps(&self) -> Option<&'static str> {
        Some("64+ kbps")
    }
    fn text_128_kbps(&self) -> Option<&'static str> {
        Some("128+ kbps")
    }
    fn text_256_kbps(&self) -> Option<&'static str> {
        Some("256+ kbps")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("Most popular first")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("PAUSED")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("LIVE NOW")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Tune out")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Search by name, country, language or genre")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Internet radio")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("just now")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Recently added")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Albums you add to the library appear here.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Recently played")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Albums you play appear here.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Search albums, songs, playlists and stations")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Search for an album, a song, a playlist or a station. Filters: artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("Stations")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("Songs")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("Music")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("Remove")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("Rescan")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Holds:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Library")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("No folders yet.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Rescan all")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("Processing")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("Kind not set")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("Choose what each of these folders holds.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("TV Shows")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("Coming soon")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("Add a folder")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some("Each folder holds one kind of media. Other kinds of file in it are skipped.")
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("TV Shows · Soon")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("Choose folder…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("Choose a folder for your library")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("Appearance")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("Theme")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("Dark")
    }
    fn light(&self) -> Option<&'static str> {
        Some("Light")
    }
    fn system(&self) -> Option<&'static str> {
        Some("System")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("Playback")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("Loudness")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "Levels tracks to the same loudness. Files with ReplayGain tags use them; other files are measured in the background as they play, and keep their own level until then.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("Album")
    }
    fn track(&self) -> Option<&'static str> {
        Some("Track")
    }
    fn off(&self) -> Option<&'static str> {
        Some("Off")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("Crossfade")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("Each track fades into the next over this many seconds. Zero turns it off.")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Look up synced lyrics on lrclib.net")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "For tracks with no .lrc file beside them. Sends each track's title, artist, album and length to lrclib.net. Lyrics found are cached in the library. A .lrc file always takes priority.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Movie details")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "Descriptions and cast for movies come from TMDb. Enter your own TMDb API key to use them. The key is saved in your settings file, readable only by your user account, and is sent only to TMDb.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("TMDb API key")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Save key")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Remove key")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("A key is saved.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some("No key saved. Movies show no descriptions or cast until you add one.")
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("This product uses the TMDB API but is not endorsed or certified by TMDB.")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("Nothing playing.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("This track has no lyrics in its tags or a .lrc file beside it.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("Looking up lyrics on lrclib.net…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("No lyrics were found on lrclib.net for this track.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Try again")
    }
    fn title(&self) -> Option<&'static str> {
        Some("Title")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("Artist")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("Duration")
    }
    fn size(&self) -> Option<&'static str> {
        Some("Size")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("Bitrate")
    }
    fn format(&self) -> Option<&'static str> {
        Some("Format")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("Channels")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("Pictures")
    }
    fn file(&self) -> Option<&'static str> {
        Some("File")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Reveal in Finder")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Tags")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Revert")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Put back the tags the file had before it was first edited")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "Look up synced lyrics on lrclib.net? The title, artist, album and length of each track you open are sent there.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Turn on lyrics lookup")
    }
    // Templates: `{0}`, `{1}` are replaced by values, so a translation can reorder them.
    fn photos_credits(&self) -> Option<&'static str> {
        Some("Photos: {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} h {1} min")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} min")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("Resume at {0}")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("Could not fetch details: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("{0} fps average, variable")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("Video needs libmpv: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("No audio output: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("Could not play the station: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("Could not connect to the station: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("Could not switch output: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("Could not play the video: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("Playback stopped early: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("Could not play {0}: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("{0} stations in the directory · synced {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0} tuned in")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  Tuned in")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("Bitrate: {0}+ kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("never")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("{0} minutes ago")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("{0} hours ago")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("{0} days ago")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("No results for \"{0}\".")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("Couldn't reach lrclib.net: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("Radio station sync failed: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: could not be scanned ({1}).")
    }
    // Counts, with the singular for one.
    fn one_movie(&self) -> Option<&'static str> {
        Some("1 movie")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} movies")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("1 album")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} albums")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("1 track")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} tracks")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("1 folder")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} folders")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("1 file")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} files")
    }
}
