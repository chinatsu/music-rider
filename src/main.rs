
use btleplug::api::Manager as _;
use btleplug::platform::Manager;
use tokio::time;

mod bike;
use bike::Bike;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await?;
    let bike = Bike::new(&adapters).await?;

    let connection_status = bike.connect().await?;

    if !connection_status {
        println!("Failed to connect to bike: {}", bike.name);
        return Ok(());
    }

    for i in 1..32 {
        bike.set_level(i).await?;
        time::sleep(time::Duration::from_secs(1)).await;
        println!("Set level to {}", i);
        if let Some(stats) = bike.read().await? {
            println!(
                "Speed: {:.2} km/h, Cadence: {:.2} rpm, Distance: {:.2} km, Resistance: {:.2}, Power: {:.2} W, Calories: {:.2}, Heart Rate: {:.2} bpm, Time: {:.2} s",
                stats.speed,
                stats.cadence,
                stats.distance,
                stats.resistance,
                stats.power,
                stats.calories,
                stats.heart_rate,
                stats.time
            );
        };
    }

    bike.disconnect().await?;

    Ok(())
}