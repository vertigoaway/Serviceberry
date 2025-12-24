use axum::{
    Router,
    body::Body,
    http::Request,
    Json,
    routing::{get, post},
};
use local_ip_address::local_ip;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::Serialize;
use std::{collections::HashMap, net::SocketAddr};
use tower_http::trace::TraceLayer;
use tracing::Span;

mod adapters;
use adapters::bluetooth::{BleBeacon, fetch_ble_beacons};
use adapters::wifi::{WifiBssid, fetch_wifi_stats};
use std::time::{SystemTime, UNIX_EPOCH};

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
    wifiAccessPoints: Vec<WifiBssid>,
    bluetoothBeacons: Vec<BleBeacon>,
}

#[derive(Serialize, Debug)]
struct GeoPayload {
    items: Vec<GeoItem>,
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
    let version = env!("CARGO_PKG_VERSION");

    let service_type = "Serviceberry".to_lowercase(); // Service your running, ServiceBerry in this case
    let instance_name = "Home Server"; // pretty, human readable name for the device you're using
    let hostname = "limeskey"; // actual mDNS url name you're broadcasting as / second level domain
    let lan_ip = local_ip().expect("Could not get local IP address");
    let port = 8080;
    let properties = HashMap::from([
        ("version".into(), version.into()),
        ("paths".into(), "/submit, /status, /request".into()),
    ]);

    let service_info: ServiceInfo = ServiceInfo::new(
        &format!("_{}._tcp.local.", service_type),
        instance_name,
        &format!("{}.local.", hostname),
        &lan_ip.to_string(),
        port,
        Some(properties),
    )
    .expect("Failed to create service info");

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    mdns.register(service_info)
        .expect("Failed to register mDNS service");

    println!(
        "mDNS service published as {} at {}:{}",
        instance_name,
        &lan_ip.to_string(),
        port
    );

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/submit", post(handle_submit))
        .route("/status", get(handle_status))
        .route("/request", get(handle_request))
        .layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let user_agent = request
                    .headers()
                    .get("user-agent")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("<unknown>");
                let remote_addr = request
                    .extensions()
                    .get::<SocketAddr>()
                    .map(|sa| sa.ip().to_string())
                    .unwrap_or_else(|| "<unknown>".into());

                tracing::info_span!(
                    "http-request",
                    method = %request.method(),
                    uri = %request.uri(),
                    user_agent = %user_agent,
                    remote_addr = %remote_addr,
                )
            })
            .on_request(|request: &Request<Body>, _span: &Span| {
                tracing::info!("started {} {}", request.method(), request.uri().path());
                }),
            );

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    // let submissions = warp::path!("/submit").and_then(|| async {
    //     //
    //     println!("[Server] Request received");

    //     let wifi = fetch_wifi_stats();
    //     let ble = fetch_ble_beacons().await;
    //     let position = get_position();

    //     let payload = GeoPayload {
    //         items: vec![GeoItem {
    //             timestamp: SystemTime::now()
    //                 .duration_since(UNIX_EPOCH)
    //                 .unwrap()
    //                 .as_millis(),
    //             position,
    //             wifiAccessPoints: wifi,
    //             bluetoothBeacons: ble,
    //         }],
    //     };

    //     let json = serde_json::to_string_pretty(&payload).unwrap();

    //     println!("\n================ FINAL JSON =================");
    //     println!("{}", json);
    //     println!("============================================\n");

    //     Ok::<_, warp::Rejection>(warp::reply::with_header(
    //         json,
    //         CONTENT_TYPE,
    //         "application/json",
    //     ))
    // });

    // let status = warp::path!("status").and_then(|| async {
    //     println!("[Server] /status request received");
    //     Ok::<_, warp::Rejection>("Server is running".to_string())
    // });

    // println!(
    //     "[Server] Running at http://{}:{}/network_json",
    //     lan_ip, port
    // );

    // warp::serve(submissions.or(status))
    //     .run(([0, 0, 0, 0], port))
    //     .await;
}

async fn handle_submit() {}

async fn handle_status() -> &'static str {
    "Server is running"
}

async fn handle_request() -> Json<GeoPayload> {
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

    Json(payload)
}
