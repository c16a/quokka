# Quokka

A music and movie player for macOS and Linux, written in Rust. It keeps an index of your
library in a local database, plays audio and video, and shows album, artist,
movie, playlist and internet radio views.

Quokka runs on Apple Silicon Macs and on Linux. Linux is built and tested in CI on x86_64
and arm64, and audio and video playback have been checked on Debian 13 (arm64). Windows is
not supported yet.

## Features

- Library index of audio and video files, with full-text search
- Audio playback with gapless playback, crossfade, ReplayGain and playback speed
- Video playback for H.264, HEVC, AV1 and ProRes. On macOS, the Mac's own decoders are used where
  they are available. On Linux, H.264 and HEVC use FFmpeg, built into the binary, and AV1 and VP9
  use bundled decoders. ProRes is macOS only.
- Subtitles from Matroska and MP4 files, and `.srt` sidecar files
- Audio track choice, frame stepping, screenshots and deinterlacing
- Movie details from TMDb, using your own API key
- Internet radio stations
- System media keys and the Now Playing display (macOS only for now)

## Platform support

**Windows is not supported yet.** Windows support is in progress. The H.264 and HEVC decoder for
Windows is written and compiles for the MSVC toolchain, and CI is set up to build and test on
Windows 11 ARM and Windows Server 2025. None of it has run on a Windows machine yet, so there is
no tested Windows build and no Windows playback. On Windows, HEVC needs the HEVC Video Extensions
from the Microsoft Store.

Each cell shows what has been tested on that platform. "Tested" means the feature was used in the
app on that platform. "CI" means only the automated build and tests have run there. "Not tested"
means it has not been checked. The macOS column reflects the developer's use of the app; the Linux
columns are from manual testing on the platforms named.

| Feature | macOS (Apple Silicon) | Linux (arm64, Debian 13) | Linux (x86_64) |
| --- | --- | --- | --- |
| Library index and search | Tested | Tested (folders scanned) | CI |
| Audio playback | Tested | Tested | CI |
| Gapless, crossfade, ReplayGain, playback speed | Tested | Not tested | Not tested |
| Video, H.264 | Tested | Tested | CI |
| Video, HEVC | Tested | Not tested | CI |
| Video, AV1 and VP9 (bundled decoders) | Tested | Not tested | CI |
| Video, ProRes | Tested | Not applicable | Not applicable |
| Playback controls in the video pane | Tested | Not tested | Not tested |
| Subtitles, audio track choice, frame stepping, screenshots, deinterlacing | Tested | Not tested | Not tested |
| Movie details from TMDb | Tested | Not tested | Not tested |
| Internet radio | Tested | Not tested | Not tested |
| System media keys and Now Playing | Tested | Not supported yet | Not supported yet |

## Requirements

- macOS on Apple Silicon, or Linux on x86_64 or arm64
- [Rust](https://rustup.rs). The toolchain version is pinned in `rust-toolchain.toml`,
  and rustup installs it automatically.
- [just](https://github.com/casey/just): `brew install just` on macOS, or your distribution's package.

On Linux, install the build tools and libraries. On Debian or Ubuntu:

```bash
sudo apt install just git nasm clang libclang-dev pkg-config libasound2-dev libdbus-1-dev \
  libssl-dev libwayland-dev libxkbcommon-dev libx11-dev libxcursor-dev libxi-dev \
  libxrandr-dev libgl-dev libegl-dev
```

FFmpeg 8.1 is built from source the first time you build, so the first build takes a while and
needs network access to GitHub. Later builds reuse it.

Quokka does not need libmpv.

## Building and running

```bash
just build        # build the workspace
just run          # run the app from the source tree
just check        # formatting, clippy and tests, as used before a change is committed
just release      # build an optimised binary, with FFmpeg built in
just release-system   # build an optimised binary against a system FFmpeg 8.1 instead
just bundle-release   # macOS only: build a release app at ~/Applications/Quokka.app
```

`just release-system` needs the FFmpeg 8.1 development files. See
[docs/relinking.md](docs/relinking.md).

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
fonts and their licences are listed in [NOTICE](NOTICE). The Linux binary contains FFmpeg 8.1,
which is licensed under the LGPL-2.1 or later. See [NOTICE](NOTICE), the licence text in
[LICENSES/LGPL-2.1.txt](LICENSES/LGPL-2.1.txt), and [docs/relinking.md](docs/relinking.md) for
how to use another FFmpeg. The licences of the Rust dependencies are listed in
[THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).
