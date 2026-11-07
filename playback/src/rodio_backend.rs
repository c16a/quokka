//! Audio playback: symphonia decodes, rodio mixes and sends the samples to the device.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player as Output, Source};

use crate::chain::{Chain, Shared};
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
    /// The device chosen by name, or `None` for the system default.
    device_name: Option<String>,
    /// Sample rate of the open output, in Hz.
    rate: u32,
    /// State shared with the audio thread for the current track. `None` when nothing is loaded.
    shared: Option<Arc<Shared>>,
    /// How many chained tracks have been reported by `take_started`.
    seen_started: u64,
    /// Playback speed, where 1 is normal.
    speed: f32,
    /// Crossfade length in seconds. Zero is off.
    fade_secs: f32,
}

struct Current {
    path: PathBuf,
    duration: Option<Duration>,
}

/// Open an output device. Tries the default device first, then every other output in turn.
/// rodio's own fallback skips devices that report no driver name (which includes all macOS
/// devices), so it can give up while a working output such as the built-in speakers exists.
pub(crate) fn open_output(rate: Option<u32>) -> Result<MixerDeviceSink> {
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
        match open_at(&device, rate) {
            Ok(sink) => return Ok(sink),
            Err(e) => last_error = e,
        }
    }
    Err(PlaybackError::Backend(last_error))
}

/// The name of an output device, as shown to the user.
fn device_name(device: &rodio::cpal::Device) -> Option<String> {
    device.description().ok().map(|d| d.name().to_string())
}

/// Names of the audio outputs the system offers, with the default one first.
pub fn output_device_names() -> Vec<String> {
    let host = rodio::cpal::default_host();
    let mut names: Vec<String> = Vec::new();
    if let Some(name) = host.default_output_device().and_then(|d| device_name(&d)) {
        names.push(name);
    }
    if let Ok(all) = host.output_devices() {
        for device in all {
            if let Some(name) = device_name(&device)
                && !names.contains(&name)
            {
                names.push(name);
            }
        }
    }
    names
}

/// Open `device`, at `rate` when one is asked for. A device that refuses that rate opens at its
/// own rate instead, so the track still plays (rodio then resamples it).
fn open_at(
    device: &rodio::cpal::Device,
    rate: Option<u32>,
) -> std::result::Result<MixerDeviceSink, String> {
    if let Some(rate) = rate.and_then(std::num::NonZero::<u32>::new)
        && let Ok(mut sink) = DeviceSinkBuilder::from_device(device.clone())
            .and_then(|b| b.with_sample_rate(rate).open_stream())
    {
        // Quitting the app is the normal way to stop audio, so rodio's warning about it is noise.
        sink.log_on_drop(false);
        return Ok(sink);
    }
    let mut sink = DeviceSinkBuilder::from_device(device.clone())
        .and_then(|b| b.open_stream())
        .map_err(|e| e.to_string())?;
    sink.log_on_drop(false);
    Ok(sink)
}

/// Open an output: the named device, or the default one first, at `rate` when one is asked for.
fn open_device(name: Option<&str>, rate: Option<u32>) -> Result<MixerDeviceSink> {
    match name {
        Some(name) => open_named(name, rate),
        None => open_output(rate),
    }
}

/// Open the output with exactly this name.
fn open_named(name: &str, rate: Option<u32>) -> Result<MixerDeviceSink> {
    let host = rodio::cpal::default_host();
    let device = host
        .output_devices()
        .ok()
        .and_then(|mut all| all.find(|d| device_name(d).as_deref() == Some(name)))
        .ok_or_else(|| PlaybackError::Backend(format!("{name} is not available")))?;
    open_at(&device, rate).map_err(PlaybackError::Backend)
}

impl RodioPlayer {
    /// Open an audio output device (see [`open_output`]).
    pub fn new() -> Result<Self> {
        Self::open(None)
    }

    /// Open the named output device, or the system default when `device` is `None`.
    pub fn open(device: Option<&str>) -> Result<Self> {
        let sink = open_device(device, None)?;
        let output = Output::connect_new(sink.mixer());
        Ok(Self {
            rate: sink.config().sample_rate().get(),
            _device: sink,
            output,
            device_name: device.map(str::to_owned),
            current: None,
            error: ErrorSlot::default(),
            shared: None,
            seen_started: 0,
            speed: 1.0,
            fade_secs: 0.0,
        })
    }

    /// The sample rate the output runs at, in Hz.
    pub fn output_rate(&self) -> u32 {
        self.rate
    }

    /// Reopen the output at `rate`, so the track is not resampled. If the device can't run at
    /// that rate, the output stays as it is and rodio resamples the track.
    fn match_rate(&mut self, rate: u32) {
        if rate == self.rate {
            return;
        }
        if let Ok(sink) = open_device(self.device_name.as_deref(), Some(rate)) {
            self.rate = sink.config().sample_rate().get();
            // The old output goes first, so nothing plays through a dropped device.
            self.output = Output::connect_new(sink.mixer());
            self.output.set_speed(self.speed);
            self._device = sink;
        }
    }

    /// The state for a new track at `rate`, with the crossfade length in force.
    fn new_shared(&self, rate: u32) -> Arc<Shared> {
        let shared = Shared::new(rate);
        shared.set_fade((self.fade_secs * rate as f32).round() as u64);
        Arc::new(shared)
    }

    /// Play a live stream. It has no path or length, so the duration is unknown.
    pub fn load_stream(&mut self, source: SymphoniaSource) -> Result<()> {
        if let Ok(mut slot) = self.error.lock() {
            *slot = None;
        }
        let rate = source.sample_rate().get();
        self.match_rate(rate);
        let shared = self.new_shared(rate);
        self.output.clear();
        self.output.pause();
        self.output.append(Chain::new(source, Arc::clone(&shared)));
        self.shared = Some(shared);
        self.seen_started = 0;
        self.current = Some(Current {
            path: PathBuf::new(),
            duration: None,
        });
        Ok(())
    }

    /// Decode `path` and queue it on a paused output.
    fn queue(&mut self, path: &Path) -> Result<Option<Duration>> {
        if let Ok(mut slot) = self.error.lock() {
            *slot = None;
        }
        let source = SymphoniaSource::open(path, Arc::clone(&self.error))
            .map_err(|_| PlaybackError::Open(path.display().to_string()))?;
        let duration = source.total_duration();
        let rate = source.sample_rate().get();
        self.match_rate(rate);
        let shared = self.new_shared(rate);
        self.output.clear();
        self.output.pause();
        self.output.append(Chain::new(source, Arc::clone(&shared)));
        self.shared = Some(shared);
        self.seen_started = 0;
        Ok(duration)
    }

    /// Open `path` now, so it plays straight after the current track. Replaces any track armed
    /// before. Nothing is opened on the audio thread.
    pub fn arm_next(&mut self, path: &Path) -> Result<()> {
        let Some(shared) = self.shared.clone() else {
            return Ok(());
        };
        let source = SymphoniaSource::open(path, Arc::clone(&self.error))
            .map_err(|_| PlaybackError::Open(path.display().to_string()))?;
        let duration = source.total_duration();
        shared.arm(Some((path.to_path_buf(), duration, source)));
        Ok(())
    }

    /// Forget the armed track, so the current one ends the output as usual.
    pub fn disarm_next(&mut self) {
        if let Some(shared) = &self.shared {
            shared.arm(None);
        }
    }

    /// The path of a track the output has moved on to since the last call, if any. Its duration
    /// becomes the current duration.
    pub fn take_started(&mut self) -> Option<PathBuf> {
        let shared = self.shared.as_ref()?;
        let count = shared.started_count();
        if count == self.seen_started {
            return None;
        }
        self.seen_started = count;
        let (path, duration) = shared.started()?;
        self.current = Some(Current {
            path: path.clone(),
            duration,
        });
        Some(path)
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
        self.shared = None;
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
        self.current.as_ref().map(|_| match &self.shared {
            Some(shared) => shared.position(),
            None => self.output.get_pos(),
        })
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

    fn set_speed(&mut self, speed: f32) -> Result<()> {
        self.speed = speed;
        self.output.set_speed(speed);
        Ok(())
    }

    fn set_crossfade(&mut self, secs: f32) -> Result<()> {
        self.fade_secs = secs.max(0.0);
        if let Some(shared) = &self.shared {
            shared.set_fade((self.fade_secs * self.rate as f32).round() as u64);
        }
        Ok(())
    }

    fn last_error(&self) -> Option<String> {
        self.error.lock().ok().and_then(|slot| slot.clone())
    }
}
