//! Plays a file the way the output device does, pulling a block of samples on a fixed schedule,
//! and reports how long each pull waited for the decoder. A wait longer than the block's own
//! duration would be an underrun on a real device.
//!
//! `cargo run --release -p quokka-playback --example underrun_probe -- <file> [block_ms]`

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use quokka_playback::SymphoniaSource;
use rodio::Source;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: underrun_probe <file> [block_ms]")?;
    let block_ms: u64 = args.next().map_or(Ok(10), |s| s.parse())?;

    let error = Arc::new(Mutex::new(None));

    // Pass 1: decode as fast as possible, to see whether the decoder keeps up with real time.
    let mut source = SymphoniaSource::open(path.as_ref(), Arc::clone(&error))?;
    let rate = source.sample_rate().get() as f64;
    let channels = source.channels().get() as f64;
    let audio_seconds = {
        let start = Instant::now();
        let mut samples = 0u64;
        for _ in &mut source {
            samples += 1;
        }
        let elapsed = start.elapsed().as_secs_f64();
        let audio = samples as f64 / (rate * channels);
        println!(
            "unthrottled: {audio:.1} s of audio in {elapsed:.2} s, {:.1}x real time",
            audio / elapsed
        );
        audio
    };

    // Pass 2: pull on the device's schedule and time every pull.
    let source = SymphoniaSource::open(path.as_ref(), Arc::clone(&error))?;
    let block_frames = (rate * block_ms as f64 / 1000.0) as usize;
    let block_samples = block_frames * channels as usize;
    let block_time = Duration::from_millis(block_ms);
    let mut samples = source;

    let mut worst = Duration::ZERO;
    let mut waits_over_block = 0u32;
    let mut waits_over_1ms = 0u32;
    let mut total_wait = Duration::ZERO;
    let mut blocks = 0u32;
    let mut next_due = Instant::now();
    let mut buffer_end = Instant::now();
    let mut first = true;

    loop {
        let wait_start = Instant::now();
        let mut pulled = 0usize;
        let mut ended = false;
        for _ in 0..block_samples {
            match samples.next() {
                Some(_) => pulled += 1,
                None => {
                    ended = true;
                    break;
                }
            }
        }
        let waited = wait_start.elapsed();
        if pulled == 0 && ended {
            break;
        }
        if first {
            println!(
                "first block ready after {:.1} ms",
                waited.as_secs_f64() * 1000.0
            );
            first = false;
            buffer_end = Instant::now();
        } else {
            total_wait += waited;
            worst = worst.max(waited);
            if waited > block_time {
                waits_over_block += 1;
            }
            if waited > Duration::from_millis(1) {
                waits_over_1ms += 1;
            }
        }
        blocks += 1;
        if ended {
            break;
        }
        // The device takes a block every block_time; sleep until the next one is due.
        next_due += block_time;
        let now = Instant::now();
        if next_due > now {
            std::thread::sleep(next_due - now);
        } else {
            next_due = now;
        }
    }
    let _ = buffer_end;

    println!(
        "paced at {block_ms} ms blocks over {audio_seconds:.1} s of audio: {blocks} blocks, \
         worst wait {:.2} ms, total wait {:.1} ms, waits over 1 ms: {waits_over_1ms}, \
         waits longer than a block (underruns): {waits_over_block}",
        worst.as_secs_f64() * 1000.0,
        total_wait.as_secs_f64() * 1000.0,
    );
    if let Some(message) = error.lock().ok().and_then(|slot| slot.clone()) {
        println!("decoder error: {message}");
    }
    Ok(())
}
