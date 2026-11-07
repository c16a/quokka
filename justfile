# Default: run the full local check.
default: check

# Fail early, with the install command, when libmpv is missing. The player links against it.
# Install it with `brew install mpv`. The linker search path for Homebrew is in .cargo/config.toml.
libmpv-check:
    @if [ "$(uname)" = "Darwin" ]; then exit 0; fi; pkg-config --exists mpv || { echo "libmpv not found. Install it with: brew install mpv"; exit 1; }

# Build the whole workspace.
build: libmpv-check
    cargo build --workspace

# Build optimised.
release: libmpv-check
    cargo build --workspace --release

# Run all tests.
test: libmpv-check
    cargo test --workspace

# Play real video through libmpv: `just test-video ~/Movies/clip.mkv`. Runs the checks that need a
# file, which are skipped by `just test`. Use an AV1 file to exercise the new backend.
test-video path: libmpv-check
    PLAYER_TEST_VIDEO={{path}} cargo test -p player-playback --test mpv_player

# Format the code in place.
fmt:
    cargo fmt --all

# Verify formatting, lints (warnings are errors) and tests. Used before commits and in CI.
check: libmpv-check
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# Launch the desktop app.
run: libmpv-check
    cargo run -p player-app

# Package the debug build as ~/Applications/Quokka.app, so macOS and the computer-use tools can find it
# by name. The bundle is ad-hoc signed, for this Mac only. The Info.plist is in packaging/.
bundle: build
    APP="$HOME/Applications/Quokka.app"; rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS"; cp target/debug/player "$APP/Contents/MacOS/player"; cp packaging/Info.plist "$APP/Contents/Info.plist"; codesign --force --sign - "$APP"; echo "Built $APP"

bundle-release: release
    APP="$HOME/Applications/Quokka.app"; rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS"; cp target/release/player "$APP/Contents/MacOS/player"; cp packaging/Info.plist "$APP/Contents/Info.plist"; codesign --force --sign - "$APP"; echo "Built $APP"


# Scan a directory into a library database: `just scan ./library.db ~/Music`.
scan db root:
    cargo run -p player-core --example scan -- {{db}} {{root}}

# Match each movie in a library database to an IMDb title file, and print the best candidates:
# `just match-titles ./library.db ~/Downloads/title.basics.tsv.gz`.
match-titles db titles:
    cargo run -p player-core --example match -- {{db}} {{titles}}

# Match each movie in a library database to TMDb and print the details, for checking by eye.
# Needs a TMDb API key: `TMDB_API_KEY=... just match-tmdb ./library.db`.
match-tmdb db:
    cargo run -p player-core --example match_tmdb -- {{db}}

# Remove build artefacts.
clean:
    cargo clean

# Cross-compile a release build with `cargo zigbuild` (zig as the linker).
# Example: `just cross x86_64-unknown-linux-gnu`. Target must be installed via `rustup target add`.
cross target:
    cargo zigbuild --release --workspace --target {{target}}

# Cross-compile for every supported target (Apple Silicon macOS, not Intel).
cross-all:
    just cross x86_64-pc-windows-gnu
    just cross aarch64-apple-darwin

# Decode the first video track of MP4 files with the platform decoder and report what it accepts:
# `just decode-check ~/Movies/clip.mp4 core/tests/fixtures/video/h264_8_420.mp4`.
decode-check *files:
    cargo run -q -p player-playback --example decode_check -- {{files}}

# Measure how fast a file's frames decode, on an optimised build: `just frame-bench clip.mkv 10`
# decodes for ten seconds and reports the frame rate.
frame-bench file seconds:
    cargo run -q --release -p player-playback --example frame_bench -- {{file}} {{seconds}}
