use std::{collections::HashMap, sync::{Arc, Mutex}};
use qrcode::{QrCode, render::unicode};

mod types;
mod files;
mod control;
mod api;
mod mdns;

use files::SharedFiles;
use control::start_control_server;
use api::build_routes;
use mdns::register_mdns;

#[tokio::main]
async fn main() {
    println!("🚀 dropdashd started");

    let shared_files: SharedFiles = Arc::new(Mutex::new(HashMap::new()));
    let ip = local_ip_address::local_ip().expect("Could not get local IP");
    let addr: std::net::SocketAddr = (ip, 8080).into();

    register_mdns(ip, addr.port());

    println!("📡 Serving at http://{}:{}", ip, addr.port());
    let code = QrCode::new(format!("http://{}:{}", ip, addr.port())).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .quiet_zone(false)
        .build();
    println!("{}", image);

    tokio::spawn(start_control_server(shared_files.clone()));

    let routes = build_routes(shared_files.clone());
    warp::serve(routes).run(addr).await;
}
