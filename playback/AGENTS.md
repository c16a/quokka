# playback crate: agent instructions

Defines the `Player` trait, the single interface the app uses for playback, and the in-memory `MockPlayer`.

- This crate must not depend on `player-core` or any UI crate.
- Backends (libmpv first, FFmpeg later) implement `Player` in their own modules or crates. Their backend types must not appear in the trait.
- Keep the state rules in `PlayerState` consistent across backends. `MockPlayer` is the reference for them and is covered by a unit test.
- Add any new control to the trait only when every backend can implement it.
