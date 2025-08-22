use btleplug::api::Manager as _;
use btleplug::platform::Manager;
use clap::Parser as _;
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

    tokio::spawn(async move {
        let mut audio = audio::Audio::new(args.path, args.update_frequency);
        for _ in 0..audio.album_length {
            audio.play_track(tx.clone()).unwrap();
            if let Some(track) = audio.next_track() {
                audio.reset();
                println!("Next track: {}", track.display());
            } else {
                println!("No more tracks to play.");
                break;
            }
        }
        audio.flush();
    });

    let analyzer = Analyzer::new();
    let mut prev_level = 0;
    if args.no_discovery {
        while let Ok(value) = rx.recv() {
            let score = analyzer.low_freq_score(value)?;
            let level = freq_score_to_level(score, args.scale);
            println!("Level: {}", level);
            println!("Low frequency score: {}", score);
            if level == prev_level {
                println!("(Skipping level setting)");
            }
            prev_level = level;
        }
    } else {
        let adapters = manager.adapters().await?;
        let bike = Bike::new(&adapters).await?;

        while let Ok(value) = rx.recv() {
            let score = analyzer.low_freq_score(value)?;
            let level = freq_score_to_level(score, args.scale);
            if level != prev_level {
                prev_level = level;
                bike.set_level(level).await?;
            }
            bike.print_stats().await?;
        }

        bike.disconnect().await?;
    }

    Ok(())
}

fn freq_score_to_level(score: f64, scale: f64) -> i16 {
    let old_min = 0.;
    let old_max = 1.;
    let new_min = 1.;
    let new_max = 32. * scale;

    (((score - old_min) / (old_max - old_min)) * (new_max - new_min) + new_min).clamp(1., 32.)
        as i16
}
