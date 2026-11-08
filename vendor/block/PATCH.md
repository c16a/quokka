# Patched copy of `block` 0.1.6

This is `block` 0.1.6 from crates.io with one change, made so the crate builds without the
`invalid_value` / uninhabited-static future-incompatibility warning that newer rustc reports
(rust-lang/rust#74840).

- `src/lib.rs`: `enum Class {}` (uninhabited) becomes `struct Class { _private: [u8; 0] }`.
  `Class` is only ever used by address (`&_NSConcreteStackBlock`, `*const Class`), so its
  layout and the public API are unchanged.
- `Cargo.toml`: the `[dev-dependencies]` on `objc_test_utils` is removed because its
  `test_utils` path is not vendored and the tests are not run here.

The crate is pulled in through `[patch.crates-io]` in the root `Cargo.toml`, via
`souvlaki` (macOS media controls). Remove the patch once `souvlaki` no longer depends on `block`.

Licence: MIT, as upstream (see the `license` field in `Cargo.toml`).
