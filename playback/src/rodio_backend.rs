//! Audio playback: symphonia decodes, rodio mixes and sends the samples to the device.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player as Output, Source};

use crate::symphonia_source::{ErrorSlot, SymphoniaSource};
use crate::{PlaybackError, Player, PlayerState, Result};

/// `Player` that plays through the default audio output device.
pub struct RodioPlayer {
    /// Keeps the audio device open. Dropping it stops all output.
    _device: MixerDeviceSink,
    output: Output,
    current: Option<Current>,
    /// Filled in by the decoder if the current track stops early.
    error: ErrorSlot,
}

struct Current {
    path: PathBuf,
    duration: Option<Duration>,
}

/// Open an output device. Tries the default device first, then every other output in turn.
/// rodio's own fallback skips devices that report no driver name (which includes all macOS
/// devices), so it can give up while a working output such as the built-in speakers exists.
fn open_output() -> Result<MixerDeviceSink> {
    let host = rodio::cpal::default_host();
    let default = host.default_output_device();
    let default_name = default
        .as_ref()
        .and_then(|d| d.description().ok().map(|x| x.name().to_string()));
    let mut devices: Vec<_> = default.into_iter().collect();
    if let Ok(all) = host.output_devices() {
        devices.extend(
            all.filter(|d| d.description().ok().map(|x| x.name().to_string()) != default_name),
        );
    }

    let mut last_error = String::from("no audio output device found");
    for device in devices {
        match DeviceSinkBuilder::from_device(device).and_then(|b| b.open_stream()) {
            Ok(mut sink) => {
                // Quitting the app is the normal way to stop audio, so rodio's warning about it is noise.
                sink.log_on_drop(false);
                return Ok(sink);
            }
            Err(e) => last_error = e.to_string(),
        }
    }
    Err(PlaybackError::Backend(last_error))
}

impl RodioPlayer {
    /// Open an audio output device (see [`open_output`]).
    pub fn new() -> Result<Self> {
        let device = open_output()?;
        let output = Output::connect_new(device.mixer());
        Ok(Self {
            _device: device,
            output,
            current: None,
            error: ErrorSlot::default(),
        })
    }

    /// Decode `path` and queue it on a paused output.
    fn queue(&mut self, path: &Path) -> Result<Option<Duration>> {
        if let Ok(mut slot) = self.error.lock() {
            *slot = None;
        }
        let source = SymphoniaSource::open(path, Arc::clone(&self.error))
            .map_err(|_| PlaybackError::Open(path.display().to_string()))?;
        let duration = source.total_duration();
        self.output.clear();
        self.output.pause();
        self.output.append(source);
        Ok(duration)
    }
}

impl Player for RodioPlayer {
    fn load(&mut self, path: &Path) -> Result<()> {
        let duration = self.queue(path)?;
        self.current = Some(Current {
            path: path.to_path_buf(),
            duration,
        });
        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        // A finished track restarts from the beginning when played again.
        if self.state() == PlayerState::Ended
            && let Some(path) = self.current.as_ref().map(|c| c.path.clone())
        {
            self.queue(&path)?;
        }
        if self.current.is_none() {
            return Err(PlaybackError::NoMedia);
        }
        self.output.play();
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if self.current.is_none() {
            return Err(PlaybackError::NoMedia);
        }
        self.output.pause();
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.output.clear();
        self.output.pause();
        self.current = None;
        Ok(())
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        if self.current.is_none() {
            return Err(PlaybackError::NoMedia);
        }
        self.output
            .try_seek(position)
            .map_err(|e| PlaybackError::Backend(e.to_string()))
    }

    fn position(&self) -> Option<Duration> {
        self.current.as_ref().map(|_| self.output.get_pos())
    }

    fn duration(&self) -> Option<Duration> {
        self.current.as_ref().and_then(|c| c.duration)
    }

    fn state(&self) -> PlayerState {
        if self.current.is_none() {
            PlayerState::Idle
        } else if self.output.empty() {
            PlayerState::Ended
        } else if self.output.is_paused() {
            PlayerState::Paused
        } else {
            PlayerState::Playing
        }
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.output.set_volume(volume);
        Ok(())
    }

    fn last_error(&self) -> Option<String> {
        self.error.lock().ok().and_then(|slot| slot.clone())
    }
}
