use std::path::Path;
use std::time::Duration;

use crate::{PlaybackError, Player, PlayerState, Result};

/// In-memory `Player` with a fixed duration. Used by tests and UI work
/// before a real backend exists. It does not decode anything.
#[derive(Debug)]
pub struct MockPlayer {
    duration: Duration,
    state: PlayerState,
    position: Duration,
    loaded: bool,
}

impl MockPlayer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            state: PlayerState::Idle,
            position: Duration::ZERO,
            loaded: false,
        }
    }

    fn require_loaded(&self) -> Result<()> {
        if self.loaded {
            Ok(())
        } else {
            Err(PlaybackError::NoMedia)
        }
    }
}

impl Player for MockPlayer {
    fn load(&mut self, _path: &Path) -> Result<()> {
        self.loaded = true;
        self.state = PlayerState::Paused;
        self.position = Duration::ZERO;
        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        self.require_loaded()?;
        if self.state == PlayerState::Ended {
            self.position = Duration::ZERO;
        }
        self.state = PlayerState::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.require_loaded()?;
        if self.state == PlayerState::Playing {
            self.state = PlayerState::Paused;
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.loaded = false;
        self.state = PlayerState::Idle;
        self.position = Duration::ZERO;
        Ok(())
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        self.require_loaded()?;
        if position >= self.duration {
            self.position = self.duration;
            self.state = PlayerState::Ended;
        } else {
            self.position = position;
            if self.state == PlayerState::Ended {
                self.state = PlayerState::Paused;
            }
        }
        Ok(())
    }

    fn position(&self) -> Option<Duration> {
        self.loaded.then_some(self.position)
    }

    fn duration(&self) -> Option<Duration> {
        self.loaded.then_some(self.duration)
    }

    fn state(&self) -> PlayerState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_follow_state_machine() {
        let mut p = MockPlayer::new(Duration::from_secs(10));
        assert_eq!(p.state(), PlayerState::Idle);
        assert!(matches!(p.play(), Err(PlaybackError::NoMedia)));

        p.load(Path::new("x.mp3")).unwrap();
        assert_eq!(p.state(), PlayerState::Paused);
        p.play().unwrap();
        assert_eq!(p.state(), PlayerState::Playing);
        p.pause().unwrap();
        assert_eq!(p.state(), PlayerState::Paused);

        p.seek(Duration::from_secs(4)).unwrap();
        assert_eq!(p.position(), Some(Duration::from_secs(4)));
        p.seek(Duration::from_secs(99)).unwrap();
        assert_eq!(p.state(), PlayerState::Ended);
        assert_eq!(p.position(), Some(Duration::from_secs(10)));

        p.play().unwrap();
        assert_eq!(p.position(), Some(Duration::ZERO));

        p.stop().unwrap();
        assert_eq!(p.state(), PlayerState::Idle);
        assert_eq!(p.position(), None);
        assert_eq!(p.duration(), None);
    }
}
