//! How loud a track is overall, measured to EBU R128 (ITU-R BS.1770), for levelling files that
//! carry no ReplayGain tags. The whole track is decoded, so this is slow; call it off the UI thread.

use std::path::Path;

use ebur128::{EbuR128, Mode};
use rodio::Source;

use crate::symphonia_source::{ErrorSlot, SymphoniaSource};

/// Frames per chunk fed to the meter.
const CHUNK_FRAMES: usize = 4096;

/// The integrated loudness of a track, and its highest sample level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loudness {
    /// Integrated loudness, in LUFS.
    pub lufs: f64,
    /// The highest sample level, as a linear amplitude where 1.0 is full scale.
    pub peak: f64,
}

/// Decode the file at `path` and measure it. `None` when it can't be read, or is silent.
pub fn measure_loudness(path: &Path) -> Option<Loudness> {
    let mut source = SymphoniaSource::open(path, ErrorSlot::default()).ok()?;
    let channels = source.channels().get();
    let rate = source.sample_rate().get();
    let mut meter = EbuR128::new(u32::from(channels), rate, Mode::I | Mode::SAMPLE_PEAK).ok()?;
    let chunk = usize::from(channels) * CHUNK_FRAMES;
    let mut buffer = Vec::with_capacity(chunk);
    loop {
        buffer.clear();
        buffer.extend(source.by_ref().take(chunk));
        // Whole frames only: a chunk may end mid-frame at the end of the file.
        let whole = buffer.len() - buffer.len() % usize::from(channels);
        if whole == 0 {
            break;
        }
        meter.add_frames_f32(&buffer[..whole]).ok()?;
    }
    let lufs = meter.loudness_global().ok()?;
    let mut peak = 0.0f64;
    for channel in 0..u32::from(channels) {
        peak = peak.max(meter.sample_peak(channel).ok()?);
    }
    (lufs.is_finite() && peak > 0.0).then_some(Loudness { lufs, peak })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::measure_loudness;

    /// Write a 16-bit mono PCM WAV file.
    fn write_wav(path: &Path, rate: u32, samples: &[i16]) {
        let data_len = (samples.len() * 2) as u32;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            out.extend_from_slice(&sample.to_le_bytes());
        }
        std::fs::write(path, out).unwrap();
    }

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("quokka-loudness-{}-{name}.wav", std::process::id()))
    }

    #[test]
    fn a_tone_is_measured_at_its_level() {
        let path = temp("tone");
        let rate = 48_000u32;
        // A 1 kHz sine at amplitude 0.1 (about -20 dBFS), for three seconds.
        let samples: Vec<i16> = (0..rate * 3)
            .map(|n| {
                let t = n as f64 / f64::from(rate);
                ((2.0 * std::f64::consts::PI * 1000.0 * t).sin() * 0.1 * 32767.0) as i16
            })
            .collect();
        write_wav(&path, rate, &samples);

        let measured = measure_loudness(&path).expect("a tone has a loudness");
        assert!(
            (-30.0..-15.0).contains(&measured.lufs),
            "a 0.1 amplitude tone is about -23 LUFS, got {}",
            measured.lufs
        );
        assert!(
            (measured.peak - 0.1).abs() < 0.01,
            "the peak is the amplitude, got {}",
            measured.peak
        );
    }

    #[test]
    fn silence_has_no_loudness() {
        let path = temp("silence");
        write_wav(&path, 48_000, &[0i16; 48_000]);
        assert_eq!(measure_loudness(&path), None);
    }
}
