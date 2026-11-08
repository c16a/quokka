# Recipes run under PowerShell on Windows, where `sh` is not on the default path. The recipes that
# use shell syntax (bundle, icon) are macOS-only and are not run there.
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

# Default: run the full local check.
default: check

# Build the whole workspace.
build:
    cargo build --workspace

# Build optimised.
release:
    cargo build --workspace --release

# Build optimised, linking against a system FFmpeg 8.1 instead of building it in. See docs/relinking.md.
release-system:
    cargo build -p quokka-app --release --no-default-features

# Run all tests.
test:
    cargo test --workspace

# Format the code in place.
fmt:
    cargo fmt --all

# Verify formatting, lints (warnings are errors) and tests. Used before commits and in CI.
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# Launch the desktop app.
run:
    cargo run -p quokka-app

# Build the app icon, packaging/AppIcon.icns, from packaging/icon/quokka.svg. Needs ImageMagick
# (`brew install imagemagick`) and macOS's iconutil. Run it after the SVG changes, then commit the .icns.
icon:
    SET="$(mktemp -d)/AppIcon.iconset"; mkdir -p "$SET"; magick -background none -density 192 packaging/icon/quokka.svg -resize 1024x1024 "$SET/icon_512x512@2x.png"; for s in 16 32 128 256 512; do magick "$SET/icon_512x512@2x.png" -resize ${s}x${s} "$SET/icon_${s}x${s}.png"; magick "$SET/icon_512x512@2x.png" -resize $((s*2))x$((s*2)) "$SET/icon_${s}x${s}@2x.png"; done; iconutil -c icns "$SET" -o packaging/AppIcon.icns; echo "Wrote packaging/AppIcon.icns"

# Package the debug build as ~/Applications/Quokka.app, so macOS and the computer-use tools can find it
# by name. The bundle is ad-hoc signed, for this Mac only. The Info.plist and the icon are in packaging/.
bundle: build
    APP="$HOME/Applications/Quokka.app"; rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"; cp target/debug/quokka "$APP/Contents/MacOS/quokka"; cp packaging/Info.plist "$APP/Contents/Info.plist"; cp packaging/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"; codesign --force --sign - "$APP"; echo "Built $APP"

bundle-release: release
    APP="$HOME/Applications/Quokka.app"; rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"; cp target/release/quokka "$APP/Contents/MacOS/quokka"; cp packaging/Info.plist "$APP/Contents/Info.plist"; cp packaging/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"; codesign --force --sign - "$APP"; echo "Built $APP"


# Scan a directory into a library database: `just scan ./library.db ~/Music`.
scan db root:
    cargo run -p quokka-core --example scan -- {{db}} {{root}}

# Match each movie in a library database to an IMDb title file, and print the best candidates:
# `just match-titles ./library.db ~/Downloads/title.basics.tsv.gz`.
match-titles db titles:
    cargo run -p quokka-core --example match -- {{db}} {{titles}}

# Match each movie in a library database to TMDb and print the details, for checking by eye.
# Needs a TMDb API key: `TMDB_API_KEY=... just match-tmdb ./library.db`.
match-tmdb db:
    cargo run -p quokka-core --example match_tmdb -- {{db}}

# Fill a database with synthetic albums and movies, for timing: `just fixture bench.db 2000 10 500`.
fixture db albums tracks movies:
    cargo run -q -p quokka-core --example library_fixture -- {{db}} {{albums}} {{tracks}} {{movies}}

# Time the library queries the UI runs most, on an optimised build: `just bench-queries bench.db`.
bench-queries db:
    cargo run -q --release -p quokka-core --example bench_queries -- {{db}}

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
    cargo run -q -p quokka-playback --example decode_check -- {{files}}

# Measure how fast a file's frames decode, on an optimised build: `just frame-bench clip.mkv 10`
# decodes for ten seconds and reports the frame rate.
frame-bench file seconds:
    cargo run -q --release -p quokka-playback --example frame_bench -- {{file}} {{seconds}}

# Play a file the way the output device does, and report how long each block waited for the decoder:
# `just underrun-probe song.flac 10`. Needs an optimised build, like frame-bench.
underrun-probe file block_ms:
    cargo run -q --release -p quokka-playback --example underrun_probe -- {{file}} {{block_ms}}
