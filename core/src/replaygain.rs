//! ReplayGain: loudness corrections that taggers write into files, so tracks play at a similar
//! level. They are read from the file when a track starts, not stored in the library.

use std::path::Path;

use lofty::file::TaggedFileExt;
use lofty::tag::ItemKey;

use crate::settings::ReplayGainMode;

/// The factor to multiply a track's output level by. `1.0` means no change, which is the answer
/// when the mode is off or the file has no ReplayGain tags.
pub fn replay_gain_factor(path: &Path, mode: ReplayGainMode) -> f32 {
    tag_gain(path, mode).unwrap_or(1.0)
}

/// The factor the file's ReplayGain tags give, or `None` when the mode is off or the file has no
/// such tags.
pub fn tag_gain(path: &Path, mode: ReplayGainMode) -> Option<f32> {
    if mode == ReplayGainMode::Off {
        return None;
    }
    let tagged = lofty::read_from_path(path).ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag())?;
    let gain = |key| tag.get_string(key).and_then(parse_db);
    let peak = |key| {
        tag.get_string(key)
            .and_then(|s| s.trim().parse::<f32>().ok())
    };
    // A (gain, peak) pair, when the file has a gain for it.
    let pick = |(gain, peak): (Option<f32>, Option<f32>)| gain.map(|g| (g, peak));
    let track = pick((
        gain(ItemKey::ReplayGainTrackGain),
        peak(ItemKey::ReplayGainTrackPeak),
    ));
    let album = pick((
        gain(ItemKey::ReplayGainAlbumGain),
        peak(ItemKey::ReplayGainAlbumPeak),
    ));
    let chosen = match mode {
        ReplayGainMode::Album => album.or(track),
        _ => track.or(album),
    };
    let (gain_db, peak) = chosen?;

    let mut factor = 10f32.powf(gain_db / 20.0);
    // Never push the loudest sample past full scale.
    if let Some(peak) = peak
        && peak > 0.0
    {
        factor = factor.min(1.0 / peak);
    }
    Some(factor)
}

/// The level tracks are brought to when they are measured, in LUFS (the ReplayGain 2 reference).
const TARGET_LUFS: f64 = -18.0;

/// The factor that brings a measured track to `TARGET_LUFS`, without pushing its peak past full
/// scale. `peak` is the highest sample level as a linear amplitude.
pub fn loudness_factor(lufs: f64, peak: f64) -> f32 {
    let mut factor = 10f64.powf((TARGET_LUFS - lufs) / 20.0);
    if peak > 0.0 {
        factor = factor.min(1.0 / peak);
    }
    factor as f32
}

/// A gain such as "-6.48 dB" in decibels. Anything that isn't a finite number is ignored.
fn parse_db(text: &str) -> Option<f32> {
    let text = text.trim();
    let text = text
        .strip_suffix("dB")
        .or_else(|| text.strip_suffix("db"))
        .unwrap_or(text);
    let db: f32 = text.trim().parse().ok()?;
    db.is_finite().then_some(db)
}

#[cfg(test)]
mod tests {
    use super::parse_db;

    #[test]
    fn parses_gains_with_and_without_units() {
        assert_eq!(parse_db("-6.48 dB"), Some(-6.48));
        assert_eq!(parse_db("+3.2 dB"), Some(3.2));
        assert_eq!(parse_db(" 1.5 "), Some(1.5));
        assert_eq!(parse_db("loud"), None);
        assert_eq!(parse_db("NaN dB"), None);
    }
}
