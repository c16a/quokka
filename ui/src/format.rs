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
