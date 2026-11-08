//! Every string the user reads in the UI, in one place. Each constant is the text shown on
//! screen. Strings are grouped by the file that first shows them.

// albums.rs
pub const NAME: &str = "Name";
pub const YEAR: &str = "Year";
pub const DATE_ADDED: &str = "Date added";
pub const FLAC: &str = "FLAC";
pub const ALAC: &str = "ALAC";
pub const WAV: &str = "WAV";
pub const AIFF: &str = "AIFF";
pub const APE: &str = "APE";
pub const WAVPACK: &str = "WAVPACK";
pub const PCM: &str = "PCM";
pub const ALBUMS: &str = "Albums";
pub const NO_ALBUMS_MATCH_THESE_FILTERS: &str = "No albums match these filters.";
pub const NO_ALBUMS_YET_CHOOSE_YOUR_LIBRARY: &str =
    "No albums yet. Choose your library folder in Settings.";
pub const ALL_GENRES: &str = "All genres";
pub const ALL_YEARS: &str = "All years";
pub const ALL_ALBUMS: &str = "All albums";
pub const UNKNOWN_ALBUM: &str = "Unknown album";
pub const PLAY: &str = "Play";
pub const SHUFFLE: &str = "Shuffle";
pub const REMOVE_FROM_FAVOURITES: &str = "Remove from favourites";
pub const ADD_TO_FAVOURITES: &str = "Add to favourites";
pub const ALBUM: &str = "ALBUM";

// artists.rs
pub const NO_ARTISTS_YET_CHOOSE_YOUR_LIBRARY: &str =
    "No artists yet. Choose your library folder in Settings.";
pub const ARTISTS: &str = "Artists";
pub const ALL_ARTISTS: &str = "All artists";

// format.rs
pub const MONO: &str = "Mono";
pub const STEREO: &str = "Stereo";

// lib.rs
pub const ADD_A_FOLDER_TO_GET_STARTED: &str = "Add a folder to get started.";
pub const SYNCING_THE_RADIO_STATION_LIST: &str = "Syncing the radio station list…";
pub const RADIO_STATION_SYNC_STOPPED_UNEXPECTEDLY: &str =
    "Radio station sync stopped unexpectedly.";
pub const THE_SCAN_STOPPED_UNEXPECTEDLY: &str = "the scan stopped unexpectedly";
pub const VIDEO: &str = "Video";
pub const SEARCH: &str = "Search";
pub const RECENT: &str = "Recent";
pub const PLAYLISTS: &str = "Playlists";
pub const RADIO: &str = "Radio";
pub const MOVIES: &str = "Movies";
pub const SETTINGS: &str = "Settings";
pub const SCANNING: &str = "Scanning…";
pub const SAVED_THE_TMDB_KEY: &str = "Saved the TMDb key.";
pub const HIDE_SIDEBAR: &str = "Hide sidebar";
pub const SHOW_SIDEBAR: &str = "Show sidebar";

// movies.rs
pub const NO_MOVIES_YET_CHOOSE_YOUR_LIBRARY: &str =
    "No movies yet. Choose your library folder in Settings.";
pub const MOVIE: &str = "MOVIE";
pub const LENGTH_UNKNOWN: &str = "Length unknown";
pub const START_OVER: &str = "Start over";
pub const WRONG_MOVIE_CHOOSE_ANOTHER: &str = "Wrong movie? Choose another";
pub const REMOVE_DETAILS: &str = "Remove details";
pub const FILE_INFORMATION: &str = "File information";
pub const SOURCE: &str = "SOURCE";
pub const FINDING_DETAILS: &str = "Finding details…";
pub const FIND_DETAILS_ON_TMDB: &str = "Find details on TMDb";
pub const ADD_YOUR_TMDB_API_KEY_IN: &str =
    "Add your TMDb API key in Settings to see the description and cast.";
pub const SYNOPSIS: &str = "Synopsis";
pub const CAST: &str = "Cast";
pub const DIRECTOR: &str = "Director";
pub const WRITER: &str = "Writer";
pub const CREW: &str = "Crew";
pub const WHICH_MOVIE_IS_THIS: &str = "Which movie is this?";
pub const PICK_THE_ONE_YOUR_FILE_IS: &str =
    "Pick the one your file is. Nothing is saved until you pick.";
pub const NONE_OF_THESE: &str = "None of these";
pub const LEFT_UNMATCHED: &str = "Left unmatched.";
pub const THIS_ONE: &str = "This one";
pub const THE_LOOKUP_STOPPED_UNEXPECTEDLY: &str = "the lookup stopped unexpectedly";
pub const NO_TMDB_DETAILS_FOR_THIS_MOVIE: &str = "No TMDb details for this movie.";
pub const UNKNOWN: &str = "Unknown";
pub const CONTAINER: &str = "Container";
pub const CODEC: &str = "Codec";
pub const COLOUR_DEPTH: &str = "Colour depth";
pub const CHROMA_SUBSAMPLING: &str = "Chroma subsampling";
pub const DYNAMIC_RANGE: &str = "Dynamic range";
pub const HDR: &str = "HDR";
pub const SDR: &str = "SDR";
pub const RESOLUTION: &str = "Resolution";
pub const ASPECT_RATIO: &str = "Aspect ratio";
pub const FRAME_RATE: &str = "Frame rate";
pub const LENGTH: &str = "Length";
pub const FILE_SIZE: &str = "File size";
pub const MP4: &str = "MP4";
pub const M4V: &str = "M4V";
pub const MATROSKA_MKV: &str = "Matroska (MKV)";
pub const WEBM: &str = "WebM";
pub const QUICKTIME_MOV: &str = "QuickTime (MOV)";
pub const AVI: &str = "AVI";
pub const MPEG_TRANSPORT_STREAM: &str = "MPEG transport stream";
pub const MPEG_PROGRAM_STREAM: &str = "MPEG program stream";
pub const RAW_AV1_STREAM_OBU: &str = "Raw AV1 stream (OBU)";

// playback.rs
pub const NO_AUDIO_OUTPUT_DEVICE_AVAILABLE: &str = "no audio output device available";
pub const COULD_NOT_CONNECT_TO_THE_STATION: &str = "Could not connect to the station.";
pub const THE_STATION_KEEPS_DROPPING_THE_CONNECTION: &str =
    "The station keeps dropping the connection. Press play to try again.";
pub const PAUSE: &str = "Pause";
pub const CONNECTING: &str = "Connecting…";
pub const PAUSED: &str = "Paused";
pub const LIVE: &str = "Live";
pub const NO_AUDIO_OUTPUT_DEVICE_AVAILABLE_2: &str = "No audio output device available";
pub const BACK_10_SECONDS_LEFT_ARROW: &str = "Back 10 seconds (Left arrow)";
pub const PAUSE_SPACE: &str = "Pause (Space)";
pub const PLAY_SPACE: &str = "Play (Space)";
pub const FORWARD_10_SECONDS_RIGHT_ARROW: &str = "Forward 10 seconds (Right arrow)";
pub const EXIT_FULL_SCREEN_ESCAPE: &str = "Exit full screen (Escape)";
pub const FULL_SCREEN_F: &str = "Full screen (F)";
pub const UNMUTE: &str = "Unmute";
pub const MUTE: &str = "Mute";
pub const LOADING_VIDEO: &str = "Loading video…";
pub const PREVIOUS_TRACK: &str = "Previous track";
pub const NEXT_TRACK: &str = "Next track";
pub const REPEAT: &str = "Repeat";
pub const NOTHING_PLAYING: &str = "Nothing playing";
pub const OUTPUT: &str = "Output";
pub const SYSTEM_DEFAULT: &str = "System default";
pub const UP_NEXT: &str = "Up next";
pub const LYRICS: &str = "Lyrics";
pub const PLAYBACK_SPEED_CLICK_TO_CHANGE_SPEEDS: &str =
    "Playback speed. Click to change. Speeds other than 1× also change the pitch.";
pub const TRACK_INFO: &str = "Track info";

// playlists.rs
pub const NEW_PLAYLIST_NAME: &str = "New playlist name";
pub const CREATE: &str = "Create";
pub const NO_PLAYLISTS_YET_ADD_TRACKS_FROM: &str =
    "No playlists yet. Add tracks from an album or the search results.";
pub const RENAME: &str = "Rename";
pub const DELETE_PLAYLIST: &str = "Delete playlist";
pub const ALL_PLAYLISTS: &str = "All playlists";
pub const SAVE: &str = "Save";
pub const CANCEL: &str = "Cancel";
pub const PLAY_PLAYLIST: &str = "Play playlist";
pub const SHUFFLE_PLAYLIST: &str = "Shuffle playlist";

// queue.rs
pub const UP_NEXT_2: &str = "Up Next";
pub const NOTHING_QUEUED: &str = "Nothing queued";
pub const CLEAR: &str = "Clear";
pub const TRACKS_YOU_QUEUE_WILL_APPEAR_HERE: &str = "Tracks you queue will appear here.";
pub const PLAY_NEXT: &str = "Play next";
pub const ADD_TO_QUEUE: &str = "Add to queue";
pub const ADD_TO_PLAYLIST: &str = "Add to playlist";
pub const NO_PLAYLISTS_YET: &str = "No playlists yet";

// radio.rs
pub const NO_STATIONS_YET: &str = "No stations yet.";
pub const SYNC_NOW: &str = "Sync now";
pub const SYNCING: &str = "Syncing…";
pub const YOUR_STATIONS: &str = "Your stations";
pub const FIND_A_STATION_BELOW_AND_TUNE: &str =
    "Find a station below and tune in. It will appear here.";
pub const FIND_STATIONS: &str = "Find stations";
pub const SYNC_THE_STATION_LIST_TO_SEARCH: &str = "Sync the station list to search it.";
pub const NO_STATIONS_MATCH: &str = "No stations match.";
pub const COUNTRY_ANY: &str = "Country: any";
pub const ANY_COUNTRY: &str = "Any country";
pub const CODEC_ANY: &str = "Codec: any";
pub const ANY_CODEC: &str = "Any codec";
pub const BITRATE_ANY: &str = "Bitrate: any";
pub const ANY_BITRATE: &str = "Any bitrate";
pub const TEXT_64_KBPS: &str = "64+ kbps";
pub const TEXT_128_KBPS: &str = "128+ kbps";
pub const TEXT_256_KBPS: &str = "256+ kbps";
pub const MOST_POPULAR_FIRST: &str = "Most popular first";
pub const PAUSED_2: &str = "PAUSED";
pub const LIVE_NOW: &str = "LIVE NOW";
pub const TUNE_OUT: &str = "Tune out";
pub const SEARCH_BY_NAME_COUNTRY_LANGUAGE_OR: &str = "Search by name, country, language or genre";
pub const INTERNET_RADIO: &str = "Internet radio";
pub const JUST_NOW: &str = "just now";

// recent.rs
pub const RECENTLY_ADDED: &str = "Recently added";
pub const ALBUMS_YOU_ADD_TO_THE_LIBRARY: &str = "Albums you add to the library appear here.";
pub const RECENTLY_PLAYED: &str = "Recently played";
pub const ALBUMS_YOU_PLAY_APPEAR_HERE: &str = "Albums you play appear here.";

// search.rs
pub const SEARCH_ALBUMS_SONGS_PLAYLISTS_AND_STATIONS: &str =
    "Search albums, songs, playlists and stations";
pub const SEARCH_FOR_AN_ALBUM_A_SONG: &str = "Search for an album, a song, a playlist or a station. Filters: artist:, album:, genre:, year:, format:flac.";
pub const STATIONS: &str = "Stations";
pub const SONGS: &str = "Songs";

// settings_view.rs
pub const MUSIC: &str = "Music";
pub const REMOVE: &str = "Remove";
pub const RESCAN: &str = "Rescan";
pub const HOLDS: &str = "Holds:";
pub const LIBRARY: &str = "Library";
pub const NO_FOLDERS_YET: &str = "No folders yet.";
pub const RESCAN_ALL: &str = "Rescan all";
pub const PROCESSING: &str = "Processing";
pub const KIND_NOT_SET: &str = "Kind not set";
pub const CHOOSE_WHAT_EACH_OF_THESE_FOLDERS: &str = "Choose what each of these folders holds.";
pub const TV_SHOWS: &str = "TV Shows";
pub const COMING_SOON: &str = "Coming soon";
pub const ADD_A_FOLDER: &str = "Add a folder";
pub const EACH_FOLDER_HOLDS_ONE_KIND_OF: &str =
    "Each folder holds one kind of media. Other kinds of file in it are skipped.";
pub const TV_SHOWS_SOON: &str = "TV Shows · Soon";
pub const CHOOSE_FOLDER: &str = "Choose folder…";
pub const CHOOSE_A_FOLDER_FOR_YOUR_LIBRARY: &str = "Choose a folder for your library";
pub const APPEARANCE: &str = "Appearance";
pub const THEME: &str = "Theme";
pub const DARK: &str = "Dark";
pub const LIGHT: &str = "Light";
pub const SYSTEM: &str = "System";
pub const PLAYBACK: &str = "Playback";
pub const LOUDNESS: &str = "Loudness";
pub const LEVELS_TRACKS_TO_THE_SAME_LOUDNESS: &str = "Levels tracks to the same loudness. Files with ReplayGain tags use them; other files are measured in the background as they play, and keep their own level until then.";
pub const ALBUM_2: &str = "Album";
pub const TRACK: &str = "Track";
pub const OFF: &str = "Off";
pub const CROSSFADE: &str = "Crossfade";
pub const EACH_TRACK_FADES_INTO_THE_NEXT: &str =
    "Each track fades into the next over this many seconds. Zero turns it off.";
pub const LOOK_UP_SYNCED_LYRICS_ON_LRCLIB: &str = "Look up synced lyrics on lrclib.net";
pub const FOR_TRACKS_WITH_NO_LRC_FILE: &str = "For tracks with no .lrc file beside them. Sends each track's title, artist, album and length to lrclib.net. Lyrics found are cached in the library. A .lrc file always takes priority.";
pub const MOVIE_DETAILS: &str = "Movie details";
pub const DESCRIPTIONS_AND_CAST_FOR_MOVIES_COME: &str = "Descriptions and cast for movies come from TMDb. Enter your own TMDb API key to use them. The key is saved in your settings file, readable only by your user account, and is sent only to TMDb.";
pub const TMDB_API_KEY: &str = "TMDb API key";
pub const SAVE_KEY: &str = "Save key";
pub const REMOVE_KEY: &str = "Remove key";
pub const A_KEY_IS_SAVED: &str = "A key is saved.";
pub const NO_KEY_SAVED_MOVIES_SHOW_NO: &str =
    "No key saved. Movies show no descriptions or cast until you add one.";
pub const THIS_PRODUCT_USES_THE_TMDB_API: &str =
    "This product uses the TMDB API but is not endorsed or certified by TMDB.";

// side.rs
pub const NOTHING_PLAYING_2: &str = "Nothing playing.";
pub const THIS_TRACK_HAS_NO_LYRICS_IN: &str =
    "This track has no lyrics in its tags or a .lrc file beside it.";
pub const LOOKING_UP_LYRICS_ON_LRCLIB_NET: &str = "Looking up lyrics on lrclib.net…";
pub const NO_LYRICS_WERE_FOUND_ON_LRCLIB: &str =
    "No lyrics were found on lrclib.net for this track.";
pub const TRY_AGAIN: &str = "Try again";
pub const TITLE: &str = "Title";
pub const ARTIST: &str = "Artist";
pub const DURATION: &str = "Duration";
pub const SIZE: &str = "Size";
pub const BITRATE: &str = "Bitrate";
pub const FORMAT: &str = "Format";
pub const CHANNELS: &str = "Channels";
pub const PICTURES: &str = "Pictures";
pub const FILE: &str = "File";
pub const REVEAL_IN_FINDER: &str = "Reveal in Finder";
pub const TAGS: &str = "Tags";
pub const REVERT: &str = "Revert";
pub const PUT_BACK_THE_TAGS_THE_FILE: &str =
    "Put back the tags the file had before it was first edited";
pub const LOOK_UP_SYNCED_LYRICS_ON_LRCLIB_2: &str = "Look up synced lyrics on lrclib.net? The title, artist, album and length of each track you open are sent there.";
pub const TURN_ON_LYRICS_LOOKUP: &str = "Turn on lyrics lookup";

// Templates: `{0}`, `{1}` are replaced by values, so a translation can reorder them.
pub const PHOTOS_CREDITS: &str = "Photos: {0}";
pub const DURATION_HOURS_MINUTES: &str = "{0} h {1} min";
pub const DURATION_MINUTES: &str = "{0} min";
pub const RESUME_AT: &str = "Resume at {0}";
pub const COULD_NOT_FETCH_DETAILS: &str = "Could not fetch details: {0}";
pub const FPS_AVERAGE_VARIABLE: &str = "{0} fps average, variable";
#[cfg(not(target_os = "macos"))]
pub const VIDEO_NEEDS_LIBMPV: &str = "Video needs libmpv: {0}";
pub const NO_AUDIO_OUTPUT_ERROR: &str = "No audio output: {0}";
pub const COULD_NOT_PLAY_STATION: &str = "Could not play the station: {0}";
pub const COULD_NOT_CONNECT_STATION_ERROR: &str = "Could not connect to the station: {0}";
pub const COULD_NOT_SWITCH_OUTPUT: &str = "Could not switch output: {0}";
pub const COULD_NOT_PLAY_VIDEO: &str = "Could not play the video: {0}";
pub const PLAYBACK_STOPPED_EARLY: &str = "Playback stopped early: {0}";
pub const COULD_NOT_PLAY_ITEM: &str = "Could not play {0}: {1}";
pub const STATIONS_IN_DIRECTORY: &str = "{0} stations in the directory · synced {1}";
pub const TUNED_IN: &str = "{0} tuned in";
pub const TUNED_IN_SUFFIX: &str = "  ·  Tuned in";
pub const BITRATE_AT_LEAST: &str = "Bitrate: {0}+ kbps";
pub const NEVER: &str = "never";
pub const MINUTES_AGO: &str = "{0} minutes ago";
pub const HOURS_AGO: &str = "{0} hours ago";
pub const DAYS_AGO: &str = "{0} days ago";
pub const NO_RESULTS_FOR: &str = "No results for \"{0}\".";
pub const COULDNT_REACH_LRCLIB: &str = "Couldn't reach lrclib.net: {0}";
pub const RADIO_SYNC_FAILED: &str = "Radio station sync failed: {0}";
pub const COULD_NOT_BE_SCANNED: &str = "{0}: could not be scanned ({1}).";

// Counts, with the singular for one.
pub const ONE_MOVIE: &str = "1 movie";
pub const MANY_MOVIES: &str = "{0} movies";
pub const ONE_ALBUM: &str = "1 album";
pub const MANY_ALBUMS: &str = "{0} albums";
pub const ONE_TRACK: &str = "1 track";
pub const MANY_TRACKS: &str = "{0} tracks";
pub const ONE_FOLDER: &str = "1 folder";
pub const MANY_FOLDERS: &str = "{0} folders";
pub const ONE_FILE: &str = "1 file";
pub const MANY_FILES: &str = "{0} files";

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

pub fn movies(count: usize) -> String {
    counted(count, ONE_MOVIE, MANY_MOVIES)
}

pub fn albums(count: usize) -> String {
    counted(count, ONE_ALBUM, MANY_ALBUMS)
}

pub fn tracks(count: usize) -> String {
    counted(count, ONE_TRACK, MANY_TRACKS)
}

pub fn folders(count: usize) -> String {
    counted(count, ONE_FOLDER, MANY_FOLDERS)
}

pub fn files(count: usize) -> String {
    counted(count, ONE_FILE, MANY_FILES)
}
