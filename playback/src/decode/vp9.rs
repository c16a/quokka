//! A bundled VP9 decoder: rusty_vp9, a pure-Rust decoder under the Apache 2.0 licence. It is the
//! fallback where the operating system does not decode VP9, which covers every Mac.

use player_core::video::Codec;
use rusty_vp9::{DecodedFrame, Error as EngineError, Vp9Decoder as Engine};

use super::{DecodeError, Packet, Picture, StreamConfig, Surface, VideoDecoder, layout_code};

const BACKEND: &str = "rusty_vp9";

/// A VP9 decoding session.
pub struct Vp9 {
    engine: Engine,
    pictures: Vec<Picture>,
}

impl Vp9 {
    /// Open a session for a VP9 stream.
    pub fn open(stream: &StreamConfig) -> Result<Self, DecodeError> {
        if stream.codec != Codec::Vp9 {
            return Err(DecodeError::Unsupported {
                backend: BACKEND,
                reason: "only VP9 is handled".into(),
            });
        }
        Ok(Vp9 {
            engine: Engine::new(),
            pictures: Vec::new(),
        })
    }

    /// Collect every frame the engine can output now.
    fn drain(&mut self) -> Result<(), DecodeError> {
        loop {
            match self.engine.next_frame() {
                Ok(frame) => self.pictures.push(picture(frame)),
                // More input is needed, or the stream is drained.
                Err(EngineError::Again) | Err(EngineError::Eof) => return Ok(()),
                Err(error) => {
                    return Err(DecodeError::Failed {
                        backend: BACKEND,
                        reason: error.to_string(),
                    });
                }
            }
        }
    }
}

impl VideoDecoder for Vp9 {
    fn name(&self) -> &'static str {
        BACKEND
    }

    fn decode(&mut self, packet: &Packet) -> Result<(), DecodeError> {
        self.engine
            .push(&packet.data, Some(packet.presentation_time))
            .map_err(|error| DecodeError::Failed {
                backend: BACKEND,
                reason: error.to_string(),
            })?;
        self.drain()
    }

    fn finish(&mut self) -> Result<(), DecodeError> {
        self.engine.flush();
        self.drain()
    }

    fn take_pictures(&mut self) -> Vec<Picture> {
        let mut pictures = std::mem::take(&mut self.pictures);
        pictures.sort_by_key(|picture| picture.presentation_time);
        pictures
    }
}

/// A decoded frame as a picture: its planes packed row by row, one after another, with no padding.
fn picture(frame: DecodedFrame) -> Picture {
    let bytes_per_sample = if frame.bit_depth > 8 { 2 } else { 1 };
    let width = frame.width as usize;
    let height = frame.height as usize;
    let (x, y) = (frame.subsampling_x as usize, frame.subsampling_y as usize);
    let chroma_width = (width + x) >> x;
    let chroma_height = (height + y) >> y;
    let sizes = [
        (width, height),
        (chroma_width, chroma_height),
        (chroma_width, chroma_height),
    ];

    let mut samples = Vec::new();
    for (index, plane) in frame.planes.iter().enumerate() {
        let Some(&(plane_width, plane_height)) = sizes.get(index) else {
            break;
        };
        let stride = frame.strides[index];
        let row = plane_width * bytes_per_sample;
        for line in 0..plane_height {
            let start = line * stride;
            match plane.get(start..start + row) {
                Some(bytes) => samples.extend_from_slice(bytes),
                None => samples.resize(samples.len() + row, 0),
            }
        }
    }

    Picture {
        width: frame.width,
        height: frame.height,
        presentation_time: frame.pts.unwrap_or(0),
        pixel_format: layout_code(frame.subsampling_x, frame.subsampling_y, frame.bit_depth),
        surface: Surface::Cpu(samples),
    }
}
