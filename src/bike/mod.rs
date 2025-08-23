pub mod different_bike;
pub mod iconsole_0028;
use btleplug::{
    api::{Central as _, CentralEvent, Peripheral as _, ScanFilter},
    platform::{Adapter, Peripheral},
};
use futures::StreamExt as _;

// todo: how do i handle different bikes :(
pub trait Bike {
    async fn new(adapters: &[Adapter], max_level: i16) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn connect(&self) -> anyhow::Result<bool>;
    async fn disconnect(&self) -> anyhow::Result<()>;
    async fn set_level(&self, level: i16) -> anyhow::Result<()>;
    async fn read(&self) -> anyhow::Result<Option<FTMSData>>;
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FTMSData {
    pub speed: f32,
    pub cadence: f32,
    pub distance: f32,
    pub resistance: f64,
    pub power: u8,
    pub calories: f64,
    pub heart_rate: f64,
    pub time: u16,
}

pub enum FTMSControlOpCode {
    RequestControl = 0x00,
    TargetPower = 0x05,
    // Start = 0x07,
    Stop = 0x08,
    // SpinDownControl = 0x13,
    TargetCadence = 0x14,
    // Success = 0x80,
}

pub enum StopCode {
    Stop = 0x01,
    // Pause = 0x02,
}

pub async fn get_peripheral(adapters: &[Adapter]) -> anyhow::Result<(Peripheral, String)> {
    let mut events = Vec::new();
    let mut peripheral_meta: Option<(Peripheral, String)> = None;

    for adapter in adapters {
        events.push(adapter.events().await?);
        adapter.start_scan(ScanFilter::default()).await?;
    }

    while let Some(event) = futures::stream::iter(events.iter_mut())
        .flatten()
        .next()
        .await
    {
        if let CentralEvent::DeviceDiscovered(id) = event {
            let central = adapters.get(1).unwrap();
            let peripheral = central.peripheral(&id).await?;
            if let Some(name) = peripheral.properties().await.unwrap().unwrap().local_name
                && (name.contains("Console") || name.contains("bike") || name.contains("fitness"))
            {
                peripheral_meta = Some((peripheral, name.to_string()));
                break;
            }
        }
    }
    for adapter in adapters {
        adapter.stop_scan().await?;
    }

    Ok(peripheral_meta.unwrap())
}
