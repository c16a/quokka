//! Decode the first video track of each MP4 file with the platform's decoder, and report what the
//! decoder accepts. Run it with `just decode-check <files>`.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use quokka_playback::decode::{self, Packet, StreamConfig, Surface};

fn main() {
    for path in std::env::args().skip(1) {
        report(Path::new(&path));
    }
}

fn report(path: &Path) {
    let name = path.display();
    let track = match quokka_core::video::read_samples(path) {
        Ok(Some(track)) => track,
        Ok(None) => {
            println!("{name}: no MP4 video track to read");
            return;
        }
        Err(error) => {
            println!("{name}: {error}");
            return;
        }
    };
    let format = quokka_core::video::inspect(path).unwrap_or_default();
    let stream = StreamConfig {
        codec: track.codec,
        width: track.width,
        height: track.height,
        config: track.config.clone(),
        timescale: track.timescale,
        bit_depth: format.bit_depth.unwrap_or(8),
        chroma: format.chroma.unwrap_or(quokka_core::video::Chroma::Yuv420),
        colour: format.colour,
    };
    let mut decoder = match decode::open(&stream) {
        Ok(decoder) => decoder,
        Err(error) => {
            println!("{name}: refused: {error}");
            return;
        }
    };
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            println!("{name}: {error}");
            return;
        }
    };
    for (index, sample) in track.samples.iter().enumerate() {
        let mut data = vec![0u8; sample.size as usize];
        if file.seek(SeekFrom::Start(sample.offset)).is_err() || file.read_exact(&mut data).is_err()
        {
            println!("{name}: could not read sample {index}");
            return;
        }
        let packet = Packet {
            data,
            decode_time: sample.decode_time,
            presentation_time: sample.presentation_time,
            sync: sample.sync,
        };
        if let Err(error) = decoder.decode(&packet) {
            println!(
                "{name}: {} failed at sample {index}: {error}",
                decoder.name()
            );
            return;
        }
    }
    if let Err(error) = decoder.finish() {
        println!("{name}: {} failed to finish: {error}", decoder.name());
        return;
    }
    let pictures = decoder.take_pictures();
    // With DECODE_CHECK_DUMP set, the pictures are also written out as raw planar bytes, for
    // comparing with another decoder's output.
    if let Ok(dump) = std::env::var("DECODE_CHECK_DUMP") {
        let mut bytes = Vec::new();
        for picture in &pictures {
            match &picture.surface {
                Surface::Cpu(samples) => bytes.extend_from_slice(samples),
                #[cfg(target_os = "macos")]
                Surface::Platform(buffer) => bytes.extend(buffer.to_bytes()),
            }
        }
        if let Err(error) = std::fs::write(&dump, bytes) {
            println!("{name}: could not write {dump}: {error}");
        }
    }
    let format = pictures
        .first()
        .map(|picture| String::from_utf8_lossy(&picture.pixel_format).into_owned())
        .unwrap_or_else(|| "none".into());
    println!(
        "{name}: {} decoded {} of {} frames at {}x{}, pixel format {format}",
        decoder.name(),
        pictures.len(),
        track.samples.len(),
        track.width,
        track.height,
    );
}
