use std::sync::mpsc::Receiver;

use btleplug::{
    api::{Central as _, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use futures::StreamExt as _;

use crate::exercise::EquipmentType;

pub async fn get_peripheral(
    equipment_type: EquipmentType,
    shutdown_rx: &mut Receiver<()>,
) -> anyhow::Result<Option<(Peripheral, String)>> {
    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await?;
    let mut events = Vec::new();
    let mut peripheral_meta: Option<(Peripheral, String)> = None;

    for adapter in &adapters {
        events.push(adapter.events().await?);
        adapter.start_scan(ScanFilter::default()).await?;
    }

    let contains_predicate = match equipment_type {
        EquipmentType::Iconsole0028Bike => "iConsole+0028",
        EquipmentType::DebugBike => "Console",
        _ => "bike",
    };

    while let Some(event) = futures::stream::iter(events.iter_mut())
        .flatten()
        .next()
        .await
    {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
        if let CentralEvent::DeviceDiscovered(id) = event {
            let central = adapters.get(1).unwrap();
            let peripheral = central.peripheral(&id).await?;
            if let Some(name) = peripheral.properties().await.unwrap().unwrap().local_name
                // todo: make this configurable ()
                && name.contains(contains_predicate)
            {
                peripheral_meta = Some((peripheral, name.to_string()));
                break;
            }
        }
    }
    for adapter in adapters {
        adapter.stop_scan().await?;
    }

    Ok(peripheral_meta)
}
