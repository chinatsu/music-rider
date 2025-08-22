use btleplug::api::Manager as _;
use btleplug::platform::Manager;
use clap::Parser as _;
use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::{Stdout, Write, stdout};
use std::sync::mpsc::channel;

mod audio;
mod bike;
mod cli;
use bike::Bike;

use crate::audio::analyze::Analyzer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = Manager::new().await.unwrap();
    let args = cli::Args::parse();

    let (tx, rx) = channel();
    let (play_tx, play_rx) = channel();
    let probed = audio::get_probe(&audio::get_flac_from_dir(args.path.clone()).unwrap());
    let sample_rate = probed
        .format
        .default_track()
        .unwrap()
        .codec_params
        .sample_rate
        .unwrap_or(44100);

    tokio::spawn(async move {
        let mut audio = audio::Audio::new(args.path);
        let play = play_rx.recv().is_ok();
        for _ in 0..audio.album_length {
            if play {
                audio.play_track(tx.clone()).unwrap();
                if let None = audio.next_track() {
                    println!("No more tracks to play.");
                    break;
                }
            }
        }
        audio.flush();
    });

    let analyzer = Analyzer::new(sample_rate, args.scale);
    let mut prev_level = 0;
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    if args.no_discovery {
        play_tx.send(true).unwrap();
        while let Ok(value) = rx.recv() {
            let score = analyzer.freq_score(value)?;
            let level = freq_score_to_level(score);
            print_state(&mut stdout, level, score);
        }
        stdout.execute(cursor::Show).unwrap();
    } else {
        let adapters = manager.adapters().await?;
        let bike = Bike::new(&adapters).await?;
        play_tx.send(true).unwrap();

        while let Ok(value) = rx.recv() {
            let score = analyzer.freq_score(value)?;
            let level = freq_score_to_level(score);
            if ![level, level + 1, level - 1].contains(&prev_level) {
                prev_level = level;
                bike.set_level(level).await?;
            }
            print_state(&mut stdout, prev_level, score);

            // bike.print_stats().await?;
        }
        stdout.execute(cursor::Show).unwrap();
        bike.disconnect().await?;
    }

    Ok(())
}

fn print_state(stdout: &mut Stdout, level: i16, score: f64) {
    stdout.queue(cursor::SavePosition).unwrap();
    stdout
        .write_all(format!("\rlevel: {level} :: freq score: {score:.5}").as_bytes())
        .unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout
        .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
        .unwrap();
}

fn freq_score_to_level(score: f64) -> i16 {
    let old_min = 0.;
    let old_max = 1.;
    let new_min = 1.;
    let new_max = 64.;

    (((score - old_min) / (old_max - old_min)) * (new_max - new_min) + new_min).clamp(1., 64.)
        as i16
}
