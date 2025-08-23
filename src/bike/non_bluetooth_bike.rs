use std::sync::mpsc::Receiver;

use async_trait::async_trait;

use super::{Bike, FTMSData};

#[derive(Debug, Clone)]
pub struct NonBluetoothBike {
    pub name: String,
    max_level: i16,
}

#[async_trait]
impl Bike for NonBluetoothBike {
    async fn new(max_level: i16, _: &mut Receiver<()>) -> anyhow::Result<Self> {
        Ok(NonBluetoothBike {
            name: "some hypothetical non-bluetooth bike".to_string(),
            max_level,
        })
    }
    async fn connect(&self) -> anyhow::Result<bool> {
        // Simulate a connection to a non-Bluetooth bike
        println!("Connecting to non-Bluetooth bike: {}", self.name);
        Ok(true)
    }
    async fn disconnect(&self) -> anyhow::Result<()> {
        // Simulate disconnection from a non-Bluetooth bike
        println!("Disconnecting from non-Bluetooth bike: {}", self.name);
        Ok(())
    }
    async fn set_level(&self, level: i16) -> anyhow::Result<()> {
        if !(1..=self.max_level).contains(&level) {
            return Err(anyhow::anyhow!(
                "Level must be between 1 and {}",
                self.max_level
            ));
        }
        // Simulate setting the level on a non-Bluetooth bike
        println!(
            "Setting level to {} on non-Bluetooth bike: {}",
            level, self.name
        );
        Ok(())
    }
    async fn read(&self) -> anyhow::Result<Option<FTMSData>> {
        // Simulate reading data from a non-Bluetooth bike
        println!("Reading data from non-Bluetooth bike: {}", self.name);
        Ok(Some(FTMSData::default()))
    }
}
