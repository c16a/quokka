//! Plays a file's video frames in presentation order, for display. A worker thread reads the
//! file's samples, decodes them, puts the pictures in order, and converts each to RGBA. The window
//! takes frames as their times come up.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use quokka_core::video::{self, Chroma, Colour};

use super::frames::Yuv;
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
    /// The picture's Y'CbCr planes. They are converted to RGB by whoever displays them.
    pub image: Yuv,
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

/// A video file's samples and format, read once. Opening it at a position from here does not read
/// the file's tables again, which is most of what a seek used to cost.
pub struct VideoSource {
    path: PathBuf,
    track: Arc<video::SampleTrack>,
    bit_depth: u8,
    chroma: Chroma,
    colour: Colour,
    duration: Option<std::time::Duration>,
}

impl VideoSource {
    /// Read the first video track of `path`.
    pub fn read(path: &Path) -> Result<Self, FrameError> {
        let track = quokka_core::video::read_samples(path)
            .map_err(|error| FrameError::Unreadable(error.to_string()))?
            .ok_or_else(|| FrameError::Unreadable("no video track in this file".into()))?;
        let format = video::inspect(path).unwrap_or_default();
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
        Ok(VideoSource {
            path: path.to_path_buf(),
            track: Arc::new(track),
            bit_depth: format.bit_depth.unwrap_or(8),
            chroma: format.chroma.unwrap_or(Chroma::Yuv420),
            colour: format.colour,
            duration,
        })
    }

    /// The length of the video, when the file states its frames.
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.duration
    }

    /// Start decoding at the last keyframe at or before `start`, in seconds.
    pub fn open_at(&self, start: f64) -> Result<FramePlayer, FrameError> {
        let track = Arc::clone(&self.track);
        let stream = StreamConfig {
            codec: track.codec,
            width: track.width,
            height: track.height,
            config: track.config.clone(),
            timescale: track.timescale,
            bit_depth: self.bit_depth,
            chroma: self.chroma,
            colour: self.colour,
        };
        let frame_count = track.samples.len();
        let target = start * track.timescale as f64;
        let first = track
            .samples
            .iter()
            .rposition(|sample| sample.sync && (sample.presentation_time as f64) <= target)
            .unwrap_or(0);
        let (sender, messages) = mpsc::sync_channel(QUEUE_DEPTH);
        let path = self.path.clone();
        thread::spawn(move || run(&path, track, stream, sender, first));

        match messages.recv() {
            Ok(Message::Ready) => Ok(FramePlayer {
                messages,
                frame_count,
                duration: self.duration,
                finished: false,
            }),
            Ok(Message::Failed(reason)) => Err(FrameError::NoDecoder(reason)),
            _ => Err(FrameError::NoDecoder("the decoder stopped".into())),
        }
    }
}

impl FramePlayer {
    /// Start reading and decoding the first video track of `path`. Returns once a decoder has
    /// accepted the stream, or with the reason it did not.
    pub fn open(path: &Path) -> Result<Self, FrameError> {
        Self::open_at(path, 0.0)
    }

    /// As `open`, but starting at the last keyframe at or before `start`, in seconds. Frames
    /// before `start` still come out, so the last one at or before it is the one to show.
    pub fn open_at(path: &Path, start: f64) -> Result<Self, FrameError> {
        VideoSource::read(path)?.open_at(start)
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
fn run(
    path: &Path,
    track: Arc<video::SampleTrack>,
    stream: StreamConfig,
    sender: SyncSender<Message>,
    first: usize,
) {
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
    let colour = stream.colour;
    let mut pending: VecDeque<Picture> = VecDeque::new();
    let mut data: Vec<u8> = Vec::new();
    for sample in &track.samples[first..] {
        data.resize(sample.size as usize, 0);
        if file.seek(SeekFrom::Start(sample.offset)).is_err() || file.read_exact(&mut data).is_err()
        {
            let _ = sender.send(Message::Failed("could not read a frame".into()));
            return;
        }
        let packet = Packet {
            data: std::mem::take(&mut data),
            decode_time: sample.decode_time,
            presentation_time: sample.presentation_time,
            sync: sample.sync,
        };
        let decoded = decoder.decode(&packet);
        data = packet.data;
        if let Err(error) = decoded {
            let _ = sender.send(Message::Failed(error.to_string()));
            return;
        }
        for picture in decoder.take_pictures() {
            insert_in_order(&mut pending, picture);
        }
        while pending.len() > REORDER_DEPTH {
            if let Some(picture) = pending.pop_front()
                && !emit(picture, timescale, &sender, colour)
            {
                return;
            }
        }
    }
    if let Err(error) = decoder.finish() {
        let _ = sender.send(Message::Failed(error.to_string()));
        return;
    }
    for picture in decoder.take_pictures() {
        insert_in_order(&mut pending, picture);
    }
    for picture in pending {
        if !emit(picture, timescale, &sender, colour) {
            return;
        }
    }
    let _ = sender.send(Message::End);
}

/// Add a picture to the queue, after any picture with the same time, so that equal times keep the
/// order they were decoded in.
fn insert_in_order(pending: &mut VecDeque<Picture>, picture: Picture) {
    let position =
        pending.partition_point(|queued| queued.presentation_time <= picture.presentation_time);
    pending.insert(position, picture);
}

/// Convert one picture and send it. `false` means the window has gone and the worker should stop.
fn emit(picture: Picture, timescale: f64, sender: &SyncSender<Message>, colour: Colour) -> bool {
    let time = picture.presentation_time as f64 / timescale;
    let Some(image) = Yuv::from_picture(picture, colour) else {
        return true;
    };
    sender
        .send(Message::Frame(VideoFrame { time, image }))
        .is_ok()
}
