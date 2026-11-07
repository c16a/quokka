//! These tests use `MpvPlayer::without_window`, because a test process never starts the macOS
//! application that libmpv's window needs. The window itself is checked by running the app.
//! The file has no harness (see `harness = false` in playback/Cargo.toml), so the checks run
//! one after another on the main thread.

// The libmpv checks run only where libmpv is the video backend.
#[cfg(not(target_os = "macos"))]
mod checks {
    use std::path::Path;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use quokka_playback::{MpvPlayer, Player, PlayerState};

    /// How long to wait for mpv to report something that happens in the background.
    const WAIT: Duration = Duration::from_secs(10);

    pub fn main() {
        run(
            "a_missing_file_is_an_error_not_a_panic",
            a_missing_file_is_an_error,
        );

        // The real-file checks need a video: `QUOKKA_TEST_VIDEO=/path/to/clip.mkv just test-video`.
        match std::env::var_os("QUOKKA_TEST_VIDEO") {
            Some(path) => {
                let path = Path::new(&path);
                run("plays_seeks_pauses_and_stops_a_real_video", || {
                    plays_seeks_pauses_and_stops(path)
                });
                run(
                    "a_finished_video_reports_ended_and_replays_from_the_start",
                    || finished_video_replays(path),
                );
            }
            None => println!("skipped real-video checks: set QUOKKA_TEST_VIDEO to a video file"),
        }
    }

    fn run(name: &str, check: impl FnOnce()) {
        print!("test {name} ... ");
        check();
        println!("ok");
    }

    /// Wait until `ready` returns a value, or fail after `WAIT`.
    fn wait_for<T>(what: &str, mut ready: impl FnMut() -> Option<T>) -> T {
        let deadline = Instant::now() + WAIT;
        loop {
            if let Some(value) = ready() {
                return value;
            }
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            sleep(Duration::from_millis(50));
        }
    }

    fn a_missing_file_is_an_error() {
        let mut player = MpvPlayer::without_window().unwrap();
        // Loading happens in the background, so the failure shows up afterwards.
        player.load(Path::new("/definitely/not/here.mkv")).unwrap();
        wait_for("the failure", || player.last_error());
        assert_eq!(player.state(), PlayerState::Ended);
    }

    fn plays_seeks_pauses_and_stops(path: &Path) {
        let mut player = MpvPlayer::without_window().unwrap();

        player.load(path).unwrap();
        assert_eq!(
            player.state(),
            PlayerState::Paused,
            "loading must not start playback"
        );
        // Raw streams such as .obu carry no length, so mpv may report none. The UI then falls back to the library.
        match player.duration() {
            Some(length) => assert!(length > Duration::ZERO),
            None => println!("(duration unknown for this file)"),
        }

        player.play().unwrap();
        wait_for("playback to start", || {
            (player.state() == PlayerState::Playing && player.position().is_some()).then_some(())
        });
        sleep(Duration::from_millis(300));
        assert_eq!(player.state(), PlayerState::Playing);

        player.seek(Duration::from_secs(2)).unwrap();
        wait_for("the seek", || {
            player.position().filter(|at| {
                *at >= Duration::from_millis(1500) && *at <= Duration::from_millis(2600)
            })
        });

        player.pause().unwrap();
        wait_for("the pause", || {
            (player.state() == PlayerState::Paused).then_some(())
        });

        player.stop().unwrap();
        assert_eq!(player.state(), PlayerState::Idle);
        assert_eq!(player.position(), None);
    }

    /// A file that plays to its end reports `Ended`, and playing again restarts it.
    fn finished_video_replays(path: &Path) {
        let mut player = MpvPlayer::without_window().unwrap();
        player.load(path).unwrap();
        // Without a known length there is no sensible time limit to wait for the end. mpv reports
        // the length a moment after loading, so give it a few seconds before giving up.
        let deadline = Instant::now() + Duration::from_secs(5);
        let length = loop {
            if let Some(length) = player.duration() {
                break length;
            }
            if Instant::now() >= deadline {
                println!("(skipped the end-of-file check: duration unknown for this file)");
                return;
            }
            sleep(Duration::from_millis(50));
        };

        player.set_speed(4.0).unwrap();
        player.play().unwrap();
        // At 4x the end arrives after a quarter of the length; allow extra time for decoding.
        let deadline = Instant::now() + length / 4 + Duration::from_secs(30);
        while player.state() != PlayerState::Ended {
            assert!(Instant::now() < deadline, "never reached the end");
            sleep(Duration::from_millis(100));
        }
        assert_eq!(player.last_error(), None);

        player.set_speed(1.0).unwrap();
        player.play().unwrap();
        wait_for("the replay", || {
            (player.state() == PlayerState::Playing).then_some(())
        });
        sleep(Duration::from_millis(300));
        assert!(player.position().unwrap() < Duration::from_secs(2));
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    checks::main();
}

#[cfg(target_os = "macos")]
fn main() {}
