//! Audio decoding for movie soundtracks and music. The operating system's decoder is tried first,
//! and a bundled decoder covers what it does not handle.

#[cfg(target_os = "macos")]
pub mod audiotoolbox;
pub mod bundled;

use player_core::video::AudioCodec;

use super::DecodeError;

/// What a decoder needs to know about an audio stream before it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioStream {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub channels: u32,
    /// The codec's configuration: the AAC `AudioSpecificConfig`, or the AC-3 and E-AC-3 records.
    pub config: Vec<u8>,
}

/// An audio decoder. Each compressed frame goes in, and the interleaved samples it decodes to come
/// out, as 32-bit floats.
pub trait AudioDecoder {
    /// The backend's name, for logs and the info panel.
    fn name(&self) -> &'static str;

    /// Decode one compressed frame.
    fn decode(&mut self, packet: &[u8]) -> Result<Vec<f32>, DecodeError>;

    /// The samples the decoder still holds once the last frame has gone in.
    fn finish(&mut self) -> Result<Vec<f32>, DecodeError>;

    /// Forget the frames held so far, as after a seek. The next frame starts a new run.
    fn reset(&mut self);
}

/// Open the first backend that accepts the stream. The operating system's decoder goes first.
pub fn open(stream: &AudioStream) -> Result<Box<dyn AudioDecoder>, DecodeError> {
    #[cfg(target_os = "macos")]
    let system = audiotoolbox::AudioToolbox::open(stream)
        .map(|decoder| Box::new(decoder) as Box<dyn AudioDecoder>);
    #[cfg(not(target_os = "macos"))]
    let system: Result<Box<dyn AudioDecoder>, DecodeError> = Err(DecodeError::Unsupported {
        backend: "system",
        reason: "no system audio decoder on this platform".into(),
    });
    match system {
        Ok(decoder) => Ok(decoder),
        Err(_) => {
            bundled::Bundled::open(stream).map(|decoder| Box::new(decoder) as Box<dyn AudioDecoder>)
        }
    }
}
