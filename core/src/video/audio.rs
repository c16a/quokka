//! Audio tracks of movie files: the codec, its sample rate and channels, its configuration, and the
//! compressed frames, for decoders. Each frame is one packet the decoder takes as it is.

use super::bits::Bits;
use super::samples::Sample;

/// The audio codecs the movie readers handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    /// AAC (MPEG-4 AAC LC, HE-AAC and the like).
    Aac,
    /// Dolby Digital (AC-3).
    Ac3,
    /// Dolby Digital Plus (E-AC-3).
    Eac3,
    /// MPEG-1 Layer 3 (MP3), as music uses it.
    Mp3,
    /// Free Lossless Audio Codec (FLAC).
    Flac,
    /// Apple Lossless Audio Codec (ALAC).
    Alac,
}

impl AudioCodec {
    /// The name people know the codec by.
    pub fn name(self) -> &'static str {
        match self {
            AudioCodec::Aac => "AAC",
            AudioCodec::Ac3 => "AC-3",
            AudioCodec::Eac3 => "E-AC-3",
            AudioCodec::Mp3 => "MP3",
            AudioCodec::Flac => "FLAC",
            AudioCodec::Alac => "ALAC",
        }
    }
}

/// The first audio track of a movie file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioTrack {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub channels: u32,
    /// The codec's configuration: the AAC `AudioSpecificConfig`, or the `dac3` or `dec3` record.
    /// Empty when the container does not store one.
    pub config: Vec<u8>,
    /// Ticks per second for the samples' times.
    pub timescale: u64,
    pub samples: Vec<Sample>,
}

/// The sample rates AAC indexes, by `samplingFrequencyIndex`.
const AAC_RATES: [u32; 13] = [
    96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350,
];

/// The channel counts AC-3 gives for each audio coding mode, not counting the LFE channel.
const AC3_CHANNELS: [u32; 8] = [2, 1, 2, 3, 3, 4, 4, 5];

/// The sample rates AC-3 gives for each `fscod`.
const AC3_RATES: [u32; 3] = [48000, 44100, 32000];

/// The sample rates E-AC-3 gives for `fscod` 3, by `fscod2`.
const EAC3_HALF_RATES: [u32; 3] = [24000, 22050, 16000];

/// The sample rate and channel count of an AAC `AudioSpecificConfig`.
pub(crate) fn aac_format(config: &[u8]) -> Option<(u32, u32)> {
    let mut bits = Bits::new(config);
    // audioObjectType, with an escape to a second field for the types above 30.
    if bits.bits(5)? == 31 {
        bits.skip(6)?;
    }
    let index = bits.bits(4)?;
    let rate = if index == 15 {
        bits.bits(24)? as u32
    } else {
        *AAC_RATES.get(index as usize)?
    };
    let channels = match bits.bits(4)? {
        config @ 1..=6 => config as u32,
        7 => 8,
        _ => return None,
    };
    Some((rate, channels))
}

/// The sample rate and channel count of an AC-3 `dac3` record.
pub(crate) fn ac3_from_dac3(record: &[u8]) -> Option<(u32, u32)> {
    // fscod (2 bits), bsid (5), bsmod (3), acmod (3), lfeon (1), then the bit rate.
    let fscod = usize::from(*record.first()? >> 6);
    let acmod = usize::from((*record.get(1)? >> 3) & 0x07);
    let lfe = u32::from((record[1] >> 2) & 0x01);
    Some((*AC3_RATES.get(fscod)?, AC3_CHANNELS[acmod] + lfe))
}

/// The sample rate and channel count of an E-AC-3 `dec3` record, from its first substream.
pub(crate) fn eac3_from_dec3(record: &[u8]) -> Option<(u32, u32)> {
    let mut bits = Bits::new(record);
    // data_rate (13 bits), num_ind_sub (3 bits), then the substream: fscod (2), bsid (5),
    // reserved (1), asvc (1), bsmod (3), acmod (3), lfeon (1).
    bits.skip(16)?;
    let fscod = bits.bits(2)? as usize;
    bits.skip(5 + 1 + 1 + 3)?;
    let acmod = bits.bits(3)? as usize;
    let lfe = bits.bits(1)? as u32;
    Some((*AC3_RATES.get(fscod)?, AC3_CHANNELS[acmod] + lfe))
}

/// The sample rate and channel count of an AC-3 frame, from its sync frame header. Used where the
/// container stores no record.
pub(crate) fn ac3_from_frame(frame: &[u8]) -> Option<(u32, u32)> {
    if frame.get(0..2)? != [0x0B, 0x77] {
        return None;
    }
    // Skip the sync word and the CRC: fscod and frmsizecod come next.
    let mut bits = Bits::new(frame.get(4..)?);
    let fscod = bits.bits(2)? as usize;
    bits.skip(6 + 5 + 3)?;
    let acmod = bits.bits(3)?;
    if acmod & 0x01 != 0 && acmod != 0x01 {
        bits.skip(2)?;
    }
    if acmod & 0x04 != 0 {
        bits.skip(2)?;
    }
    if acmod == 0x02 {
        bits.skip(2)?;
    }
    let lfe = bits.bits(1)? as u32;
    Some((*AC3_RATES.get(fscod)?, AC3_CHANNELS[acmod as usize] + lfe))
}

/// The sample rate and channel count of an E-AC-3 frame, from its sync frame header.
pub(crate) fn eac3_from_frame(frame: &[u8]) -> Option<(u32, u32)> {
    if frame.get(0..2)? != [0x0B, 0x77] {
        return None;
    }
    // strmtyp (2), substreamid (3), frmsiz (11), then fscod (2), and fscod2 or numblks.
    let mut bits = Bits::new(frame.get(2..)?);
    bits.skip(2 + 3 + 11)?;
    let fscod = bits.bits(2)? as usize;
    let rate = if fscod == 3 {
        *EAC3_HALF_RATES.get(bits.bits(2)? as usize)?
    } else {
        bits.skip(2)?;
        *AC3_RATES.get(fscod)?
    };
    let acmod = bits.bits(3)?;
    let lfe = bits.bits(1)? as u32;
    Some((rate, AC3_CHANNELS[acmod as usize] + lfe))
}

/// The body of the first descriptor with the tag `wanted` in an MPEG-4 `esds` box, searching into
/// the descriptors that hold others. Descriptor lengths are variable: seven bits per byte.
pub(crate) fn descriptor(mut data: &[u8], wanted: u8) -> Option<&[u8]> {
    while let Some(&tag) = data.first() {
        let mut length = 0usize;
        let mut used = 1;
        loop {
            let byte = *data.get(used)?;
            used += 1;
            length = (length << 7) | usize::from(byte & 0x7f);
            if byte & 0x80 == 0 || used > 4 {
                break;
            }
        }
        let body = data.get(used..used + length)?;
        if tag == wanted {
            return Some(body);
        }
        let inner = match tag {
            // ES_Descriptor: ES_ID (2 bytes), flags (1 byte), then optional fields the flags name.
            0x03 => {
                let flags = *body.get(2)?;
                let mut at = 3;
                if flags & 0x80 != 0 {
                    at += 2;
                }
                if flags & 0x40 != 0 {
                    at += 1 + usize::from(*body.get(at)?);
                }
                if flags & 0x20 != 0 {
                    at += 2;
                }
                body.get(at..)?
            }
            // DecoderConfigDescriptor: 13 fixed bytes, then the decoder specific information.
            0x04 => body.get(13..)?,
            _ => &[],
        };
        if let Some(found) = descriptor(inner, wanted) {
            return Some(found);
        }
        data = data.get(used + length..)?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aac_config_gives_rate_and_channels() {
        // AAC LC (2), 48 kHz (index 3), stereo (2): 00010 0011 0010 000
        assert_eq!(aac_format(&[0x11, 0x90]), Some((48000, 2)));
    }

    #[test]
    fn dac3_gives_rate_and_channels() {
        // fscod 0 (48 kHz), acmod 7 (three front, two rear), LFE on: 5.1.
        assert_eq!(ac3_from_dac3(&[0x00, 0x3C, 0x00]), Some((48000, 6)));
    }

    #[test]
    fn esds_descriptors_lead_to_the_audio_config() {
        // ES_Descriptor (3) with ES_ID 1, then DecoderConfigDescriptor (4) with 13 fixed bytes, then
        // DecoderSpecificInfo (5) holding the two-byte config.
        let mut esds = vec![0x03, 22, 0x00, 0x01, 0x00, 0x04, 17];
        esds.extend([0x40, 0x15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        esds.extend([0x05, 2, 0x11, 0x90]);
        assert_eq!(descriptor(&esds, 0x05), Some(&[0x11, 0x90][..]));
    }
}
