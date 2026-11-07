//! The video window: mpv draws the picture in its own window, and our controls live in a borderless
//! child window attached under it. AppKit keeps a child window with its parent, so the controls
//! move and resize with the video. macOS only; other platforms use mpv's own controls.

use std::cell::{Cell, OnceCell, RefCell};
use std::path::Path;
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSButton, NSColor, NSImage, NSSlider, NSTextField, NSView,
    NSWindow, NSWindowOrderingMode, NSWindowStyleMask,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSTimer};

use crate::mpv_backend::{MpvHandle, MpvPlayer, Status};
use crate::{Player, PlayerState, Result};

/// The height of the control bar, which sits under the video.
const BAR: f64 = 52.0;
/// The control bar's width until the video's width is known.
const WIDTH: f64 = 960.0;
/// How often the controls are updated from playback, in seconds.
const TICK: f64 = 0.1;
/// How far above the bottom of a full-screen video the control bar sits.
const FULLSCREEN_INSET: f64 = 24.0;

/// What the controls need: the player, and the controls themselves.
pub struct Ivars {
    handle: MpvHandle,
    window: OnceCell<Retained<NSWindow>>,
    close: OnceCell<Retained<NSButton>>,
    play: OnceCell<Retained<NSButton>>,
    seek: OnceCell<Retained<NSSlider>>,
    volume: OnceCell<Retained<NSSlider>>,
    elapsed: OnceCell<Retained<NSTextField>>,
    remaining: OnceCell<Retained<NSTextField>>,
    /// Set by `begin_tracking`: the next visible window that is not one of these is the video's.
    tracking: Cell<bool>,
    known: RefCell<Vec<usize>>,
    /// The video window the controls are attached to, while it is open.
    parent: RefCell<Option<Retained<NSWindow>>>,
}

define_class!(
    /// Receives the controls' actions, and keeps the control bar with the video window.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    pub struct PlayerControls;

    unsafe impl NSObjectProtocol for PlayerControls {}

    impl PlayerControls {
        #[unsafe(method(playPause:))]
        fn play_pause(&self, _sender: Option<&AnyObject>) {
            let handle = &self.ivars().handle;
            if handle.status().state == PlayerState::Playing {
                handle.pause();
            } else {
                handle.play();
            }
            self.refresh();
        }

        #[unsafe(method(closeVideo:))]
        fn close_video(&self, _sender: Option<&AnyObject>) {
            self.ivars().handle.stop();
            self.detach();
        }

        #[unsafe(method(seekChanged:))]
        fn seek_changed(&self, _sender: Option<&AnyObject>) {
            let ivars = self.ivars();
            let Some(seek) = ivars.seek.get() else {
                return;
            };
            if let Some(length) = ivars.handle.status().duration {
                ivars.handle.seek(seek.doubleValue() * length);
            }
        }

        #[unsafe(method(volumeChanged:))]
        fn volume_changed(&self, _sender: Option<&AnyObject>) {
            let ivars = self.ivars();
            if let Some(volume) = ivars.volume.get() {
                ivars.handle.set_volume(volume.doubleValue() as f32);
            }
        }

        #[unsafe(method(tick:))]
        fn tick(&self, _timer: Option<&AnyObject>) {
            self.sync();
        }
    }
);

impl PlayerControls {
    fn new(mtm: MainThreadMarker, handle: MpvHandle) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars {
            handle,
            window: OnceCell::new(),
            close: OnceCell::new(),
            play: OnceCell::new(),
            seek: OnceCell::new(),
            volume: OnceCell::new(),
            elapsed: OnceCell::new(),
            remaining: OnceCell::new(),
            tracking: Cell::new(false),
            known: RefCell::new(Vec::new()),
            parent: RefCell::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }

    /// Note the windows that are already open, so the next new one can be recognised as the video.
    fn begin_tracking(&self) {
        let ivars = self.ivars();
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        let controls = ivars.window.get().map(|w| ptr(w));
        let known: Vec<usize> = app
            .windows()
            .iter()
            .filter(|w| w.isVisible() && Some(ptr(w)) != controls)
            .map(|w| ptr(&w))
            .collect();
        *ivars.known.borrow_mut() = known;
        ivars.tracking.set(true);
    }

    /// Attach the controls to the video window when it appears, keep them under it, and detach
    /// them when it closes. Also updates the controls from playback.
    fn sync(&self) {
        let ivars = self.ivars();
        let Some(controls) = ivars.window.get() else {
            return;
        };
        let mut parent = ivars.parent.borrow_mut();
        if let Some(window) = parent.as_ref()
            && !window.isVisible()
        {
            window.removeChildWindow(controls);
            controls.orderOut(None);
            *parent = None;
        }
        if parent.is_none()
            && ivars.tracking.get()
            && let Some(mtm) = MainThreadMarker::new()
        {
            let app = NSApplication::sharedApplication(mtm);
            let known = ivars.known.borrow();
            let candidate = app
                .windows()
                .iter()
                .find(|w| w.isVisible() && ptr(w) != ptr(controls) && !known.contains(&ptr(w)));
            if let Some(window) = candidate {
                unsafe { window.addChildWindow_ordered(controls, NSWindowOrderingMode::Above) };
                *parent = controls.parentWindow();
                ivars.tracking.set(false);
            }
        }
        if let Some(window) = parent.as_ref() {
            let frame = window.frame();
            // Full screen leaves no room below the video, so the bar goes over its bottom edge.
            let fullscreen = window.styleMask().contains(NSWindowStyleMask::FullScreen);
            let y = if fullscreen {
                frame.origin.y + FULLSCREEN_INSET
            } else {
                frame.origin.y - BAR
            };
            controls.setFrame_display(
                NSRect::new(
                    NSPoint::new(frame.origin.x, y),
                    NSSize::new(frame.size.width, BAR),
                ),
                true,
            );
            self.layout(frame.size.width);
        }
        drop(parent);
        self.refresh();
    }

    /// Take the controls off the video window and hide them.
    fn detach(&self) {
        let ivars = self.ivars();
        if let Some(controls) = ivars.window.get() {
            if let Some(window) = ivars.parent.borrow_mut().take() {
                window.removeChildWindow(controls);
            }
            controls.orderOut(None);
        }
        ivars.tracking.set(false);
    }

    /// Place the controls across the bar for a video `width` pixels wide.
    fn layout(&self, width: f64) {
        let ivars = self.ivars();
        if let Some(view) = ivars.close.get() {
            place(view, 12.0, 30.0);
        }
        if let Some(view) = ivars.play.get() {
            place(view, 48.0, 40.0);
        }
        if let Some(view) = ivars.elapsed.get() {
            place(view, 96.0, 56.0);
        }
        if let Some(view) = ivars.seek.get() {
            place(view, 158.0, (width - 420.0).max(120.0));
        }
        if let Some(view) = ivars.remaining.get() {
            place(view, width - 254.0, 70.0);
        }
        if let Some(view) = ivars.volume.get() {
            place(view, width - 170.0, 150.0);
        }
    }

    /// Show the playback position, length and play state on the controls.
    fn refresh(&self) {
        let ivars = self.ivars();
        let Status {
            state,
            position,
            duration,
        } = ivars.handle.status();
        let playing = state == PlayerState::Playing;
        if let Some(play) = ivars.play.get() {
            set_symbol(play, if playing { "pause.fill" } else { "play.fill" });
        }
        if let Some(seek) = ivars.seek.get() {
            let fraction = match (position, duration) {
                (Some(at), Some(length)) if length > 0.0 => (at / length).clamp(0.0, 1.0),
                _ => 0.0,
            };
            seek.setDoubleValue(fraction);
        }
        if let Some(elapsed) = ivars.elapsed.get() {
            elapsed.setStringValue(&NSString::from_str(&clock(position.unwrap_or(0.0))));
        }
        if let Some(remaining) = ivars.remaining.get() {
            let text = match (position, duration) {
                (Some(at), Some(length)) => format!("-{}", clock((length - at).max(0.0))),
                _ => String::new(),
            };
            remaining.setStringValue(&NSString::from_str(&text));
        }
    }
}

/// The address of a window, for comparing windows.
fn ptr(window: &NSWindow) -> usize {
    window as *const NSWindow as usize
}

/// Give a control its frame on the bar.
fn place(view: &NSView, x: f64, width: f64) {
    view.setFrame(NSRect::new(NSPoint::new(x, 14.0), NSSize::new(width, 24.0)));
}

/// Set a button's image to a macOS system symbol, such as `play.fill`.
fn set_symbol(button: &NSButton, name: &str) {
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        None,
    );
    button.setImage(image.as_deref());
}

/// `m:ss`, or `h:mm:ss` for an hour or more.
fn clock(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let (hours, minutes, seconds) = (total / 3600, (total / 60) % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// A player with its picture in its own window, and native controls attached under it.
pub struct VideoPlayer {
    // Dropped first, so mpv stops before the controls that refer to it go away.
    player: MpvPlayer,
    controls: Retained<PlayerControls>,
    _timer: Retained<NSTimer>,
}

impl VideoPlayer {
    /// Create the player and its control bar. Must run on the main thread, which the UI does.
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or_else(|| {
            crate::PlaybackError::Backend(
                "the video window must be created on the main thread".into(),
            )
        })?;

        let player = MpvPlayer::new()?;
        let controls = PlayerControls::new(mtm, player.handle());
        let target: &AnyObject = &controls;

        // The control bar: a borderless window that is attached under the video window.
        let frame = NSRect::new(NSPoint::new(200.0, 200.0), NSSize::new(WIDTH, BAR));
        let bar = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                frame,
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        // The window is reused for each video, so it must not be freed when hidden.
        unsafe { bar.setReleasedWhenClosed(false) };
        bar.setOpaque(true);
        bar.setBackgroundColor(Some(&NSColor::windowBackgroundColor()));
        let content = bar.contentView().expect("a window has a content view");

        let button = |symbol: &str, action| {
            let button = unsafe {
                NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(""),
                    Some(target),
                    Some(action),
                    mtm,
                )
            };
            set_symbol(&button, symbol);
            content.addSubview(&button);
            button
        };
        let close = button("xmark", sel!(closeVideo:));
        let play = button("play.fill", sel!(playPause:));

        let label = |text: &str| {
            let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
            content.addSubview(&label);
            label
        };
        let elapsed = label("0:00");
        let remaining = label("");

        let slider = |action, min, max, value| {
            let slider =
                unsafe { NSSlider::sliderWithTarget_action(Some(target), Some(action), mtm) };
            slider.setMinValue(min);
            slider.setMaxValue(max);
            slider.setDoubleValue(value);
            content.addSubview(&slider);
            slider
        };
        let seek = slider(sel!(seekChanged:), 0.0, 1.0, 0.0);
        let volume = slider(sel!(volumeChanged:), 0.0, 1.0, 1.0);

        let ivars = controls.ivars();
        let _ = ivars.window.set(bar);
        let _ = ivars.close.set(close);
        let _ = ivars.play.set(play);
        let _ = ivars.seek.set(seek);
        let _ = ivars.volume.set(volume);
        let _ = ivars.elapsed.set(elapsed);
        let _ = ivars.remaining.set(remaining);
        controls.layout(WIDTH);

        let timer = unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                TICK,
                target,
                sel!(tick:),
                None,
                true,
            )
        };

        Ok(Self {
            player,
            controls,
            _timer: timer,
        })
    }
}

impl Player for VideoPlayer {
    fn load(&mut self, path: &Path) -> Result<()> {
        self.controls.begin_tracking();
        self.player.load(path)
    }

    fn play(&mut self) -> Result<()> {
        self.player.play()
    }

    fn pause(&mut self) -> Result<()> {
        self.player.pause()
    }

    fn stop(&mut self) -> Result<()> {
        self.controls.detach();
        self.player.stop()
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        self.player.seek(position)
    }

    fn position(&self) -> Option<Duration> {
        self.player.position()
    }

    fn duration(&self) -> Option<Duration> {
        self.player.duration()
    }

    fn state(&self) -> PlayerState {
        self.player.state()
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.player.set_volume(volume)
    }

    fn set_speed(&mut self, speed: f32) -> Result<()> {
        self.player.set_speed(speed)
    }

    fn last_error(&self) -> Option<String> {
        self.player.last_error()
    }
}
