//! Reading ISO base media (MP4 and MOV) boxes.

use std::io::{self, Read, Seek, SeekFrom};

/// The largest box read into memory. A `moov` box is kilobytes to a few megabytes.
const MAX_BOX: u64 = 64 * 1024 * 1024;

/// The boxes in a run of bytes, as (type, body) pairs. Stops at the first malformed box.
pub(crate) struct Boxes<'a> {
    rest: &'a [u8],
}

pub(crate) fn boxes(data: &[u8]) -> Boxes<'_> {
    Boxes { rest: data }
}

impl<'a> Iterator for Boxes<'a> {
    type Item = ([u8; 4], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.rest;
        if data.len() < 8 {
            return None;
        }
        let size32 = u32::from_be_bytes(data[0..4].try_into().ok()?);
        let kind: [u8; 4] = data[4..8].try_into().ok()?;
        let (size, header) = match size32 {
            1 => (
                u64::from_be_bytes(data.get(8..16)?.try_into().ok()?),
                16usize,
            ),
            0 => (data.len() as u64, 8usize),
            size => (u64::from(size), 8usize),
        };
        if size < header as u64 || size > data.len() as u64 {
            self.rest = &[];
            return None;
        }
        let size = size as usize;
        self.rest = &data[size..];
        Some((kind, &data[header..size]))
    }
}

/// The body of the first box of type `kind`, if there is one.
pub(crate) fn child<'a>(data: &'a [u8], kind: &[u8; 4]) -> Option<&'a [u8]> {
    boxes(data)
        .find(|(found, _)| found == kind)
        .map(|(_, body)| body)
}

/// The version and the bytes after the version and flags of a full box.
pub(crate) fn full_box(body: &[u8]) -> Option<(u8, &[u8])> {
    Some((*body.first()?, body.get(4..)?))
}

/// The body of the first top-level box of type `kind` in a file, read into memory.
pub(crate) fn find_top<R: Read + Seek>(
    file: &mut R,
    kind: &[u8; 4],
) -> io::Result<Option<Vec<u8>>> {
    let end = file.seek(SeekFrom::End(0))?;
    let mut offset = 0u64;
    while offset + 8 <= end {
        file.seek(SeekFrom::Start(offset))?;
        let mut header = [0u8; 16];
        file.read_exact(&mut header[..8])?;
        let size32 = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
        let (size, header_len) = match size32 {
            1 => {
                file.read_exact(&mut header[8..16])?;
                let mut large = [0u8; 8];
                large.copy_from_slice(&header[8..16]);
                (u64::from_be_bytes(large), 16u64)
            }
            0 => (end - offset, 8),
            size => (u64::from(size), 8),
        };
        if size < header_len || offset.checked_add(size).is_none_or(|stop| stop > end) {
            return Ok(None);
        }
        if &header[4..8] == kind {
            let body_len = size - header_len;
            if body_len > MAX_BOX {
                return Ok(None);
            }
            file.seek(SeekFrom::Start(offset + header_len))?;
            let mut body = vec![0u8; body_len as usize];
            file.read_exact(&mut body)?;
            return Ok(Some(body));
        }
        offset += size;
    }
    Ok(None)
}

/// A big-endian integer of `N` bytes at `at`.
pub(crate) fn be<const N: usize>(data: &[u8], at: usize) -> Option<u64> {
    let bytes = data.get(at..at + N)?;
    Some(
        bytes
            .iter()
            .fold(0u64, |value, &byte| (value << 8) | u64::from(byte)),
    )
}
