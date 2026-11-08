//! Localised text for the UI. Each supported language is a file named by its language tag, as
//! [`detect`] reports it (`en-US.rs`), that implements [`Translation`]. A language returns `None` for
//! a string it has no translation for, and the UI shows the American English text instead.
//!
//! The UI reads a string through the free function of the same name, such as [`play`], which
//! resolves it in the current language. That is the operating system's preferred language until
//! the user picks one in Settings, which [`set_language`] applies.

#[path = "de-DE.rs"]
mod de_de;
mod detect;
#[path = "en-US.rs"]
mod en_us;
#[path = "es-ES.rs"]
mod es_es;
#[path = "fr-FR.rs"]
mod fr_fr;
#[path = "it-IT.rs"]
mod it_it;
#[path = "ja-JP.rs"]
mod ja_jp;
#[path = "ko-KR.rs"]
mod ko_kr;
#[path = "nl-NL.rs"]
mod nl_nl;
#[path = "pt-BR.rs"]
mod pt_br;
#[path = "pt-PT.rs"]
mod pt_pt;
#[path = "zh-Hans.rs"]
mod zh_hans;
#[path = "zh-Hant.rs"]
mod zh_hant;

use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

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

            /// The language's name in itself, for the language menu: `Deutsch`, `日本語`.
            fn native_name(&self) -> &'static str;

            /// Whether `count` takes the singular form, as in `{0} album`. English uses it for one
            /// alone; French uses it for zero too. Languages without plurals use the same text in both.
            fn is_singular(&self, count: usize) -> bool {
                count == 1
            }

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
    could_not_save_settings,
    radio_station_count,
    scanning_folder,
    scanning_folder_queued,
    scan_summary,
    matched_on_tmdb,
    tmdb_lookup_stopped,
    could_not_find_album,
    could_not_tune_in,
    could_not_tune_out,
    could_not_add_to_playlist,
    already_in_library,
    removed_folder,
    could_not_remove_folder,
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
    language,
    language_system,
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

/// Every translation the UI ships, in the order the language menu lists them. When a user's
/// language matches two only by its base language, the first wins.
const LANGUAGES: &[&dyn Translation] = &[
    &en_us::EnUs,
    &de_de::DeDe,
    &es_es::EsEs,
    &fr_fr::FrFr,
    &it_it::ItIt,
    &nl_nl::NlNl,
    &pt_br::PtBr,
    &pt_pt::PtPt,
    &ja_jp::JaJp,
    &ko_kr::KoKr,
    &zh_hans::ZhHans,
    &zh_hant::ZhHant,
];

/// The index into [`LANGUAGES`] of the current translation, or [`UNSET`] before the first string
/// is read.
static CURRENT: AtomicUsize = AtomicUsize::new(UNSET);
const UNSET: usize = usize::MAX;

/// The current translation. Until [`set_language`] is called, the system's.
fn current() -> &'static dyn Translation {
    let mut index = CURRENT.load(Ordering::Relaxed);
    if index == UNSET {
        index = system_index();
        CURRENT.store(index, Ordering::Relaxed);
    }
    LANGUAGES[index]
}

/// The index of the translation for the operating system's preferred languages, read once.
fn system_index() -> usize {
    static SYSTEM: OnceLock<usize> = OnceLock::new();
    *SYSTEM.get_or_init(|| select(&detect::preferred_languages()))
}

/// Show the UI in the language `tag` names, as saved from the language menu, or in the system's
/// language when `tag` is `None`. A tag no translation serves falls back as a system language does.
pub fn set_language(tag: Option<&str>) {
    let index = match tag {
        Some(tag) => select(&[tag.to_owned()]),
        None => system_index(),
    };
    CURRENT.store(index, Ordering::Relaxed);
}

/// The tag of the current translation, such as `ja-JP`, for choosing fonts that suit it.
pub fn current_tag() -> &'static str {
    current().language_tags()[0]
}

/// The name of the language the system prefers, in that language, for the menu's System option.
pub fn system_language_name() -> &'static str {
    LANGUAGES[system_index()].native_name()
}

/// Every language the UI ships, as the tag saved in settings and the name the menu shows.
pub fn languages() -> impl Iterator<Item = (&'static str, &'static str)> {
    LANGUAGES
        .iter()
        .map(|language| (language.language_tags()[0], language.native_name()))
}

/// The index of the translation for the first of `preferred` that one serves, by BCP 47 lookup:
/// `zh-Hant-TW` tries `zh-Hant-TW`, `zh-TW`, `zh-Hant`, then any translation of `zh`. English when
/// none matches.
fn select(preferred: &[String]) -> usize {
    let serves = |tag: &str| {
        LANGUAGES.iter().position(|l| {
            l.language_tags()
                .iter()
                .any(|served| served.eq_ignore_ascii_case(tag))
        })
    };
    for tag in preferred {
        if let Some(index) = lookup_candidates(tag).iter().find_map(|c| serves(c)) {
            return index;
        }
        let language = primary(tag);
        let same_language = LANGUAGES.iter().position(|l| {
            l.language_tags()
                .iter()
                .any(|served| primary(served).eq_ignore_ascii_case(language))
        });
        if let Some(index) = same_language {
            return index;
        }
    }
    0
}

/// The tags to try for `tag`, most specific first: the tag, the language with its region but no
/// script, then the tag with its last subtag removed, again and again.
fn lookup_candidates(tag: &str) -> Vec<String> {
    let parts: Vec<&str> = tag.split('-').collect();
    let mut candidates = vec![tag.to_owned()];
    // A script is four letters: `zh-Hant-TW` also tries `zh-TW`, as Windows and Linux name it.
    if parts.len() >= 3 && parts[1].len() == 4 {
        candidates.push(format!("{}-{}", parts[0], parts[2]));
    }
    for end in (1..parts.len()).rev() {
        candidates.push(parts[..end].join("-"));
    }
    candidates
}

/// The language subtag of a tag: `en` of `en-US`.
fn primary(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// A message to show later, such as a status line or an error, kept as the strings and values it is
/// made from rather than as finished text. It is put together each time it is shown, so it follows
/// a change of language made while it is on screen.
#[derive(Clone)]
pub struct Text(Arc<dyn Fn() -> String + Send + Sync>);

impl Text {
    /// One of the UI's strings, such as `Text::of(i18n::scanning)`.
    pub fn of(string: fn() -> &'static str) -> Self {
        Self(Arc::new(move || string().to_owned()))
    }

    /// A template filled with `values`, which may themselves be translated, as [`fill`] does.
    pub fn filled(template: fn() -> &'static str, values: Vec<Text>) -> Self {
        Self(Arc::new(move || {
            let values: Vec<String> = values.iter().map(Text::to_string).collect();
            let values: Vec<&str> = values.iter().map(String::as_str).collect();
            fill(template(), &values)
        }))
    }

    /// Text that is the same in every language, such as a path or an error from the system.
    pub fn plain(text: impl Into<String>) -> Self {
        let text = text.into();
        Self(Arc::new(move || text.clone()))
    }

    /// `texts` one after another, separated by `separator`.
    pub fn joined(texts: Vec<Text>, separator: &'static str) -> Self {
        Self(Arc::new(move || {
            texts
                .iter()
                .map(Text::to_string)
                .collect::<Vec<_>>()
                .join(separator)
        }))
    }

    /// Text made by `render`, for anything the other constructors don't cover.
    pub fn new(render: impl Fn() -> String + Send + Sync + 'static) -> Self {
        Self(Arc::new(render))
    }

    /// No text.
    pub fn empty() -> Self {
        Self::plain(String::new())
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&(self.0)())
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Text({:?})", self.to_string())
    }
}

/// Two texts are equal when they are the same message, not merely the same words.
impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Text {}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self::plain(text)
    }
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
    let template = if current().is_singular(count) {
        one
    } else {
        many
    };
    fill(template, &[&count.to_string()])
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

    fn tag(index: usize) -> &'static str {
        LANGUAGES[index].language_tags()[0]
    }

    fn pick(preferred: &[&str]) -> &'static str {
        let preferred: Vec<String> = preferred.iter().map(|t| (*t).to_owned()).collect();
        tag(select(&preferred))
    }

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
    fn translations_keep_every_placeholder() {
        for language in LANGUAGES {
            for ((name, english), (_, text)) in all(&en_us::EnUs).into_iter().zip(all(*language)) {
                let (Some(english), Some(text)) = (english, text) else {
                    continue;
                };
                for index in 0..4 {
                    let placeholder = format!("{{{index}}}");
                    assert_eq!(
                        english.contains(&placeholder),
                        text.contains(&placeholder),
                        "{} {name} has a different {placeholder}",
                        language.language_tags()[0],
                    );
                }
            }
        }
    }

    #[test]
    fn tags_are_unique() {
        let mut seen = Vec::new();
        for language in LANGUAGES {
            for served in language.language_tags() {
                assert!(!seen.contains(served), "{served} is served twice");
                seen.push(served);
            }
        }
    }

    #[test]
    fn falls_back_to_english() {
        assert_eq!(pick(&[]), "en-US");
        assert_eq!(pick(&["xx-YY"]), "en-US");
    }

    #[test]
    fn takes_the_first_language_served() {
        assert_eq!(pick(&["xx-YY", "fr-CA", "de-DE"]), "fr-FR");
    }

    #[test]
    fn matches_another_region_of_a_language() {
        assert_eq!(pick(&["en-GB"]), "en-US");
        assert_eq!(pick(&["nl-BE"]), "nl-NL");
        assert_eq!(pick(&["es-419"]), "es-ES");
    }

    #[test]
    fn tells_chinese_scripts_apart() {
        assert_eq!(pick(&["zh-Hans-CN"]), "zh-Hans");
        assert_eq!(pick(&["zh-Hant-TW"]), "zh-Hant");
        assert_eq!(pick(&["zh-TW"]), "zh-Hant");
        assert_eq!(pick(&["zh-HK"]), "zh-Hant");
        assert_eq!(pick(&["zh-CN"]), "zh-Hans");
        assert_eq!(pick(&["zh"]), "zh-Hans");
    }

    #[test]
    fn tells_portuguese_regions_apart() {
        assert_eq!(pick(&["pt-BR"]), "pt-BR");
        assert_eq!(pick(&["pt-PT"]), "pt-PT");
        assert_eq!(pick(&["pt-AO"]), "pt-PT");
        assert_eq!(pick(&["pt"]), "pt-BR");
    }

    #[test]
    fn fills_templates_in_any_order() {
        assert_eq!(fill("{1} then {0}", &["a", "b"]), "b then a");
    }

    #[test]
    fn text_follows_a_change_of_language() {
        use std::sync::atomic::AtomicBool;

        static GERMAN: AtomicBool = AtomicBool::new(false);
        fn greeting() -> &'static str {
            if GERMAN.load(Ordering::Relaxed) {
                "Hallo {0}"
            } else {
                "Hello {0}"
            }
        }
        // Made while the language is English, shown after it changes.
        let text = Text::filled(greeting, vec![Text::plain("Ada")]);
        assert_eq!(text.to_string(), "Hello Ada");
        GERMAN.store(true, Ordering::Relaxed);
        assert_eq!(text.to_string(), "Hallo Ada");
    }

    #[test]
    fn french_counts_zero_as_singular() {
        assert!(fr_fr::FrFr.is_singular(0));
        assert!(!en_us::EnUs.is_singular(0));
        assert!(!fr_fr::FrFr.is_singular(2));
    }
}
