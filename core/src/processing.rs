//! The processing pass, run after each scan. The scan reads each changed file's tags and duration.
//! This pass reads the format of each video file that changed: its codec, picture size, display
//! aspect ratio, colour depth, chroma subsampling and frame rate, through the `video` module. A
//! file is marked processed once it has been read, whether or not anything was found, so it is not
//! read again until the scan finds it changed.

use std::path::Path;

use rusqlite::{Connection, params};

use crate::{Result, video};

/// Read the format of every video that has not been processed since it last changed.
pub(crate) fn process_pending(conn: &Connection) -> Result<()> {
    let pending: Vec<(i64, String)> = {
        let mut stmt = conn
            .prepare_cached("SELECT id, path FROM media WHERE kind = 'video' AND processed = 0")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect::<rusqlite::Result<_>>()?
    };

    let mut update = conn.prepare_cached(
        "UPDATE media SET video_codec = COALESCE(?2, video_codec), \
           width = COALESCE(?3, width), height = COALESCE(?4, height), \
           video_bit_depth = ?5, video_chroma = ?6, video_fps = ?7, \
           video_fps_variable = ?8, video_aspect = ?9, hdr = ?10, processed = 1 WHERE id = ?1",
    )?;
    for (id, path) in pending {
        // A file that cannot be read gives nothing, and is not tried again until it changes.
        let format = video::inspect(Path::new(&path)).unwrap_or_default();
        update.execute(params![
            id,
            format.codec.map(|codec| codec.name()),
            format.width.map(i64::from),
            format.height.map(i64::from),
            format.bit_depth.map(i64::from),
            format.chroma.map(|chroma| chroma.name()),
            format.frame_rate.map(|rate| rate.per_second()),
            format.variable_frame_rate,
            format.display_aspect,
            format.colour.hdr().map(i64::from),
        ])?;
    }
    Ok(())
}
