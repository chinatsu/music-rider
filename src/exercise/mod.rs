pub mod debug_bike;
pub mod iconsole_0028_bike;
pub mod non_bluetooth_device;
use std::sync::mpsc::Receiver;

use async_trait::async_trait;

use debug_bike::DebugBike;
use iconsole_0028_bike::Iconsole0028Bike;
use non_bluetooth_device::NonBluetoothDevice;

#[allow(dead_code)]
pub enum EquipmentType {
    Iconsole0028Bike,
    DebugBike,
    NonBluetoothDevice,
}

#[async_trait]
pub trait Equipment {
    async fn new(max_level: i16, shutdown_rx: &mut Receiver<()>) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn connect(&mut self) -> anyhow::Result<bool>;
    async fn disconnect(&self) -> anyhow::Result<()>;
    async fn set_level(&self, level: i16) -> anyhow::Result<()>;
    async fn read(&self) -> anyhow::Result<Option<FTMSData>>;
}

pub async fn equipment_type_to_equipment(
    name: String,
    max_level: i16,
    shutdown_rx: &mut Receiver<()>,
) -> Option<Box<dyn Equipment>> {
    match name.as_str() {
        "28" => Some(Box::new(
            Iconsole0028Bike::new(max_level, shutdown_rx).await.unwrap(),
        )),
        "debug" => Some(Box::new(
            DebugBike::new(max_level, shutdown_rx).await.unwrap(),
        )),
        "device" => Some(Box::new(
            NonBluetoothDevice::new(max_level, shutdown_rx)
                .await
                .unwrap(),
        )),
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
