//! Opens the video window on a file, plays it for about ten seconds, then exits. Use it to see the
//! controls without the rest of the app.
//!
//! Usage: `cargo run -p player-playback --example video_window_check -- <video file>`

#[cfg(target_os = "macos")]
fn main() {
    use std::path::Path;

    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    use objc2_foundation::{NSDate, NSRunLoop};
    use player_playback::{Player, VideoPlayer};

    let path = std::env::args()
        .nth(1)
        .expect("usage: video_window_check <video file>");
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    let mut player = VideoPlayer::new().expect("video window opens");
    player.load(Path::new(&path)).expect("file loads");
    player.play().expect("playback starts");

    // Pump the window's events so it draws and the controls update.
    for _ in 0..200 {
        NSRunLoop::currentRunLoop().runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(0.05));
    }
    drop(player);
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("the video window is macOS only");
}
