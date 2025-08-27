use std::sync::mpsc::Receiver;

use async_trait::async_trait;
use btleplug::{
    api::{CharPropFlags, Characteristic, Peripheral as _},
    platform::Peripheral,
};
use futures::StreamExt;
use uuid::Uuid;

use super::{Equipment, EquipmentType, FTMSData};
use crate::bt::get_peripheral;

#[derive(Debug, Clone)]
pub struct DebugBike {
    peripheral: Peripheral,
    pub name: String,
    idk: Vec<Characteristic>,
    max_level: i16,
}

#[async_trait]
impl Equipment for DebugBike {
    async fn new(max_level: i16, shutdown_rx: &mut Receiver<()>) -> anyhow::Result<Self> {
        let meta = get_peripheral(EquipmentType::DebugBike, shutdown_rx)
            .await?
            .unwrap();
        Ok(DebugBike {
            peripheral: meta.0,
            name: meta.1,
            idk: Vec::new(),
            max_level,
        })
    }

    async fn connect(&mut self) -> anyhow::Result<bool> {
        let is_connected = self.peripheral.is_connected().await?;
        if !is_connected {
            self.peripheral.connect().await?;
        }
        self.set_characteristics().await?;
        self.subscribe().await?;
        println!("Found and connected to: {}", self.name);
        Ok(self.peripheral.is_connected().await?)
    }

    async fn disconnect(&self) -> anyhow::Result<()> {
        self.cleanup().await?;
        self.peripheral.disconnect().await?;
        Ok(())
    }

    async fn set_level(&self, level: i16) -> anyhow::Result<()> {
        if !(1..=self.max_level).contains(&level) {
            return Err(anyhow::anyhow!(
                "Level must be between 1 and {}",
                self.max_level
            ));
        }
        Ok(())
    }

    async fn read(&self) -> anyhow::Result<Option<FTMSData>> {
        let (data, _) = self.notifications().await?;
        println!("Received data: {data:?}");

        Ok(Some(FTMSData {
            speed: 0.0,
            cadence: 0.0,
            distance: 0.0,
            resistance: 0.0,
            power: 0,
            calories: 0.0,
            heart_rate: 0.0,
            time: 0,
        }))
    }
}

impl DebugBike {
    async fn cleanup(&self) -> anyhow::Result<()> {
        for characteristic in &self.idk {
            self.peripheral.unsubscribe(characteristic).await?;
        }
        Ok(())
    }

    async fn set_characteristics(&mut self) -> anyhow::Result<()> {
        self.peripheral.discover_services().await?;
        for characteristic in self.peripheral.characteristics() {
            if characteristic.properties.contains(CharPropFlags::NOTIFY) {
                self.idk.push(characteristic.clone());
            }
        }
        Ok(())
    }

    async fn subscribe(&self) -> anyhow::Result<()> {
        for characteristic in &self.idk {
            self.peripheral.subscribe(characteristic).await?;
        }
        Ok(())
    }

    async fn notifications(&self) -> anyhow::Result<(Vec<u8>, Uuid)> {
        let mut notifications = self.peripheral.notifications().await?;
        if let Some(data) = notifications.next().await {
            return Ok((data.value, data.uuid));
        }

        Ok((Vec::new(), Uuid::nil()))
    }
}
