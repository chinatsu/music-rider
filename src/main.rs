use clap::Parser as _;
use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::{Stdout, Write, stdout};
use std::sync::mpsc::channel;

mod audio;
mod bike;
mod cli;

use crate::audio::analyze::Analyzer;
use crate::bike::bike_type_to_bike;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    // channel for audio samples
    let (tx, rx) = channel();

    // the bike decides when to start
    let (play_tx, play_rx) = channel();

    // the music player decides when to stop
    let (stop_tx, stop_rx) = channel();

    let (shutdown_tx, mut shutdown_rx) = channel();
    let (shutdown_tx2, shutdown_rx2) = channel();
    let (shutdown_tx3, mut shutdown_rx3) = channel();

    // let's figure out the sample rate of the audio files in the specified directory
    let flac = audio::get_flac_from_dir(args.path.clone());
    if flac.is_none() {
        eprintln!("No FLAC files found in the specified directory.");
        return Ok(());
    }
    let probed = audio::get_probe(&flac.unwrap());
    let sample_rate = probed
        .format
        .default_track()
        .unwrap()
        .codec_params
        .sample_rate
        .unwrap_or(44100);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        shutdown_tx.send(()).unwrap();
        shutdown_tx2.send(()).unwrap();
        shutdown_tx3.send(()).unwrap();
    });

    // spawn a task to play the audio files and send samples to the main thread
    tokio::spawn(async move {
        let mut audio = audio::Audio::new(args.path);
        let play = play_rx.recv().is_ok();
        for _ in 0..audio.album_length {
            if play {
                audio.play_track(tx.clone(), &mut shutdown_rx).unwrap();
                if audio.next_track().is_none() {
                    println!("No more tracks to play.");
                    stop_tx.send(()).unwrap();
                    break;
                }
            }
        }
        audio.flush();
    });

    // "pretty" printing
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();

    // try to figure out what gets sent from the bike
    if args.debug && !args.no_discovery {
        let bike = bike_type_to_bike(args.bike_type, args.max_level, &mut shutdown_rx3)
            .await
            .unwrap();
        loop {
            if let Some(data) = bike.read().await? {
                let state = format!(
                    "{:03} rpm :: {:03} W :: {:.2} km/h",
                    data.cadence, data.power, data.speed
                );
                print_state(&mut stdout, state);
            }
        }
    }

    // create an analyzer to receive the audio samples
    let analyzer = Analyzer::new(sample_rate, args.scale);

    // prev_level helps keep track, and reduce the number of calls being sent to the bike
    let mut prev_level = 0;

    // no discovery means we just print the levels to stdout
    if args.no_discovery {
        // enable playback
        play_tx.send(true).unwrap();

        // receive samples, analyze them, and print the resulting levels
        while let Ok(value) = rx.recv() {
            let score = analyzer.freq_score(value)?;
            let level = freq_score_to_level(args.max_level, score);
            let level_state = format!(
                "level {:<width$}",
                "#".repeat(level as usize),
                width = args.max_level as usize
            );
            print_state(&mut stdout, level_state);
        }
        stdout.execute(cursor::Show).unwrap();
    } else {
        // connect to the bike
        let bike = bike_type_to_bike(args.bike_type, args.max_level, &mut shutdown_rx3)
            .await
            .unwrap();

        // enable playback
        play_tx.send(true).unwrap();

        // receive samples, analyze them, and set the bike level accordingly (and also print the levels lol)
        while let Ok(value) = rx.recv() {
            if shutdown_rx2.try_recv().is_ok() {
                break;
            }
            if stop_rx.try_recv().is_ok() {
                break; // stop playback if the stop channel is closed
            }
            let score = analyzer.freq_score(value)?;
            let level = freq_score_to_level(args.max_level, score);
            if ![level, level + 1, level - 1].contains(&prev_level) {
                prev_level = level;
                bike.set_level(level).await?;
            }
            let level_state = format!(
                "level {:<width$}",
                "#".repeat(level as usize),
                width = args.max_level as usize
            );
            if args.no_read {
                print_state(&mut stdout, level_state);
            } else if let Some(data) = bike.read().await? {
                let state = format!(
                    "{:03} rpm :: {:03} W :: {:.2} km/h :: {:03} s",
                    data.cadence, data.power, data.speed, data.time
                );
                print_state(&mut stdout, format!("{state} :: {level_state}"));
            }

            // this is where we'd print stats like speed, cadence, wattage, etc.
            // but i haven't yet figured out which characteristics output this data
            // FTMS_STATS_UUID in bike.rs is what's used, and there are some other candidates in iconsole-characteristics.json
            // bike.print_stats().await?;
        }
        stdout.execute(cursor::Show).unwrap();

        // cleanly disconnect the bike once the songs are done playing and `rx` is closed
        // todo: disconnect the bike, and flush the audio output on SIGINT or SIGTERM
        bike.disconnect().await?;
    }

    Ok(())
}

fn print_state(stdout: &mut Stdout, input: String) {
    stdout.queue(cursor::SavePosition).unwrap();
    stdout.write_all(input.as_bytes()).unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout
        .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
        .unwrap();
}

/// Converts a frequency score (0.0 to 1.0) to a level (1 to 64).
fn freq_score_to_level(max: i16, score: f64) -> i16 {
    let old_min = 0.;
    let old_max = 1.;
    let new_min = 1.;
    let new_max = max as f64;

    (((score - old_min) / (old_max - old_min)) * (new_max - new_min) + new_min)
        .clamp(new_min, new_max) as i16
}
