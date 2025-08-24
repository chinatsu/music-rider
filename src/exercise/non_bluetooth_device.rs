use std::sync::mpsc::Receiver;

use async_trait::async_trait;

use super::{Equipment, FTMSData};

#[derive(Debug, Clone)]
pub struct NonBluetoothDevice {
    pub name: String,
    max_level: i16,
    start_time: std::time::Instant,
}

#[async_trait]
impl Equipment for NonBluetoothDevice {
    async fn new(max_level: i16, _: &mut Receiver<()>) -> anyhow::Result<Self> {
        Ok(NonBluetoothDevice {
            name: "some hypothetical non-bluetooth device".to_string(),
            max_level,
            start_time: std::time::Instant::now(),
        })
    }
    async fn connect(&mut self) -> anyhow::Result<bool> {
        // Simulate a connection to a non-Bluetooth device
        println!("Connecting to: {}", self.name);
        Ok(true)
    }
    async fn disconnect(&self) -> anyhow::Result<()> {
        // Simulate disconnection from a non-Bluetooth device
        println!("Disconnecting from: {}", self.name);
        Ok(())
    }
    async fn set_level(&self, level: i16) -> anyhow::Result<()> {
        if !(1..=self.max_level).contains(&level) {
            return Err(anyhow::anyhow!(
                "Level must be between 1 and {}",
                self.max_level
            ));
        }
        let seconds_elapsed = self.start_time.elapsed().as_secs_f32();
        // Simulate setting the level on a non-Bluetooth device
        println!(
            "Setting level on: {} to {} at {}",
            self.name, level, seconds_elapsed
        );
        Ok(())
    }
    async fn read(&self) -> anyhow::Result<Option<FTMSData>> {
        // Simulate reading data from a non-Bluetooth device
        //println!("Reading data from: {}", self.name);
        Ok(Some(FTMSData {
            speed: f32::default(),
            cadence: f32::default(),
            distance: f32::default(),
            resistance: f64::default(),
            power: u8::default(),
            calories: f64::default(),
            heart_rate: f64::default(),
            time: self.start_time.elapsed().as_secs() as u16,
        }))
    }
}
