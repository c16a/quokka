# Quokka

A music and movie player for macOS, written in Rust. It keeps an index of your
library in a local database, plays audio and video, and shows album, artist,
movie, playlist and internet radio views.

Quokka runs on macOS only for now. It was built and tested on Apple Silicon.

## Features

- Library index of audio and video files, with full-text search
- Audio playback with gapless playback, crossfade, ReplayGain and playback speed
- Video playback for H.264, HEVC, AV1 and ProRes, with the Mac's own decoders where they are available
- Subtitles from Matroska and MP4 files, and `.srt` sidecar files
- Audio track choice, frame stepping, screenshots and deinterlacing
- Movie details from TMDb, using your own API key
- Internet radio stations
- System media keys and the Now Playing display

## Requirements

- An Apple Silicon Mac
- [Rust](https://rustup.rs). The toolchain version is pinned in `rust-toolchain.toml`,
  and rustup installs it automatically.
- [just](https://github.com/casey/just): `brew install just`

Quokka does not need libmpv on macOS.

## Building and running

```bash
just build        # build the workspace
just run          # run the app from the source tree
just check        # formatting, clippy and tests, as used before a change is committed
just bundle-release   # build a release app at ~/Applications/Quokka.app
```

To index a folder from the command line:

```bash
just scan ./library.db ~/Music
```

## Optional online features

Quokka makes these network requests only when you turn them on, or use them:

- **TMDb (movie details and posters).** Add your API key in Settings. The command-line
  matcher reads the `TMDB_API_KEY` environment variable instead.
- **Synced lyrics from lrclib.net.** Off by default. When turned on, the title, artist,
  album and duration of each track you open are sent to lrclib.net.
- **Artist photos** come from Wikimedia Commons and Wikidata.
- **Internet radio** stations come from radio-browser.info.

## Project layout

- `core/`: the library, indexing, metadata, search and video container readers. It has no UI.
- `playback/`: the playback engine and the decoders.
- `ui/`: the windows, views and controls, built with egui.
- `app/`: the executable.

`AGENTS.md` in each crate describes its rules for contributors.

## Licence

Quokka is released under the MIT licence. See [LICENSE](LICENSE). Third-party
fonts and their licences are listed in [NOTICE](NOTICE).
