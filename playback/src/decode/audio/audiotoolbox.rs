//! AudioToolbox, Apple's operating-system audio decoder. It decodes AAC, AC-3 and E-AC-3 on the
//! Macs this runs on, and calls it through the C interface of the framework.

use std::ffi::c_void;
use std::ptr;

use player_core::video::AudioCodec;

use super::{AudioDecoder, AudioStream};
use crate::decode::DecodeError;

const BACKEND: &str = "AudioToolbox";

type OSStatus = i32;
type Converter = *mut c_void;

const FORMAT_LINEAR_PCM: u32 = u32::from_be_bytes(*b"lpcm");
const FORMAT_AAC: u32 = u32::from_be_bytes(*b"aac ");
const FORMAT_AC3: u32 = u32::from_be_bytes(*b"ac-3");
const FORMAT_EAC3: u32 = u32::from_be_bytes(*b"ec-3");
const FORMAT_MP3: u32 = u32::from_be_bytes(*b"mp3 ");
const FORMAT_FLAC: u32 = u32::from_be_bytes(*b"flac");
const FORMAT_ALAC: u32 = u32::from_be_bytes(*b"alac");
/// Float samples, packed, as the stream's own sample rate.
const FLAG_FLOAT_PACKED: u32 = 1 | 8;
/// The property that gives the converter the codec's configuration.
const PROPERTY_MAGIC_COOKIE: u32 = u32::from_be_bytes(*b"dmgc");
/// What the input callback returns when it has no packet to give. Returning success with no packet
/// would tell the converter the stream has ended, so this distinct status says "send more later".
const NEED_INPUT: OSStatus = 0x6E65_6564;
/// Frames asked for per call to the converter.
const FRAMES_PER_CALL: u32 = 4096;

/// An `AudioStreamBasicDescription`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Format {
    sample_rate: f64,
    format_id: u32,
    format_flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

/// An `AudioStreamPacketDescription`: where a compressed frame sits, and how big it is.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct PacketDescription {
    start_offset: i64,
    variable_frames: u32,
    data_size: u32,
}

/// An `AudioBufferList` with one buffer. The buffer's fields sit after eight bytes of padding,
/// because the data pointer that follows them needs eight-byte alignment.
#[repr(C)]
struct BufferList {
    buffers: u32,
    _padding: u32,
    channels: u32,
    size: u32,
    data: *mut c_void,
}

/// The callback the converter asks for input through.
type InputProc = extern "C" fn(
    converter: Converter,
    packets: *mut u32,
    buffers: *mut BufferList,
    descriptions: *mut *mut PacketDescription,
    user: *mut c_void,
) -> OSStatus;

#[link(name = "AudioToolbox", kind = "framework")]
unsafe extern "C" {
    fn AudioConverterNew(
        source: *const Format,
        destination: *const Format,
        converter_out: *mut Converter,
    ) -> OSStatus;
    fn AudioConverterDispose(converter: Converter) -> OSStatus;
    fn AudioConverterSetProperty(
        converter: Converter,
        property: u32,
        size: u32,
        data: *const c_void,
    ) -> OSStatus;
    fn AudioConverterReset(converter: Converter) -> OSStatus;
    fn AudioConverterFillComplexBuffer(
        converter: Converter,
        input: InputProc,
        user: *mut c_void,
        packets: *mut u32,
        output: *mut BufferList,
        descriptions: *mut PacketDescription,
    ) -> OSStatus;
}

/// What the input callback hands the converter: at most one packet, held until it has been read.
struct Input {
    packet: Vec<u8>,
    description: PacketDescription,
    channels: u32,
    pending: bool,
}

/// The channel order AC-3 and E-AC-3 decoders give out for 5.1, as positions in the order they
/// are written: L, C, R, Ls, Rs, LFE. Each output channel is taken from one of these positions, so
/// the result is in WAVE order, L, R, C, LFE, Ls, Rs, as the rest of the player expects.
const AC3_51_TO_WAVE: [usize; 6] = [0, 2, 1, 5, 3, 4];

/// The channel order AAC gives for 5.1, which is MPEG's: C, L, R, Ls, Rs, LFE. Taken the same way
/// as the AC-3 order above.
const AAC_51_TO_WAVE: [usize; 6] = [1, 2, 0, 5, 3, 4];

// SAFETY: a converter is used by one thread at a time, and the decoder is moved to the thread that
// owns its stream once, when the stream is opened. Nothing in it is shared.
unsafe impl Send for AudioToolbox {}

/// An AudioToolbox converter for one stream.
pub struct AudioToolbox {
    converter: Converter,
    input: Box<Input>,
    channels: u32,
    /// The channel order to reorder into, when the codec's order differs from the player's.
    reorder: Option<[usize; 6]>,
}

extern "C" fn supply(
    _converter: Converter,
    packets: *mut u32,
    buffers: *mut BufferList,
    descriptions: *mut *mut PacketDescription,
    user: *mut c_void,
) -> OSStatus {
    // SAFETY: `user` is the decoder's `Input`, which outlives the conversion call. The converter
    // gives valid pointers for the buffer list, the packet count and the description.
    unsafe {
        let input = &mut *(user as *mut Input);
        if !input.pending {
            *packets = 0;
            *descriptions = ptr::null_mut();
            return NEED_INPUT;
        }
        input.pending = false;
        input.description = PacketDescription {
            start_offset: 0,
            variable_frames: 0,
            data_size: input.packet.len() as u32,
        };
        *packets = 1;
        *descriptions = &mut input.description;
        let list = &mut *buffers;
        list.buffers = 1;
        list.channels = input.channels;
        list.size = input.packet.len() as u32;
        list.data = input.packet.as_mut_ptr().cast();
    }
    0
}

impl AudioToolbox {
    /// Open a converter for the stream. Fails when AudioToolbox does not decode the codec, or does
    /// not take the stream's configuration.
    pub fn open(stream: &AudioStream) -> Result<Self, DecodeError> {
        let (format_id, frames_per_packet) = match stream.codec {
            AudioCodec::Aac => (FORMAT_AAC, 1024),
            AudioCodec::Ac3 => (FORMAT_AC3, 1536),
            AudioCodec::Eac3 => (FORMAT_EAC3, 1536),
            AudioCodec::Mp3 => (FORMAT_MP3, 1152),
            AudioCodec::Flac => (FORMAT_FLAC, 0),
            AudioCodec::Alac => (FORMAT_ALAC, 4096),
        };
        let channels = stream.channels;
        let source = Format {
            sample_rate: f64::from(stream.sample_rate),
            format_id,
            frames_per_packet,
            channels_per_frame: channels,
            ..Format::default()
        };
        let destination = Format {
            sample_rate: f64::from(stream.sample_rate),
            format_id: FORMAT_LINEAR_PCM,
            format_flags: FLAG_FLOAT_PACKED,
            bytes_per_packet: channels * 4,
            frames_per_packet: 1,
            bytes_per_frame: channels * 4,
            channels_per_frame: channels,
            bits_per_channel: 32,
            reserved: 0,
        };
        let mut converter = ptr::null_mut();
        // SAFETY: both formats and the output pointer are valid for the call.
        let status = unsafe { AudioConverterNew(&source, &destination, &mut converter) };
        if status != 0 || converter.is_null() {
            return Err(refused(format!("AudioConverterNew returned {status}")));
        }
        // Only the MPEG-family orders are known; other 5.1 streams are left as the decoder gives them.
        let reorder = match (channels, stream.codec) {
            (6, AudioCodec::Aac) => Some(AAC_51_TO_WAVE),
            (6, AudioCodec::Ac3 | AudioCodec::Eac3) => Some(AC3_51_TO_WAVE),
            _ => None,
        };
        let decoder = AudioToolbox {
            converter,
            input: Box::new(Input {
                packet: Vec::new(),
                description: PacketDescription::default(),
                channels,
                pending: false,
            }),
            channels,
            reorder,
        };
        // AAC takes its MPEG-4 `esds` descriptors as the magic cookie, ALAC and FLAC their own
        // records. MP3, AC-3 and E-AC-3 need none.
        let cookie = match stream.codec {
            AudioCodec::Aac => Some(esds(&stream.config)),
            AudioCodec::Alac | AudioCodec::Flac => Some(stream.config.clone()),
            AudioCodec::Ac3 | AudioCodec::Eac3 | AudioCodec::Mp3 => None,
        };
        if let Some(cookie) = cookie.filter(|cookie| !cookie.is_empty()) {
            // SAFETY: the converter is valid, and the cookie is valid for the call.
            let status = unsafe {
                AudioConverterSetProperty(
                    decoder.converter,
                    PROPERTY_MAGIC_COOKIE,
                    cookie.len() as u32,
                    cookie.as_ptr().cast(),
                )
            };
            if status != 0 {
                return Err(refused(format!(
                    "the codec configuration was refused ({status})"
                )));
            }
        }
        Ok(decoder)
    }

    /// Run the converter until it has no more output for the input it holds.
    fn fill(&mut self) -> Result<Vec<f32>, DecodeError> {
        let mut samples = Vec::new();
        loop {
            let mut buffer = vec![0f32; (FRAMES_PER_CALL * self.channels) as usize];
            let mut list = BufferList {
                buffers: 1,
                _padding: 0,
                channels: self.channels,
                size: (buffer.len() * 4) as u32,
                data: buffer.as_mut_ptr().cast(),
            };
            let mut frames = FRAMES_PER_CALL;
            let user: *mut Input = &mut *self.input;
            // SAFETY: the converter, the input (owned by this decoder), and the output buffer are
            // all valid for the call.
            let status = unsafe {
                AudioConverterFillComplexBuffer(
                    self.converter,
                    supply,
                    user.cast(),
                    &mut frames,
                    &mut list,
                    ptr::null_mut(),
                )
            };
            if status == NEED_INPUT {
                // The converter has used the packet it had; the frames it made are below.
                if frames > 0 {
                    self.append(&mut samples, &buffer[..(frames * self.channels) as usize]);
                }
                return Ok(samples);
            }
            if status != 0 {
                return Err(DecodeError::Backend {
                    backend: BACKEND,
                    status,
                });
            }
            if frames == 0 {
                return Ok(samples);
            }
            self.append(&mut samples, &buffer[..(frames * self.channels) as usize]);
        }
    }

    /// Append decoded frames, reordering their channels where the codec's order differs.
    fn append(&self, samples: &mut Vec<f32>, frames: &[f32]) {
        match self.reorder {
            None => samples.extend_from_slice(frames),
            Some(order) => {
                for frame in frames.chunks(6) {
                    samples.extend(order.iter().map(|&position| frame[position]));
                }
            }
        }
    }
}

impl AudioDecoder for AudioToolbox {
    fn name(&self) -> &'static str {
        BACKEND
    }

    fn decode(&mut self, packet: &[u8]) -> Result<Vec<f32>, DecodeError> {
        self.input.packet = packet.to_vec();
        self.input.pending = true;
        self.fill()
    }

    fn finish(&mut self) -> Result<Vec<f32>, DecodeError> {
        self.input.pending = false;
        self.fill()
    }

    fn reset(&mut self) {
        self.input.pending = false;
        self.input.packet.clear();
        // SAFETY: the converter is valid until `Drop`.
        unsafe { AudioConverterReset(self.converter) };
    }
}

impl Drop for AudioToolbox {
    fn drop(&mut self) {
        // SAFETY: the converter was created in `open` and is disposed once, here.
        unsafe { AudioConverterDispose(self.converter) };
    }
}

/// The MPEG-4 `esds` descriptors that wrap an AAC `AudioSpecificConfig`: an ES descriptor, a
/// decoder configuration for AAC audio, the specific information, and an SL configuration.
fn esds(config: &[u8]) -> Vec<u8> {
    let mut specific = vec![0x05, config.len() as u8];
    specific.extend_from_slice(config);
    // Object type 0x40 (MPEG-4 audio), stream type 0x15 (audio), then the buffer size and bit rates.
    let mut decoder = vec![0x04, (13 + specific.len()) as u8, 0x40, 0x15, 0, 0, 0];
    decoder.extend_from_slice(&[0; 8]);
    decoder.extend_from_slice(&specific);
    let mut es = vec![0x03, (3 + decoder.len() + 3) as u8, 0, 0, 0];
    es.extend_from_slice(&decoder);
    es.extend_from_slice(&[0x06, 0x01, 0x02]);
    es
}

fn refused(reason: String) -> DecodeError {
    DecodeError::Unsupported {
        backend: BACKEND,
        reason,
    }
}
