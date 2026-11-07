//! Play a music file through the music source and report which decoder made its samples. Set
//! `DECODE_CHECK_DUMP` to write the samples as raw 32-bit floats for comparing with another decoder.

use std::path::Path;
use std::sync::{Arc, Mutex};

use player_playback::SymphoniaSource;
use rodio::Source;

fn main() {
    for path in std::env::args().skip(1) {
        let name = Path::new(&path).display().to_string();
        let source = match SymphoniaSource::open(Path::new(&path), Arc::new(Mutex::new(None))) {
            Ok(source) => source,
            Err(error) => {
                println!("{name}: {error}");
                continue;
            }
        };
        let decoder = source.decoder_name();
        let (channels, rate) = (source.channels().get(), source.sample_rate().get());
        let samples: Vec<f32> = source.collect();
        if let Some(dump) = std::env::var_os("DECODE_CHECK_DUMP") {
            let bytes: Vec<u8> = samples
                .iter()
                .flat_map(|sample| sample.to_le_bytes())
                .collect();
            let _ = std::fs::write(dump, bytes);
        }
        println!(
            "{name}: {decoder} produced {} samples ({rate} Hz, {channels} channels)",
            samples.len()
        );
    }
}
