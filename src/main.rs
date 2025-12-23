use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use warp::Filter;
use warp::http::header::CONTENT_TYPE;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::time;

mod adapters;
use adapters::bluetooth::BleBeacon;
use adapters::wifi::*;

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
struct Position {
    latitude: f64,
    longitude: f64,
    accuracy: f64,
    altitude: f64,
    altitudeAccuracy: f64,
    heading: f64,
    speed: f64,
    source: String,
}

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
struct GeoItem {
    timestamp: u128,
    position: Position,
    wifiAccessPoints: Vec<WifiAP>,
    bluetoothBeacons: Vec<BleBeacon>,
}

#[derive(Serialize, Debug)]
struct GeoPayload {
    items: Vec<GeoItem>,
}

async fn fetch_ble_beacons() -> Vec<BleBeacon> {
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

fn get_position() -> Position {
    Position {
        latitude: 43.731425,
        longitude: -79.607407,
        accuracy: 10.0,
        altitude: 170.0,
        altitudeAccuracy: 0.0,
        heading: 0.0,
        speed: 0.0,
        source: String::from("gps"),
    }
}

#[tokio::main]
async fn main() {
    let lan_ip = local_ip_address::local_ip().unwrap();
    let port = 3030;

    println!("[Server] LAN IP: {}", lan_ip);

    let route = warp::path!("network_json").and_then(|| async {
        println!("[Server] Request received");

        let wifi = fetch_wifi_stats();
        let ble = fetch_ble_beacons().await;
        let position = get_position();

        let payload = GeoPayload {
            items: vec![GeoItem {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
                position,
                wifiAccessPoints: wifi,
                bluetoothBeacons: ble,
            }],
        };

        let json = serde_json::to_string_pretty(&payload).unwrap();

        println!("\n================ FINAL JSON =================");
        println!("{}", json);
        println!("============================================\n");

        Ok::<_, warp::Rejection>(warp::reply::with_header(
            json,
            CONTENT_TYPE,
            "application/json",
        ))
    });

    println!(
        "[Server] Running at http://{}:{}/network_json",
        lan_ip, port
    );

    warp::serve(route).run((lan_ip, port)).await;
}
