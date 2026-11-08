//! The user's preferred languages, read from the operating system, as BCP 47 tags such as `en-US`
//! or `zh-Hant-TW`, most preferred first. Language files in this module are named by these tags.
//!
//! macOS reads the language list from System Settings, Windows the display-language list, and
//! other systems the POSIX locale variables. On every system the locale variables follow the
//! system list, so a language set only in the environment is still found.

/// The preferred languages, most preferred first, without repeats. Empty when the system names
/// none, or only the `C` or `POSIX` locale.
pub fn preferred_languages() -> Vec<String> {
    let mut tags = Vec::new();
    for tag in system_languages()
        .into_iter()
        .chain(environment_languages())
        .filter_map(|raw| normalise(&raw))
    {
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags
}

/// The languages in the POSIX locale variables, in the order gettext reads them: the `LANGUAGE`
/// list, then the first of `LC_ALL`, `LC_MESSAGES` and `LANG` that is set.
fn environment_languages() -> Vec<String> {
    let var = |name| std::env::var(name).ok().filter(|v| !v.is_empty());
    let mut languages: Vec<String> = var("LANGUAGE")
        .map(|list| list.split(':').map(str::to_owned).collect())
        .unwrap_or_default();
    languages.extend(["LC_ALL", "LC_MESSAGES", "LANG"].into_iter().find_map(var));
    languages
}

#[cfg(target_os = "macos")]
fn system_languages() -> Vec<String> {
    use objc2_foundation::NSLocale;

    NSLocale::preferredLanguages()
        .iter()
        .map(|language| language.to_string())
        .collect()
}

#[cfg(windows)]
fn system_languages() -> Vec<String> {
    /// Asks for the languages by name (`en-US`), not by numeric identifier.
    const MUI_LANGUAGE_NAME: u32 = 0x8;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserPreferredUILanguages(
            flags: u32,
            languages: *mut u32,
            buffer: *mut u16,
            buffer_len: *mut u32,
        ) -> i32;
    }

    let mut count = 0;
    let mut len = 0;
    // SAFETY: a null buffer asks only for the length, which is written to `len`.
    let sized = unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut count,
            std::ptr::null_mut(),
            &mut len,
        )
    };
    if sized == 0 || len == 0 {
        return Vec::new();
    }
    let mut buffer = vec![0u16; len as usize];
    // SAFETY: `buffer` holds `len` UTF-16 units, as `len` tells the call.
    let filled = unsafe {
        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, buffer.as_mut_ptr(), &mut len)
    };
    if filled == 0 {
        return Vec::new();
    }
    // The buffer is a list of names, each ended by a null, and the list by a second null.
    buffer
        .split(|&unit| unit == 0)
        .filter(|name| !name.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn system_languages() -> Vec<String> {
    Vec::new()
}

/// `raw` as a BCP 47 tag: `en_US.UTF-8@euro` becomes `en-US`, `zh-hans-cn` becomes `zh-Hans-CN`.
/// `None` for an empty locale or the `C` and `POSIX` locales, which name no language.
fn normalise(raw: &str) -> Option<String> {
    let locale = raw.split(['.', '@']).next().unwrap_or_default().trim();
    if locale.is_empty() || locale == "C" || locale == "POSIX" {
        return None;
    }
    let mut parts = locale.split(['-', '_']).filter(|part| !part.is_empty());
    let language = parts.next()?;
    if !(2..=8).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut tag = language.to_ascii_lowercase();
    for part in parts {
        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        tag.push('-');
        match part.len() {
            // A script, such as `Hant`.
            4 if part.chars().all(|c| c.is_ascii_alphabetic()) => {
                tag.push_str(&part[..1].to_ascii_uppercase());
                tag.push_str(&part[1..].to_ascii_lowercase());
            }
            // A region, such as `US` or `419`.
            2 | 3 => tag.push_str(&part.to_ascii_uppercase()),
            _ => tag.push_str(&part.to_ascii_lowercase()),
        }
    }
    Some(tag)
}

#[cfg(test)]
mod tests {
    use super::normalise;

    #[test]
    fn normalises_posix_locales() {
        assert_eq!(normalise("en_US.UTF-8").as_deref(), Some("en-US"));
        assert_eq!(normalise("de_DE@euro").as_deref(), Some("de-DE"));
        assert_eq!(normalise("fr").as_deref(), Some("fr"));
    }

    #[test]
    fn normalises_bcp47_case() {
        assert_eq!(normalise("zh-hant-tw").as_deref(), Some("zh-Hant-TW"));
        assert_eq!(normalise("ES-419").as_deref(), Some("es-419"));
    }

    #[test]
    fn rejects_locales_without_a_language() {
        assert_eq!(normalise("C"), None);
        assert_eq!(normalise("POSIX"), None);
        assert_eq!(normalise("C.UTF-8"), None);
        assert_eq!(normalise(""), None);
        assert_eq!(normalise("1_US"), None);
    }
}
