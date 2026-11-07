//! The bundled audio decoder: symphonia, in pure Rust. It covers the codecs symphonia has decoders
//! for, and it is the fallback where the operating system does not decode a codec.

use quokka_core::video::AudioCodec;
use symphonia::core::codecs::audio::well_known::{
    CODEC_ID_AAC, CODEC_ID_ALAC, CODEC_ID_FLAC, CODEC_ID_MP3,
};
use symphonia::core::codecs::audio::{AudioCodecParameters, AudioDecoderOptions};
use symphonia::core::packet::Packet;
use symphonia::core::units::{Duration, Timestamp};

use super::{AudioDecoder, AudioStream};
use crate::decode::DecodeError;

const BACKEND: &str = "symphonia";

/// A symphonia decoder for one stream.
pub struct Bundled {
    decoder: Box<dyn symphonia::core::codecs::audio::AudioDecoder>,
    /// How many frames have gone in, so each packet has a time.
    frames: i64,
}

impl Bundled {
    /// Open a decoder for the stream, if symphonia has one for its codec.
    pub fn open(stream: &AudioStream) -> Result<Self, DecodeError> {
        let codec = match stream.codec {
            AudioCodec::Aac => CODEC_ID_AAC,
            AudioCodec::Mp3 => CODEC_ID_MP3,
            AudioCodec::Flac => CODEC_ID_FLAC,
            AudioCodec::Alac => CODEC_ID_ALAC,
            other => {
                return Err(DecodeError::Unsupported {
                    backend: BACKEND,
                    reason: format!("no bundled decoder for {}", other.name()),
                });
            }
        };
        let params = AudioCodecParameters {
            codec,
            sample_rate: Some(stream.sample_rate),
            extra_data: (!stream.config.is_empty())
                .then(|| stream.config.clone().into_boxed_slice()),
            ..AudioCodecParameters::new()
        };
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(&params, &AudioDecoderOptions::default())
            .map_err(|error| DecodeError::Unsupported {
                backend: BACKEND,
                reason: error.to_string(),
            })?;
        Ok(Bundled { decoder, frames: 0 })
    }
}

impl AudioDecoder for Bundled {
    fn name(&self) -> &'static str {
        BACKEND
    }

    fn decode(&mut self, packet: &[u8]) -> Result<Vec<f32>, DecodeError> {
        let frame = Packet::new(
            0,
            Timestamp::new(self.frames),
            Duration::new(0),
            packet.to_vec(),
        );
        self.frames += 1;
        let decoded = self
            .decoder
            .decode(&frame)
            .map_err(|error| DecodeError::Failed {
                backend: BACKEND,
                reason: error.to_string(),
            })?;
        let mut samples = Vec::new();
        decoded.copy_to_vec_interleaved(&mut samples);
        Ok(samples)
    }

    fn finish(&mut self) -> Result<Vec<f32>, DecodeError> {
        Ok(Vec::new())
    }

    fn reset(&mut self) {
        self.decoder.reset();
    }
}
