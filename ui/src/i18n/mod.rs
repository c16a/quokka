//! Localised text for the UI. Each supported language is a file named by its language tag, as
//! [`detect`] reports it (`en-US.rs`), that implements [`Translation`]. A language returns `None` for
//! a string it has no translation for, and the UI shows the American English text instead.
//!
//! The UI reads a string through the free function of the same name, such as [`play`], which
//! resolves it in the user's language. The language is chosen once, from the operating system's
//! preferred languages, the first time a string is read.

mod detect;
#[path = "en-US.rs"]
mod en_us;

use std::sync::OnceLock;

/// Declares the strings: a method on [`Translation`] for each one, and a free function that
/// resolves it in the current language with the English fallback.
macro_rules! strings {
    ($($(#[$meta:meta])* $name:ident,)*) => {
        /// The UI's text in one language. Every method but [`Translation::language_tags`] returns the
        /// translated string, or `None` when this language has none and English should be shown.
        /// Templates hold `{0}`, `{1}` placeholders that [`fill`] replaces, so a translation can
        /// reorder them.
        pub trait Translation: Sync {
            /// The BCP 47 tags this translation serves, most specific first, such as `["en-US", "en"]`.
            /// A bare language (`en`) matches any region of it the user prefers.
            fn language_tags(&self) -> &'static [&'static str];

            $(
                $(#[$meta])*
                #[cfg_attr(
                not(target_os = "macos"),
                allow(dead_code, reason = "the video window and its controls are macOS-only")
            )]
                fn $name(&self) -> Option<&'static str> {
                    None
                }
            )*
        }

        $(
            $(#[$meta])*
            #[cfg_attr(
                not(target_os = "macos"),
                allow(dead_code, reason = "the video window and its controls are macOS-only")
            )]
            pub fn $name() -> &'static str {
                current().$name().or_else(|| en_us::EnUs.$name()).unwrap_or_default()
            }
        )*

        /// Every string in `translation`, by name, for checking a translation is complete.
        #[cfg(test)]
        fn all(translation: &dyn Translation) -> Vec<(&'static str, Option<&'static str>)> {
            let mut strings = Vec::new();
            $(
                $(#[$meta])*
                strings.push((stringify!($name), translation.$name()));
            )*
            strings
        }
    };
}

strings! {
    // albums.rs
    name,
    year,
    date_added,
    albums,
    no_albums_match_these_filters,
    no_albums_yet_choose_your_library,
    all_genres,
    all_years,
    all_albums,
    unknown_album,
    play,
    shuffle,
    remove_from_favourites,
    add_to_favourites,
    album,
    // artists.rs
    no_artists_yet_choose_your_library,
    artists,
    all_artists,
    // format.rs
    mono,
    stereo,
    // lib.rs
    add_a_folder_to_get_started,
    syncing_the_radio_station_list,
    radio_station_sync_stopped_unexpectedly,
    the_scan_stopped_unexpectedly,
    video,
    search,
    recent,
    playlists,
    radio,
    movies,
    settings,
    scanning,
    saved_the_tmdb_key,
    hide_sidebar,
    show_sidebar,
    // movies.rs
    no_movies_yet_choose_your_library,
    movie,
    length_unknown,
    start_over,
    wrong_movie_choose_another,
    remove_details,
    file_information,
    source,
    finding_details,
    find_details_on_tmdb,
    add_your_tmdb_api_key_in,
    synopsis,
    cast,
    director,
    writer,
    crew,
    which_movie_is_this,
    pick_the_one_your_file_is,
    none_of_these,
    left_unmatched,
    this_one,
    the_lookup_stopped_unexpectedly,
    no_tmdb_details_for_this_movie,
    unknown,
    container,
    codec,
    colour_depth,
    chroma_subsampling,
    dynamic_range,
    hdr,
    sdr,
    resolution,
    aspect_ratio,
    frame_rate,
    length,
    file_size,
    mp4,
    m4v,
    matroska_mkv,
    webm,
    quicktime_mov,
    avi,
    mpeg_transport_stream,
    mpeg_program_stream,
    raw_av1_stream_obu,
    // playback.rs
    no_audio_output_device_available,
    could_not_connect_to_the_station,
    the_station_keeps_dropping_the_connection,
    pause,
    connecting,
    paused,
    live,
    no_audio_output_device_available_2,
    back_10_seconds_left_arrow,
    pause_space,
    play_space,
    forward_10_seconds_right_arrow,
    exit_full_screen_escape,
    full_screen_f,
    unmute,
    mute,
    loading_video,
    previous_track,
    next_track,
    repeat,
    nothing_playing,
    output,
    system_default,
    up_next,
    lyrics,
    playback_speed_click_to_change_speeds,
    track_info,
    // playlists.rs
    new_playlist_name,
    create,
    no_playlists_yet_add_tracks_from,
    rename,
    delete_playlist,
    all_playlists,
    save,
    cancel,
    play_playlist,
    shuffle_playlist,
    // queue.rs
    up_next_2,
    nothing_queued,
    clear,
    tracks_you_queue_will_appear_here,
    play_next,
    add_to_queue,
    add_to_playlist,
    no_playlists_yet,
    // radio.rs
    no_stations_yet,
    sync_now,
    syncing,
    your_stations,
    find_a_station_below_and_tune,
    find_stations,
    sync_the_station_list_to_search,
    no_stations_match,
    country_any,
    any_country,
    codec_any,
    any_codec,
    bitrate_any,
    any_bitrate,
    text_64_kbps,
    text_128_kbps,
    text_256_kbps,
    most_popular_first,
    paused_2,
    live_now,
    tune_out,
    search_by_name_country_language_or,
    internet_radio,
    just_now,
    // recent.rs
    recently_added,
    albums_you_add_to_the_library,
    recently_played,
    albums_you_play_appear_here,
    // search.rs
    search_albums_songs_playlists_and_stations,
    search_for_an_album_a_song,
    stations,
    songs,
    // settings_view.rs
    music,
    remove,
    rescan,
    holds,
    library,
    no_folders_yet,
    rescan_all,
    processing,
    kind_not_set,
    choose_what_each_of_these_folders,
    tv_shows,
    coming_soon,
    add_a_folder,
    each_folder_holds_one_kind_of,
    tv_shows_soon,
    choose_folder,
    choose_a_folder_for_your_library,
    appearance,
    theme,
    dark,
    light,
    system,
    playback,
    loudness,
    levels_tracks_to_the_same_loudness,
    album_2,
    track,
    off,
    crossfade,
    each_track_fades_into_the_next,
    look_up_synced_lyrics_on_lrclib,
    for_tracks_with_no_lrc_file,
    movie_details,
    descriptions_and_cast_for_movies_come,
    tmdb_api_key,
    save_key,
    remove_key,
    a_key_is_saved,
    no_key_saved_movies_show_no,
    this_product_uses_the_tmdb_api,
    // side.rs
    nothing_playing_2,
    this_track_has_no_lyrics_in,
    looking_up_lyrics_on_lrclib_net,
    no_lyrics_were_found_on_lrclib,
    try_again,
    title,
    artist,
    duration,
    size,
    bitrate,
    format,
    channels,
    pictures,
    file,
    reveal_in_finder,
    tags,
    revert,
    put_back_the_tags_the_file,
    look_up_synced_lyrics_on_lrclib_2,
    turn_on_lyrics_lookup,
    // Templates: `{0}`, `{1}` are replaced by values, so a translation can reorder them.
    photos_credits,
    duration_hours_minutes,
    duration_minutes,
    resume_at,
    could_not_fetch_details,
    fps_average_variable,
    #[cfg(not(target_os = "macos"))]
    video_needs_libmpv,
    no_audio_output_error,
    could_not_play_station,
    could_not_connect_station_error,
    could_not_switch_output,
    could_not_play_video,
    playback_stopped_early,
    could_not_play_item,
    stations_in_directory,
    tuned_in,
    tuned_in_suffix,
    bitrate_at_least,
    never,
    minutes_ago,
    hours_ago,
    days_ago,
    no_results_for,
    couldnt_reach_lrclib,
    radio_sync_failed,
    could_not_be_scanned,
    // Counts, with the singular for one.
    one_movie,
    many_movies,
    one_album,
    many_albums,
    one_track,
    many_tracks,
    one_folder,
    many_folders,
    one_file,
    many_files,
}

/// Every translation the UI ships, in no particular order.
const LANGUAGES: &[&dyn Translation] = &[&en_us::EnUs];

/// The translation for the user's language, chosen on first use.
fn current() -> &'static dyn Translation {
    static CURRENT: OnceLock<&'static dyn Translation> = OnceLock::new();
    *CURRENT.get_or_init(|| select(&detect::preferred_languages()))
}

/// The translation for the first of `preferred` that one serves: a tag the translation lists, or
/// failing that the same language in another region. English when none matches.
fn select(preferred: &[String]) -> &'static dyn Translation {
    for tag in preferred {
        let exact = LANGUAGES.iter().find(|l| {
            l.language_tags()
                .iter()
                .any(|t| t.eq_ignore_ascii_case(tag))
        });
        let language = primary(tag);
        let same_language = || {
            LANGUAGES.iter().find(|l| {
                l.language_tags()
                    .iter()
                    .any(|t| primary(t).eq_ignore_ascii_case(language))
            })
        };
        if let Some(translation) = exact.or_else(same_language) {
            return *translation;
        }
    }
    &en_us::EnUs
}

/// The language subtag of a tag: `en` of `en-US`.
fn primary(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// `template` with each `{n}` replaced by the nth value.
pub fn fill(template: &str, values: &[&str]) -> String {
    let mut text = template.to_owned();
    for (index, value) in values.iter().enumerate() {
        text = text.replace(&format!("{{{index}}}"), value);
    }
    text
}

fn counted(count: usize, one: &str, many: &str) -> String {
    if count == 1 {
        one.to_owned()
    } else {
        fill(many, &[&count.to_string()])
    }
}

pub fn movie_count(count: usize) -> String {
    counted(count, one_movie(), many_movies())
}

pub fn album_count(count: usize) -> String {
    counted(count, one_album(), many_albums())
}

pub fn track_count(count: usize) -> String {
    counted(count, one_track(), many_tracks())
}

pub fn folder_count(count: usize) -> String {
    counted(count, one_folder(), many_folders())
}

pub fn file_count(count: usize) -> String {
    counted(count, one_file(), many_files())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_has_every_string() {
        let missing: Vec<_> = all(&en_us::EnUs)
            .into_iter()
            .filter(|(_, text)| text.is_none())
            .map(|(name, _)| name)
            .collect();
        assert!(missing.is_empty(), "en-US is missing {missing:?}");
    }

    #[test]
    fn falls_back_to_english() {
        assert_eq!(select(&[]).language_tags()[0], "en-US");
        assert_eq!(select(&["xx-YY".into()]).language_tags()[0], "en-US");
    }

    #[test]
    fn matches_another_region_of_a_language() {
        assert_eq!(select(&["en-GB".into()]).language_tags()[0], "en-US");
    }

    #[test]
    fn fills_templates_in_any_order() {
        assert_eq!(fill("{1} then {0}", &["a", "b"]), "b then a");
    }

    #[test]
    fn counts_use_the_singular_for_one() {
        assert_eq!(album_count(1), "1 album");
        assert_eq!(album_count(3), "3 albums");
    }
}
