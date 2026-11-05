//! Live internet radio: an HTTP stream decoded as it arrives. A stream has no length and can't
//! be sought, so the player plays it until the connection ends.

use std::io::{self, Read};
use std::sync::Mutex;
use std::time::Duration;

use symphonia::core::formats::probe::Hint;
use symphonia::core::io::ReadOnlySource;

use crate::symphonia_source::{ErrorSlot, SymphoniaSource};

const USER_AGENT: &str = concat!("player/", env!("CARGO_PKG_VERSION"));

/// A reader that can be shared between threads. The HTTP body reader is not `Sync` on its own,
/// and the decoder needs a `Sync` source.
struct SharedReader(Mutex<Box<dyn Read + Send>>);

impl Read for SharedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut inner = self
            .0
            .lock()
            .map_err(|_| io::Error::other("stream reader poisoned"))?;
        inner.read(buf)
    }
}

/// Connect to a stream and start decoding it. Blocks until the server answers, so call it off
/// the UI thread.
pub fn open_stream(url: &str) -> Result<SymphoniaSource, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(10)))
        .user_agent(USER_AGENT)
        .build()
        .into();
    let response = agent.get(url).call().map_err(|e| e.to_string())?;
    let hint = content_hint(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
    );
    let reader = response.into_body().into_reader();
    let source = SharedReader(Mutex::new(Box::new(reader)));
    SymphoniaSource::from_source(
        Box::new(ReadOnlySource::new(source)),
        &hint,
        ErrorSlot::default(),
    )
}

/// The format a stream's content type names, so the decoder knows what to expect.
fn content_hint(content_type: Option<&str>) -> Hint {
    let mut hint = Hint::new();
    let mime = content_type
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let extension = match mime.as_str() {
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/aac" | "audio/aacp" | "audio/x-aac" => "aac",
        "audio/ogg" | "application/ogg" => "ogg",
        "audio/flac" | "audio/x-flac" => "flac",
        _ => return hint,
    };
    hint.with_extension(extension);
    hint
}
