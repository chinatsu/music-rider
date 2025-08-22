use btleplug::api::Manager as _;
use btleplug::platform::Manager;
use std::sync::mpsc::channel;
use std::{env, path::Path};

mod audio;
mod bike;
use bike::Bike;

const ALBUM_DIR: &str = "/home/cn/nas/media/Music/MONO/Hymn to the Immortal Wind/";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await?;
    let bike = Bike::new(&adapters).await?;
    let (tx, rx) = channel();

    tokio::spawn(async move {
        let directory = env::args()
            .nth(1)
            .unwrap_or_else(|| String::from(ALBUM_DIR));
        let path = Path::new(&directory);

        let mut audio = audio::Audio::new(path.to_path_buf());
        for _ in 0..audio.album_length {
            audio.play_track(tx.clone()).unwrap();
            if let Some(track) = audio.next_track() {
                audio.reset();
                println!("Next track: {track}");
            } else {
                println!("No more tracks to play.");
                break;
            }
        }
        audio.flush();
    });

    while let Ok(value) = rx.recv() {
        bike.set_level_from_loudness(value).await?;
        bike.print_stats().await?;
    }

    bike.disconnect().await?;

    Ok(())
}
