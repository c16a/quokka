# Agent instructions

Cross-platform Rust media library and player (macOS, Linux, Windows). Layout: `core/` (library), `playback/` (playback trait and backends). Each crate has its own `AGENTS.md`; read the nearest one before editing a crate.

## Toolchain
- Rust is pinned to 1.99.1 in `rust-toolchain.toml`. Edition is 2024 (`[workspace.package]`). Do not change either without the user asking.

## Commands: use `just`
- Run builds, tests and other recipes through `just`, not raw `cargo`: `just build`, `just test`, `just fmt`, `just check`, `just scan <db> <root>`.
- Before you finish a change, `just check` must pass (formatting, clippy with warnings as errors, tests).
- When you finish a change, also run `just bundle-release`, so `~/Applications/Quokka.app` matches the code. The debug bundle (`just bundle`) is far too slow to play video with.
- When you finish a turn, commit the working tree with a clear message, once `just check` and `just bundle-release` pass.
- Cross-compilation uses `cargo zigbuild` through the recipes: `just cross <target>` or `just cross-all`. Do not invoke plain `cargo build --target` for other platforms. Install a target with `rustup target add <target>` first.

## Dependencies
- Declare every third-party dependency once, in the root `Cargo.toml` under `[workspace.dependencies]`. Crates reference it with `dep.workspace = true`. Do not add a version to a crate's own `Cargo.toml`.
- Default features are always disabled (`default-features = false`). Enable only the features you need, explicitly, in the root file.

## Code
- Library code returns errors (`thiserror` enums); it does not panic on bad input.
- Keep backend-specific types out of public APIs. The UI depends on the `Player` trait and the `core` types, not on libmpv, FFmpeg or SQLite types.
