//! Reading EBML, the binary format Matroska and WebM use.

/// A variable-length integer. Element ids keep their length marker bits; sizes and values do not.
/// Returns the value and how many bytes it took.
pub(crate) fn vint(data: &[u8], keep_marker: bool) -> Option<(u64, usize)> {
    let first = *data.first()?;
    if first == 0 {
        return None;
    }
    let len = first.leading_zeros() as usize + 1;
    let bytes = data.get(..len)?;
    let mut value = if keep_marker {
        u64::from(first)
    } else {
        u64::from(first) & ((1u64 << (8 - len)) - 1)
    };
    for &byte in &bytes[1..] {
        value = (value << 8) | u64::from(byte);
    }
    Some((value, len))
}

/// The elements in a run of bytes, as (id, body) pairs. An element of unknown size runs to the
/// end of the bytes. A body cut short by the end of the bytes is returned as far as it goes.
pub(crate) struct Elements<'a> {
    rest: &'a [u8],
}

pub(crate) fn elements(data: &[u8]) -> Elements<'_> {
    Elements { rest: data }
}

impl<'a> Iterator for Elements<'a> {
    type Item = (u64, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.rest;
        let (id, id_len) = vint(rest, true)?;
        let (size, size_len) = vint(rest.get(id_len..)?, false)?;
        let start = id_len + size_len;
        // An all-ones size means the size is unknown.
        let unknown = size == (1u64 << (7 * size_len)) - 1;
        let end = if unknown {
            rest.len()
        } else {
            usize::try_from(size)
                .ok()
                .and_then(|size| start.checked_add(size))
                .unwrap_or(rest.len())
                .min(rest.len())
        };
        if end < start {
            self.rest = &[];
            return None;
        }
        self.rest = &rest[end..];
        Some((id, &rest[start..end]))
    }
}

/// An unsigned integer element body.
pub(crate) fn uint(body: &[u8]) -> u64 {
    body.iter()
        .fold(0u64, |value, &byte| (value << 8) | u64::from(byte))
}

/// A string element body, without any trailing NUL bytes.
pub(crate) fn text(body: &[u8]) -> String {
    String::from_utf8_lossy(body)
        .trim_end_matches('\0')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vints_keep_or_drop_their_marker() {
        assert_eq!(vint(&[0x81], false), Some((1, 1)));
        assert_eq!(vint(&[0x81], true), Some((0x81, 1)));
        assert_eq!(vint(&[0x40, 0x02], false), Some((2, 2)));
        assert_eq!(vint(&[0x00], false), None);
    }

    #[test]
    fn elements_are_split_and_unknown_sizes_run_to_the_end() {
        // Id 0xAE with size 2, then id 0xD7 with an unknown size (0xFF).
        let data = [0xAE, 0x82, 0xAA, 0xBB, 0xD7, 0xFF, 0x11];
        let found: Vec<_> = elements(&data).collect();
        assert_eq!(found, vec![(0xAE, &[0xAA, 0xBB][..]), (0xD7, &[0x11][..])]);
    }
}
