use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use serde::Serialize;
use std::time::Duration;
use tokio::time;

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
pub struct BleBeacon {
    pub macAddress: String,
    pub signalStrength: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub async fn fetch_ble_beacons() -> Vec<BleBeacon> {
    println!("[BLE] Starting BLE scan...");
    let mut beacons = vec![];

    let manager = match Manager::new().await {
        Ok(m) => m,
        Err(e) => {
            println!("[BLE] Manager error: {:?}", e);
            return beacons;
        }
    };

    let adapters = manager.adapters().await.unwrap_or_default();
    let adapter = match adapters.into_iter().next() {
        Some(a) => a,
        None => {
            println!("[BLE] No adapters found");
            return beacons;
        }
    };

    if let Err(e) = adapter.start_scan(ScanFilter::default()).await {
        println!("[BLE] Scan failed: {:?}", e);
        return beacons;
    }

    time::sleep(Duration::from_secs(5)).await;

    let peripherals = adapter.peripherals().await.unwrap_or_default();

    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            let beacon = BleBeacon {
                macAddress: p.address().to_string(),
                signalStrength: props.rssi.unwrap_or(-100) as i32,
                name: props.local_name.filter(|n| !n.is_empty()),
            };

            println!("[BLE] {:?}", beacon);
            beacons.push(beacon);
        }
    }

    println!("[BLE] Total beacons: {}", beacons.len());
    beacons
}
