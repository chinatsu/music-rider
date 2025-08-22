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
        let mut audio = audio::Audio::new(args.path, args.frequency);
        for _ in 0..audio.album_length {
            if play_rx.recv().is_ok() {
                audio.play_track(tx.clone()).unwrap();
                if let Some(track) = audio.next_track() {
                    audio.reset();
                    println!("Next track: {}", track.display());
                } else {
                    println!("No more tracks to play.");
                    break;
                }
            }
        }
        audio.flush();
    });

    let analyzer = Analyzer::new(sample_rate, args.scale);
    let mut prev_level = 0;
    if args.no_discovery {
        play_tx.send(true).unwrap();
        while let Ok(value) = rx.recv() {
            let score = analyzer.freq_score(value)?;
            let level = freq_score_to_level(score);
            println!("level: {level} :: freq score: {score}");
            if level == prev_level {
                println!("(Skipping level setting)");
            }
            prev_level = level;
        }
    } else {
        let adapters = manager.adapters().await?;
        let bike = Bike::new(&adapters).await?;

        play_tx.send(true).unwrap();

        while let Ok(value) = rx.recv() {
            let score = analyzer.freq_score(value)?;
            let level = freq_score_to_level(score);
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

fn freq_score_to_level(score: f64) -> i16 {
    let old_min = 0.;
    let old_max = 1.;
    let new_min = 1.;
    let new_max = 64.;

    (((score - old_min) / (old_max - old_min)) * (new_max - new_min) + new_min).clamp(1., 64.)
        as i16
}
