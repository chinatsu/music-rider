use btleplug::api::Manager as _;
use btleplug::platform::Manager;
use std::{env, path::Path};

mod audio;
mod bike;
use bike::Bike;

const ALBUM_DIR: &str = "/home/cn/nas/media/Music/MONO/Hymn to the Immortal Wind/";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let directory = env::args()
            .nth(1)
            .unwrap_or_else(|| String::from(ALBUM_DIR));
    let path = Path::new(
        &directory,
    );

    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await?;
    let bike = Bike::new(&adapters).await?;

    let mut audio = audio::Audio::new(path.to_path_buf(), Some(bike.clone()));
    for _ in 0..audio.album_length {
        audio.play_track().await?;
        if let Some(track) = audio.next_track() {
            audio.reset_analyzer();
            println!("Next track: {track}");
        } else {
            println!("No more tracks to play.");
            break;
        }
    }
    audio.flush();

    bike.disconnect().await?;

    Ok(())
}
