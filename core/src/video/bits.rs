//! Bit-level reading shared by the codec header parsers.

/// Reads bits most significant first, with the Exp-Golomb codes H.264 and HEVC use.
pub(crate) struct Bits<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Bits<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    pub(crate) fn bit(&mut self) -> Option<u64> {
        let byte = *self.data.get(self.at / 8)?;
        let bit = (byte >> (7 - self.at % 8)) & 1;
        self.at += 1;
        Some(u64::from(bit))
    }

    pub(crate) fn flag(&mut self) -> Option<bool> {
        Some(self.bit()? == 1)
    }

    /// An unsigned value of `count` bits, `f(n)` in the specifications.
    pub(crate) fn bits(&mut self, count: u32) -> Option<u64> {
        (0..count).try_fold(0u64, |value, _| Some((value << 1) | self.bit()?))
    }

    pub(crate) fn skip(&mut self, count: u32) -> Option<()> {
        for _ in 0..count {
            self.bit()?;
        }
        Some(())
    }

    /// An unsigned Exp-Golomb code, `ue(v)`.
    pub(crate) fn ue(&mut self) -> Option<u64> {
        let mut zeros = 0u32;
        while self.bit()? == 0 {
            zeros += 1;
            if zeros > 31 {
                return None;
            }
        }
        Some((1u64 << zeros) - 1 + self.bits(zeros)?)
    }

    /// A signed Exp-Golomb code, `se(v)`.
    pub(crate) fn se(&mut self) -> Option<i64> {
        let code = self.ue()?;
        Some(if code % 2 == 1 {
            code.div_ceil(2) as i64
        } else {
            -((code / 2) as i64)
        })
    }
}

/// Remove emulation prevention bytes: a `03` that follows two zero bytes is not data.
pub(crate) fn unescape(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut zeros = 0;
    for &byte in data {
        if zeros >= 2 && byte == 3 {
            zeros = 0;
            continue;
        }
        zeros = if byte == 0 { zeros + 1 } else { 0 };
        out.push(byte);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_golomb_codes_are_read() {
        // 1 | 010 | 011 | 00100 : ue values 0, 1, 2, 3
        let data = [0b1010_0110, 0b0100_0000];
        let mut bits = Bits::new(&data);
        assert_eq!(bits.ue(), Some(0));
        assert_eq!(bits.ue(), Some(1));
        assert_eq!(bits.ue(), Some(2));
        assert_eq!(bits.ue(), Some(3));
    }

    #[test]
    fn signed_exp_golomb_codes_alternate_sign() {
        // ue 1 -> +1, ue 2 -> -1, ue 3 -> +2
        let data = [0b0100_1100, 0b1000_0000];
        let mut bits = Bits::new(&data);
        assert_eq!(bits.se(), Some(1));
        assert_eq!(bits.se(), Some(-1));
        assert_eq!(bits.se(), Some(2));
    }

    #[test]
    fn emulation_prevention_bytes_are_removed() {
        assert_eq!(
            unescape(&[0, 0, 3, 1, 0, 0, 3, 0, 0, 0]),
            vec![0, 0, 1, 0, 0, 0, 0, 0]
        );
    }
}
