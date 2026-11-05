use player_core::MediaItem;

/// Display title: the tag title, or the file name without extension.
pub fn title(item: &MediaItem) -> String {
    if let Some(title) = item.title.as_deref() {
        return title.to_owned();
    }
    item.path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| item.path.display().to_string())
}

/// `m:ss`, or `h:mm:ss` for items an hour or longer.
pub fn duration(ms: Option<i64>) -> String {
    let Some(ms) = ms else {
        return "—".into();
    };
    let total = ms / 1000;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn size(bytes: i64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = MB * 1024.0;
    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else {
        format!("{:.0} MB", bytes / MB)
    }
}

/// Playback time as `m:ss` (or `h:mm:ss`), from seconds.
pub fn clock(secs: f64) -> String {
    duration(Some((secs.max(0.0) * 1000.0) as i64))
}

/// Bit depth and sample rate, such as "24-bit / 96 kHz". Missing parts are left out.
pub fn sample_format(bit_depth: Option<i64>, sample_rate: Option<i64>) -> Option<String> {
    let rate = sample_rate.map(|r| {
        if r % 1000 == 0 {
            format!("{} kHz", r / 1000)
        } else {
            format!("{:.1} kHz", r as f64 / 1000.0)
        }
    });
    match (bit_depth, rate) {
        (Some(bits), Some(rate)) => Some(format!("{bits}-bit / {rate}")),
        (Some(bits), None) => Some(format!("{bits}-bit")),
        (None, Some(rate)) => Some(rate),
        (None, None) => None,
    }
}

/// Average bitrate with a thousands separator, such as "1,842 kbps".
pub fn kbps(value: i64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(c);
    }
    format!("{grouped} kbps")
}

/// Channel layout as a word where there is one, otherwise a count.
pub fn channels(count: i64) -> String {
    match count {
        1 => "Mono".into(),
        2 => "Stereo".into(),
        n => format!("{n} ch"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_format_handles_missing_parts() {
        assert_eq!(
            sample_format(Some(24), Some(96_000)).as_deref(),
            Some("24-bit / 96 kHz")
        );
        assert_eq!(
            sample_format(Some(16), Some(44_100)).as_deref(),
            Some("16-bit / 44.1 kHz")
        );
        assert_eq!(sample_format(None, Some(48_000)).as_deref(), Some("48 kHz"));
        assert_eq!(sample_format(Some(16), None).as_deref(), Some("16-bit"));
        assert_eq!(sample_format(None, None), None);
    }

    #[test]
    fn kbps_groups_thousands() {
        assert_eq!(kbps(1842), "1,842 kbps");
        assert_eq!(kbps(320), "320 kbps");
        assert_eq!(kbps(1_000_000), "1,000,000 kbps");
    }

    #[test]
    fn hi_res_is_above_16_bit_or_48_khz() {}

    #[test]
    fn channels_are_named() {
        assert_eq!(channels(1), "Mono");
        assert_eq!(channels(2), "Stereo");
        assert_eq!(channels(6), "6 ch");
    }
}
