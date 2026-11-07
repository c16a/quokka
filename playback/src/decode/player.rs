//! Plays a file's video frames in presentation order, for display. A worker thread reads the
//! file's samples, decodes them, puts the pictures in order, and converts each to RGBA. The window
//! takes frames as their times come up.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use player_core::video::{self, Chroma};

use super::frames::{Rgba, to_rgba};
use super::{DecodeError, Packet, Picture, StreamConfig};

/// Pictures held back while later ones decode, so that frames out of decode order come out in
/// presentation order. Reordering in practice spans far fewer frames than this.
const REORDER_DEPTH: usize = 8;

/// Converted frames waiting for the window.
const QUEUE_DEPTH: usize = 4;

/// One frame ready to show.
pub struct VideoFrame {
    /// When the frame is shown, in seconds from the start.
    pub time: f64,
    pub image: Rgba,
}

/// Why playback could not start.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    /// The file could not be read.
    #[error("could not read the video: {0}")]
    Unreadable(String),
    /// No decoder handles the stream.
    #[error("no decoder plays this video: {0}")]
    NoDecoder(String),
}

enum Message {
    Ready,
    Frame(VideoFrame),
    Failed(String),
    End,
}

/// A running frame source for one video file.
pub struct FramePlayer {
    messages: Receiver<Message>,
    /// Frames in the file's video track, when the container states them.
    pub frame_count: usize,
    /// The length of the video, from its last frame and the gap before it.
    duration: Option<std::time::Duration>,
    finished: bool,
}

impl FramePlayer {
    /// Start reading and decoding the first video track of `path`. Returns once a decoder has
    /// accepted the stream, or with the reason it did not.
    pub fn open(path: &Path) -> Result<Self, FrameError> {
        let track = player_core::video::read_samples(path)
            .map_err(|error| FrameError::Unreadable(error.to_string()))?
            .ok_or_else(|| FrameError::Unreadable("no video track in this file".into()))?;
        let format = video::inspect(path).unwrap_or_default();
        let stream = StreamConfig {
            codec: track.codec,
            width: track.width,
            height: track.height,
            config: track.config.clone(),
            timescale: track.timescale,
            bit_depth: format.bit_depth.unwrap_or(8),
            chroma: format.chroma.unwrap_or(Chroma::Yuv420),
        };
        let frame_count = track.samples.len();
        let duration = track.samples.last().and_then(|last| {
            let times: Vec<i64> = track
                .samples
                .iter()
                .rev()
                .take(2)
                .map(|s| s.presentation_time)
                .collect();
            let step = if times.len() == 2 {
                times[0] - times[1]
            } else {
                0
            };
            let end = last.presentation_time + step.max(0);
            u64::try_from(end).ok().map(|ticks| {
                std::time::Duration::from_secs_f64(ticks as f64 / track.timescale as f64)
            })
        });
        let (sender, messages) = mpsc::sync_channel(QUEUE_DEPTH);
        let path = path.to_path_buf();
        thread::spawn(move || run(&path, track, stream, sender));

        match messages.recv() {
            Ok(Message::Ready) => Ok(FramePlayer {
                messages,
                frame_count,
                duration,
                finished: false,
            }),
            Ok(Message::Failed(reason)) => Err(FrameError::NoDecoder(reason)),
            _ => Err(FrameError::NoDecoder("the decoder stopped".into())),
        }
    }

    /// The next frame if one is ready, without waiting. `Ok(None)` means none is ready yet, or the
    /// file has ended; `finished` tells the two apart.
    pub fn next_frame(&mut self) -> Result<Option<VideoFrame>, DecodeError> {
        if self.finished {
            return Ok(None);
        }
        match self.messages.try_recv() {
            Ok(Message::Frame(frame)) => Ok(Some(frame)),
            Ok(Message::End) | Err(mpsc::TryRecvError::Disconnected) => {
                self.finished = true;
                Ok(None)
            }
            Ok(Message::Failed(reason)) => {
                self.finished = true;
                Err(DecodeError::Failed {
                    backend: "player",
                    reason,
                })
            }
            Ok(Message::Ready) | Err(mpsc::TryRecvError::Empty) => Ok(None),
        }
    }

    /// Whether every frame has been taken.
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// The length of the video, when the file states its frames.
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.duration
    }
}

/// The worker: decode every sample, then send the frames in order. The decoder is opened here,
/// since decoders are not sent between threads.
fn run(path: &Path, track: video::SampleTrack, stream: StreamConfig, sender: SyncSender<Message>) {
    let mut decoder = match super::open(&stream) {
        Ok(decoder) => decoder,
        Err(error) => {
            let _ = sender.send(Message::Failed(error.to_string()));
            return;
        }
    };
    if sender.send(Message::Ready).is_err() {
        return;
    }
    let Ok(mut file) = File::open(path) else {
        let _ = sender.send(Message::Failed("could not reopen the file".into()));
        return;
    };
    let timescale = track.timescale as f64;
    let mut pending: Vec<Picture> = Vec::new();
    for sample in &track.samples {
        let mut data = vec![0u8; sample.size as usize];
        if file.seek(SeekFrom::Start(sample.offset)).is_err() || file.read_exact(&mut data).is_err()
        {
            let _ = sender.send(Message::Failed("could not read a frame".into()));
            return;
        }
        let packet = Packet {
            data,
            decode_time: sample.decode_time,
            presentation_time: sample.presentation_time,
            sync: sample.sync,
        };
        if let Err(error) = decoder.decode(&packet) {
            let _ = sender.send(Message::Failed(error.to_string()));
            return;
        }
        pending.extend(decoder.take_pictures());
        pending.sort_by_key(|picture| picture.presentation_time);
        while pending.len() > REORDER_DEPTH {
            if !emit(pending.remove(0), timescale, &sender) {
                return;
            }
        }
    }
    if let Err(error) = decoder.finish() {
        let _ = sender.send(Message::Failed(error.to_string()));
        return;
    }
    pending.extend(decoder.take_pictures());
    pending.sort_by_key(|picture| picture.presentation_time);
    for picture in pending {
        if !emit(picture, timescale, &sender) {
            return;
        }
    }
    let _ = sender.send(Message::End);
}

/// Convert one picture and send it. `false` means the window has gone and the worker should stop.
fn emit(picture: Picture, timescale: f64, sender: &SyncSender<Message>) -> bool {
    let time = picture.presentation_time as f64 / timescale;
    let Some(image) = to_rgba(&picture) else {
        return true;
    };
    sender
        .send(Message::Frame(VideoFrame { time, image }))
        .is_ok()
}
