//! Decode the first audio track of a movie file with the audio decoder chain, and report which
//! backend took it. Set `AUDIO_CHECK_BUNDLED=1` to force the bundled decoder, and
//! `DECODE_CHECK_DUMP` to write the samples as raw 32-bit floats for comparing with another decoder.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use quokka_playback::decode::audio::bundled::Bundled;
use quokka_playback::decode::audio::{self, AudioStream};

fn main() {
    for path in std::env::args().skip(1) {
        report(Path::new(&path));
    }
}

fn report(path: &Path) {
    let name = path.display();
    let track = match quokka_core::video::read_audio_samples(path) {
        Ok(Some(track)) => track,
        Ok(None) => {
            println!("{name}: no audio track this reads");
            return;
        }
        Err(error) => {
            println!("{name}: {error}");
            return;
        }
    };
    let stream = AudioStream {
        codec: track.codec,
        sample_rate: track.sample_rate,
        channels: track.channels,
        config: track.config.clone(),
    };
    if std::env::var_os("AUDIO_CHECK_SYSTEM").is_some() {
        #[cfg(target_os = "macos")]
        if let Err(error) =
            quokka_playback::decode::audio::audiotoolbox::AudioToolbox::open(&stream)
        {
            println!("{name}: AudioToolbox refused: {error}");
        }
    }
    let mut decoder: Box<dyn audio::AudioDecoder> =
        if std::env::var_os("AUDIO_CHECK_BUNDLED").is_some() {
            match Bundled::open(&stream) {
                Ok(decoder) => Box::new(decoder),
                Err(error) => {
                    println!("{name}: bundled decoder refused: {error}");
                    return;
                }
            }
        } else {
            match audio::open(&stream) {
                Ok(decoder) => decoder,
                Err(error) => {
                    println!("{name}: refused: {error}");
                    return;
                }
            }
        };
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            println!("{name}: {error}");
            return;
        }
    };
    let mut samples: Vec<f32> = Vec::new();
    for (index, sample) in track.samples.iter().enumerate() {
        let mut data = vec![0u8; sample.size as usize];
        if file.seek(SeekFrom::Start(sample.offset)).is_err() || file.read_exact(&mut data).is_err()
        {
            println!("{name}: could not read frame {index}");
            return;
        }
        match decoder.decode(&data) {
            Ok(decoded) => samples.extend(decoded),
            Err(error) => {
                println!(
                    "{name}: {} failed at frame {index}: {error}",
                    decoder.name()
                );
                return;
            }
        }
    }
    match decoder.finish() {
        Ok(tail) => samples.extend(tail),
        Err(error) => {
            println!("{name}: {} failed to finish: {error}", decoder.name());
            return;
        }
    }
    if let Some(dump) = std::env::var_os("DECODE_CHECK_DUMP") {
        let bytes: Vec<u8> = samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect();
        if let Err(error) = std::fs::write(&dump, bytes) {
            println!("{name}: could not write the dump: {error}");
        }
    }
    println!(
        "{name}: {} decoded {} frames into {} samples ({} Hz, {} channels)",
        decoder.name(),
        track.samples.len(),
        samples.len(),
        track.sample_rate,
        track.channels,
    );
}
