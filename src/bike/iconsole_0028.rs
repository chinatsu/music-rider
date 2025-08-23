use std::sync::mpsc::Receiver;

use btleplug::{
    api::{CharPropFlags, Characteristic, Manager as _, Peripheral as _},
    platform::{Manager, Peripheral},
};
use futures::StreamExt;
use uuid::Uuid;

use super::{Bike, FTMSControlOpCode, FTMSData, StopCode};

static FTMS_SERVICE_UUID: &str = "00001826"; // FTMS service
static FTMS_STATS_UUID: &str = "00002ad2"; // FTMS read?

#[derive(Debug, Clone)]
pub struct Iconsole0028Bike {
    peripheral: Peripheral,
    pub name: String,
    notify: Option<Characteristic>,
    control: Option<Characteristic>,
    stats: Option<Characteristic>,
    idk: Vec<Characteristic>,
    max_level: i16,
}

impl Bike for Iconsole0028Bike {
    async fn new(max_level: i16, shutdown_rx: &mut Receiver<()>) -> anyhow::Result<Self> {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await?;
        let meta = super::get_peripheral(&adapters, shutdown_rx)
            .await?
            .unwrap();
        let mut bike = Iconsole0028Bike {
            peripheral: meta.0,
            name: meta.1,
            notify: None,
            control: None,
            stats: None,
            idk: Vec::new(),
            max_level,
        };
        bike.connect().await?;
        bike.set_characteristics().await?;
        bike.subscribe().await?;
        bike.request_control().await?;
        println!("Found and connected to bike: {}", bike.name);
        Ok(bike)
    }

    async fn connect(&self) -> anyhow::Result<bool> {
        let is_connected = self.peripheral.is_connected().await?;
        if !is_connected {
            self.peripheral.connect().await?;
        }
        Ok(is_connected)
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
        // we might be able to set only one of these, but for now we're setting both
        self.set_cadence(level).await?;
        self.set_power(level).await
    }

    async fn read(&self) -> anyhow::Result<Option<FTMSData>> {
        let (data, _) = self.notifications().await?;
        if data.len() < 29 {
            return Ok(None);
        }
        let distance = data[10] as f32 / 1000.;
        let power = data[15]; // does not seem to be the correct field
        let time = data[26] as u16 | ((data[27] as u16) << 8);
        let cadence = (data[6] as f32 / 2.).round();
        let speed = (data[2] as u16 | ((data[3] as u16) << 8)) as f32 / 100.;

        Ok(Some(FTMSData {
            speed,
            cadence,
            distance,
            resistance: 0.0,
            power,
            calories: 0.0,
            heart_rate: 0.0,
            time,
        }))
    }
}

impl Iconsole0028Bike {
    async fn cleanup(&self) -> anyhow::Result<()> {
        if let Some(notify) = &self.notify {
            self.peripheral.unsubscribe(notify).await?;
        }
        self.write(&[FTMSControlOpCode::Stop as u8, StopCode::Stop as u8])
            .await
    }

    async fn set_characteristics(&mut self) -> anyhow::Result<()> {
        self.peripheral.discover_services().await?;
        for characteristic in self.peripheral.characteristics() {
            if characteristic
                .service_uuid
                .to_string()
                .starts_with(FTMS_SERVICE_UUID)
                && characteristic.properties.contains(CharPropFlags::NOTIFY)
                && characteristic.properties.contains(CharPropFlags::READ)
            {
                self.notify = Some(characteristic.clone());
            }
            if characteristic
                .service_uuid
                .to_string()
                .starts_with(FTMS_SERVICE_UUID)
                && characteristic.properties.contains(CharPropFlags::WRITE)
                && characteristic.properties.contains(CharPropFlags::INDICATE)
            {
                self.control = Some(characteristic.clone());
            }
            if characteristic.uuid.to_string().starts_with(FTMS_STATS_UUID) {
                self.stats = Some(characteristic.clone());
            }
            if characteristic.properties.contains(CharPropFlags::NOTIFY) {
                self.idk.push(characteristic.clone());
            }
        }
        Ok(())
    }

    async fn subscribe(&self) -> anyhow::Result<()> {
        if let Some(notify) = &self.notify {
            self.peripheral.subscribe(notify).await?;
        } else {
            return Err(anyhow::anyhow!("No notify characteristic found"));
        }
        if let Some(stats) = &self.stats {
            self.peripheral.subscribe(stats).await?;
        } else {
            return Err(anyhow::anyhow!("No stats characteristic found"));
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

    async fn request_control(&self) -> anyhow::Result<()> {
        let request_control = [FTMSControlOpCode::RequestControl as u8];
        self.write(&request_control).await
    }

    async fn set_cadence(&self, level: i16) -> anyhow::Result<()> {
        let i16_num = level * 10;

        let resistance = [(i16_num & 0xFF) as u8, ((i16_num >> 8) & 0xFF) as u8];

        self.write(&[
            FTMSControlOpCode::TargetCadence as u8,
            resistance[0],
            resistance[1],
        ])
        .await
    }

    async fn set_power(&self, level: i16) -> anyhow::Result<()> {
        let i16_num = level * 10;

        let resistance = [(i16_num & 0xFF) as u8, ((i16_num >> 8) & 0xFF) as u8];

        self.write(&[
            FTMSControlOpCode::TargetPower as u8,
            resistance[0],
            resistance[1],
        ])
        .await
    }

    async fn write(&self, data: &[u8]) -> anyhow::Result<()> {
        if let Some(control) = &self.control {
            self.peripheral
                .write(control, data, btleplug::api::WriteType::WithResponse)
                .await?;
        } else {
            return Err(anyhow::anyhow!("No control characteristic found"));
        }
        Ok(())
    }
}
