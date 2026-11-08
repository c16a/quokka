//! A bundled H.264 and HEVC decoder for Linux, through FFmpeg's libavcodec. Linux has no system
//! decoder for these codecs, so this is the one that plays them there. macOS uses VideoToolbox.

use ffmpeg_next as av;
use quokka_core::video::Codec;

use super::{DecodeError, Packet, Picture, StreamConfig, Surface, VideoDecoder, layout_code};

const BACKEND: &str = "libavcodec";

/// A session with libavcodec's decoder for one stream.
pub struct Libav {
    decoder: av::decoder::Video,
    pictures: Vec<Picture>,
}

impl Libav {
    /// Open a session for an H.264 or HEVC stream. The stream's configuration record (`avcC` or
    /// `hvcC`) is handed to the decoder as its extradata, so it reads the packets as they are.
    pub fn open(stream: &StreamConfig) -> Result<Self, DecodeError> {
        let id = match stream.codec {
            Codec::H264 => av::codec::Id::H264,
            Codec::Hevc => av::codec::Id::HEVC,
            _ => return Err(unsupported("only H.264 and HEVC are handled")),
        };
        av::init().map_err(failed)?;
        let codec =
            av::decoder::find(id).ok_or_else(|| unsupported("no decoder for this codec"))?;
        let mut context = av::codec::context::Context::new_with_codec(codec);
        set_extradata(&mut context, &stream.config)?;
        let decoder = context.decoder().video().map_err(failed)?;
        Ok(Libav {
            decoder,
            pictures: Vec::new(),
        })
    }

    /// Collect every picture the decoder can output now.
    fn drain(&mut self) -> Result<(), DecodeError> {
        let mut frame = av::frame::Video::empty();
        loop {
            match self.decoder.receive_frame(&mut frame) {
                Ok(()) => self.pictures.push(picture(&frame)?),
                // More input is needed, or the stream is drained.
                Err(av::Error::Other { errno }) if errno == av::error::EAGAIN => return Ok(()),
                Err(av::Error::Eof) => return Ok(()),
                Err(error) => return Err(failed(error)),
            }
        }
    }
}

impl VideoDecoder for Libav {
    fn name(&self) -> &'static str {
        BACKEND
    }

    fn decode(&mut self, packet: &Packet) -> Result<(), DecodeError> {
        let mut input = av::Packet::copy(&packet.data);
        input.set_pts(Some(packet.presentation_time));
        input.set_dts(Some(packet.decode_time));
        match self.decoder.send_packet(&input) {
            Ok(()) => {}
            // The decoder is full: take its pictures, then send the packet again.
            Err(av::Error::Other { errno }) if errno == av::error::EAGAIN => {
                self.drain()?;
                self.decoder.send_packet(&input).map_err(failed)?;
            }
            Err(error) => return Err(failed(error)),
        }
        self.drain()
    }

    fn finish(&mut self) -> Result<(), DecodeError> {
        self.decoder.send_eof().map_err(failed)?;
        self.drain()
    }

    fn take_pictures(&mut self) -> Vec<Picture> {
        let mut pictures = std::mem::take(&mut self.pictures);
        pictures.sort_by_key(|picture| picture.presentation_time);
        pictures
    }
}

/// Copy the configuration record into the context's extradata, which libavcodec frees with it.
fn set_extradata(
    context: &mut av::codec::context::Context,
    config: &[u8],
) -> Result<(), DecodeError> {
    let size =
        i32::try_from(config.len()).map_err(|_| unsupported("configuration record too large"))?;
    // SAFETY: the context is owned here and not yet opened. The buffer is allocated with
    // libavcodec's allocator and padded as libavcodec requires, so `avcodec_free_context` frees it.
    unsafe {
        let raw = context.as_mut_ptr();
        let padded = config.len() + av::ffi::AV_INPUT_BUFFER_PADDING_SIZE as usize;
        let buffer = av::ffi::av_mallocz(padded).cast::<u8>();
        if buffer.is_null() {
            return Err(unsupported("out of memory for the configuration record"));
        }
        std::ptr::copy_nonoverlapping(config.as_ptr(), buffer, config.len());
        (*raw).extradata = buffer;
        (*raw).extradata_size = size;
    }
    Ok(())
}

/// A decoded frame as a picture: its planes packed row by row, one after another, with no padding.
fn picture(frame: &av::frame::Video) -> Result<Picture, DecodeError> {
    let (x, y, bit_depth): (u32, u32, u32) = match frame.format() {
        av::format::Pixel::YUV420P => (1, 1, 8),
        av::format::Pixel::YUV422P => (1, 0, 8),
        av::format::Pixel::YUV444P => (0, 0, 8),
        av::format::Pixel::YUV420P10LE => (1, 1, 10),
        av::format::Pixel::YUV422P10LE => (1, 0, 10),
        av::format::Pixel::YUV444P10LE => (0, 0, 10),
        other => {
            return Err(DecodeError::Failed {
                backend: BACKEND,
                reason: format!("pixel format {other:?} is not handled"),
            });
        }
    };
    let bytes_per_sample = if bit_depth > 8 { 2 } else { 1 };
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let (shift_x, shift_y) = (x as usize, y as usize);
    let chroma_width = (width + shift_x) >> shift_x;
    let chroma_height = (height + shift_y) >> shift_y;
    let sizes = [
        (width, height),
        (chroma_width, chroma_height),
        (chroma_width, chroma_height),
    ];

    let mut samples = Vec::new();
    for (index, &(plane_width, plane_height)) in sizes.iter().enumerate() {
        let stride = frame.stride(index);
        let data = frame.data(index);
        let row = plane_width * bytes_per_sample;
        for line in 0..plane_height {
            let start = line * stride;
            match data.get(start..start + row) {
                Some(bytes) => samples.extend_from_slice(bytes),
                None => samples.resize(samples.len() + row, 0),
            }
        }
    }

    Ok(Picture {
        width: frame.width(),
        height: frame.height(),
        presentation_time: frame.pts().unwrap_or(0),
        pixel_format: layout_code(x, y, bit_depth),
        surface: Surface::Cpu(samples),
    })
}

fn unsupported(reason: &str) -> DecodeError {
    DecodeError::Unsupported {
        backend: BACKEND,
        reason: reason.into(),
    }
}

fn failed(error: av::Error) -> DecodeError {
    DecodeError::Failed {
        backend: BACKEND,
        reason: error.to_string(),
    }
}
