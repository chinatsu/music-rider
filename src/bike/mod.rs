pub mod different_bike;
pub mod iconsole_0028;
pub mod non_bluetooth_bike;
use btleplug::{
    api::{Central as _, CentralEvent, Peripheral as _, ScanFilter},
    platform::{Adapter, Peripheral},
};
use futures::StreamExt as _;

use different_bike::DifferentBike;
use iconsole_0028::Iconsole0028Bike;
use non_bluetooth_bike::NonBluetoothBike;

pub trait Bike {
    async fn new(max_level: i16) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn connect(&self) -> anyhow::Result<bool>;
    async fn disconnect(&self) -> anyhow::Result<()>;
    async fn set_level(&self, level: i16) -> anyhow::Result<()>;
    async fn read(&self) -> anyhow::Result<Option<FTMSData>>;
}

pub enum BikeType {
    Iconsole0028(Box<Iconsole0028Bike>),
    DifferentBike(Box<DifferentBike>),
    NonBluetoothBike(Box<NonBluetoothBike>),
}

#[allow(dead_code)]
impl BikeType {
    pub async fn connect(&self) -> anyhow::Result<bool> {
        match self {
            BikeType::Iconsole0028(bike) => bike.connect().await,
            BikeType::DifferentBike(bike) => bike.connect().await,
            BikeType::NonBluetoothBike(bike) => bike.connect().await,
        }
    }

    pub async fn disconnect(&self) -> anyhow::Result<()> {
        match self {
            BikeType::Iconsole0028(bike) => bike.disconnect().await,
            BikeType::DifferentBike(bike) => bike.disconnect().await,
            BikeType::NonBluetoothBike(bike) => bike.disconnect().await,
        }
    }

    pub async fn set_level(&self, level: i16) -> anyhow::Result<()> {
        match self {
            BikeType::Iconsole0028(bike) => bike.set_level(level).await,
            BikeType::DifferentBike(bike) => bike.set_level(level).await,
            BikeType::NonBluetoothBike(bike) => bike.set_level(level).await,
        }
    }

    pub async fn read(&self) -> anyhow::Result<Option<FTMSData>> {
        match self {
            BikeType::Iconsole0028(bike) => bike.read().await,
            BikeType::DifferentBike(bike) => bike.read().await,
            BikeType::NonBluetoothBike(bike) => bike.read().await,
        }
    }
}

pub async fn bike_type_to_bike(name: String, max_level: i16) -> Option<BikeType> {
    match name.as_str() {
        "0028" => Some(BikeType::Iconsole0028(Box::new(
            Iconsole0028Bike::new(max_level).await.unwrap(),
        ))),
        "debug-bike" => Some(BikeType::DifferentBike(Box::new(
            DifferentBike::new(max_level).await.unwrap(),
        ))),
        "non-bluetooth-bike" => Some(BikeType::NonBluetoothBike(Box::new(
            NonBluetoothBike::new(max_level).await.unwrap(),
        ))),
        _ => {
            eprintln!("Unknown bike type: {name}");
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
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
