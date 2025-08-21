use btleplug::{
    api::{Central as _, CentralEvent, CharPropFlags, Characteristic, Peripheral as _, ScanFilter},
    platform::{Adapter, Peripheral},
};
use futures::StreamExt;

static FTMS_SERVICE_UUID: &str = "00001826"; // FTMS service
static FTMS_STATS_UUID: &str = "00002ad2"; // FTMS read?

pub struct FTMSData {
    pub speed: f64,      // (data[2] | data[3] << 8) / 100
    pub cadence: f64,    // data[4] / 2
    pub distance: f64,   // data[6] / 1000
    pub resistance: f64, // data[9]
    pub power: f64,      // data[11]
    pub calories: f64,   // data[13]
    pub heart_rate: f64, // data[18]
    pub time: f64,       // data[19]
}

enum FTMSControlOpCode {
    RequestControl = 0x00,
    TargetPower = 0x05,
    Start = 0x07,
    Stop = 0x08,
    SpinDownControl = 0x13,
    TargetCadence = 0x14,
    Success = 0x80,
}

pub enum StopCode {
    Stop = 0x01,
    Pause = 0x02,
}

#[derive(Debug, Clone)]
pub struct Bike {
    peripheral: Peripheral,
    pub name: String,
    notify: Option<Characteristic>,
    control: Option<Characteristic>,
    stats: Option<Characteristic>,
}

impl Bike {
    pub async fn new(adapters: &Vec<Adapter>) -> anyhow::Result<Self> {
        let meta = get_peripheral(adapters).await?;
        let mut bike = Bike {
            peripheral: meta.0,
            name: meta.1,
            notify: None,
            control: None,
            stats: None,
        };
        bike.connect().await?;
        bike.set_characteristics().await?;
        bike.subscribe().await?;
        bike.request_control().await?;
        Ok(bike)
    }

    pub async fn connect(&self) -> anyhow::Result<bool> {
        let is_connected = self.peripheral.is_connected().await?;
        if !is_connected {
            self.peripheral.connect().await?;
        }
        Ok(is_connected)
    }

    pub async fn disconnect(&self) -> anyhow::Result<()> {
        self.cleanup().await?;
        self.peripheral.disconnect().await?;
        Ok(())
    }

    pub async fn set_level(&self, level: i16) -> anyhow::Result<()> {
        if level < 1 || level > 32 {
            return Err(anyhow::anyhow!("Level must be between 1 and 32"));
        }
        self.set_cadence(level).await?;
        self.set_power(level).await
    }

    pub async fn read(&self) -> anyhow::Result<Option<FTMSData>> {
        let data = self.notifications().await?;
        if data.len() < 20 {
            return Ok(None);
        }
        Ok(Some(FTMSData {
            speed: ((data[2] as u16 | ((data[3] as u16) << 8)) as f64 / 100.),
            cadence: (data[4] as f64 / 2.),
            distance: data[6] as f64 / 1000.,
            resistance: data[9] as f64,
            power: data[11] as f64,
            calories: data[13] as f64,
            heart_rate: data[18] as f64,
            time: data[19] as f64,
        }))
    }

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

    pub async fn notifications(&self) -> anyhow::Result<Vec<u8>> {
        let mut notifications = self.peripheral.notifications().await?.take(1);
        while let Some(data) = notifications.next().await {
            return Ok(data.value);
        }

        Ok(Vec::new())
    }

    async fn request_control(&self) -> anyhow::Result<()> {
        let request_control = [0x00];
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

async fn get_peripheral(adapters: &Vec<Adapter>) -> anyhow::Result<(Peripheral, String)> {
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
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                let central = adapters.iter().nth(1).unwrap();
                let peripheral = central.peripheral(&id).await?;
                if let Some(name) = peripheral.properties().await.unwrap().unwrap().local_name {
                    if name.contains("Console") || name.contains("bike") || name.contains("fitness")
                    {
                        println!("Found bike: {}", name);
                        peripheral_meta = Some((peripheral, name.to_string()));
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    for adapter in adapters {
        adapter.stop_scan().await?;
    }

    Ok(peripheral_meta.unwrap())
}
