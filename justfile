# Default: run the full local check.
default: check

# Build the whole workspace.
build:
    cargo build --workspace

# Build optimised.
release:
    cargo build --workspace --release

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
    cargo run -p player-app

# Package the debug build as ~/Applications/Player.app, so macOS and the computer-use tools can find it
# by name. The bundle is ad-hoc signed, for this Mac only. The Info.plist is in packaging/.
bundle: build
    APP="$HOME/Applications/Player.app"; rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS"; cp target/debug/player "$APP/Contents/MacOS/player"; cp packaging/Info.plist "$APP/Contents/Info.plist"; codesign --force --sign - "$APP"; echo "Built $APP"

# Scan a directory into a library database: `just scan ./library.db ~/Music`.
scan db root:
    cargo run -p player-core --example scan -- {{db}} {{root}}

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
