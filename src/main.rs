use clap::Parser as _;
use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::{Stdout, Write, stdout};
use std::sync::mpsc::channel;

mod analysis;
mod audio;
mod bt;
mod cli;
mod exercise;

use exercise::equipment_type_to_equipment;

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

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!(
            "\nReceived SIGINT, shutting down~ keep in mind that if the program is currently scanning for bluetooth devices, we'll hang until a bluetooth event is received"
        );
        shutdown_tx.send(()).unwrap();
        shutdown_tx2.send(()).unwrap();
        shutdown_tx3.send(()).unwrap();
    });

    // spawn a task to play the audio files and send samples to the main thread
    tokio::spawn(async move {
        let mut audio = audio::Audio::new(args.path, args.scale, args.offset);
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
        let mut equipment = equipment_type_to_equipment(
            args.exercise_equipment_type,
            args.max_level,
            &mut shutdown_rx3,
        )
        .await
        .unwrap();
        if !equipment.connect().await? {
            return Ok(());
        }
        loop {
            if let Some(data) = equipment.read().await? {
                let state = format!(
                    "{:03} rpm :: {:03} W :: {:.2} km/h",
                    data.cadence, data.power, data.speed
                );
                print_state(&mut stdout, state);
            }
        }
    }

    // prev_level helps keep track, and reduce the number of calls being sent to the bike
    let mut prev_level = 0;

    // no discovery means we just print the levels to stdout
    if args.no_discovery {
        // enable playback
        play_tx.send(true).unwrap();

        // receive samples, analyze them, and print the resulting levels
        while let Ok(value) = rx.recv() {
            let level = freq_score_to_level(args.max_level, value);
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
        let mut equipment = equipment_type_to_equipment(
            args.exercise_equipment_type,
            args.max_level,
            &mut shutdown_rx3,
        )
        .await
        .unwrap();

        if !equipment.connect().await? {
            return Ok(());
        }

        // enable playback
        play_tx.send(true).unwrap();

        // receive samples, analyze them, and set the equipment level accordingly (and also print the levels lol)
        while let Ok(value) = rx.recv() {
            if shutdown_rx2.try_recv().is_ok() {
                break;
            }
            if stop_rx.try_recv().is_ok() {
                break; // stop playback if the stop channel is closed
            }
            let level = freq_score_to_level(args.max_level, value);
            if ![level, level + 1, level - 1].contains(&prev_level) {
                prev_level = level;
                equipment.set_level(level).await?;
            }
            let level_state = format!(
                "level {:<width$}",
                "#".repeat(level as usize),
                width = args.max_level as usize
            );
            if args.no_read {
                print_state(&mut stdout, level_state);
            } else if let Some(data) = equipment.read().await? {
                let state = format!(
                    "{:03} rpm :: {:03} W :: {:.2} km/h :: {:03} s",
                    data.cadence, data.power, data.speed, data.time
                );
                print_state(&mut stdout, format!("{state} :: {level_state}"));
            }
        }
        stdout.execute(cursor::Show).unwrap();

        // cleanly disconnect the equipment once the songs are done playing and `rx` is closed
        // todo: disconnect the equipment, and flush the audio output on SIGINT or SIGTERM
        equipment.disconnect().await?;
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
