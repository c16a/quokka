# core crate: agent instructions

Pure Rust library: SQLite index, scanning, metadata (`lofty`), FTS5 search and tags. It must not depend on any UI or playback crate.

- Schema changes: never edit `SCHEMA_V1` once it has shipped. Add a new step in `db::configure`, bump `SCHEMA_VERSION`, and keep the FTS triggers in sync with the `media` table.
- Files are identified by their path. Scanning compares size and mtime to skip unchanged files.
- Metadata reading must not fail the scan: a file lofty cannot parse is still indexed, without tags.
- Tests: integration tests live in `core/tests/`. They use `tempfile` and in-memory databases. Add a test for each new query or scan behaviour.
- The `scan` example (`just scan <db> <root>`) is the manual check for the scanner.
