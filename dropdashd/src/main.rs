use warp::Filter;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use gethostname::gethostname;
use qrcode::QrCode;
use qrcode::render::unicode;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use tokio::net::{TcpListener, TcpStream};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, sync::{Arc, Mutex}};
use uuid::Uuid;

#[derive(Deserialize)]
struct AddCommand {
    cmd: String,
    path: String,
}

#[derive(Debug)]
struct FileEntry {
    id: String,
    name: String,
    path: PathBuf,
}

type SharedFiles = Arc<Mutex<HashMap<String, FileEntry>>>;

async fn handle_connection(mut socket: TcpStream, shared_files: SharedFiles) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut reader = BufReader::new(&mut socket);
    let mut line = String::new();
    
    if reader.read_line(&mut line).await.is_ok() {
        if let Ok(add_cmd) = serde_json::from_str::<AddCommand>(&line) {
            if add_cmd.cmd == "add" {
                let path = PathBuf::from(&add_cmd.path);
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let id = Uuid::new_v4().to_string();
    
                shared_files.lock().unwrap().insert(
                    id.clone(),
                    FileEntry { id: id.clone(), name: name.clone(), path },
                );
    
                let response = serde_json::json!({ "status": "ok", "id": id, "name": name });
                let response_str = format!("{response}\n"); 
                let _ = socket.write_all(response_str.as_bytes()).await;
            }
        }
    }
}

async fn start_control_server(shared_files: SharedFiles) {
    let listener = TcpListener::bind("127.0.0.1:59123").await.expect("failed to bind TCP");

    println!("📡 Listening for control commands on 127.0.0.1:59123");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let files = shared_files.clone();
        tokio::spawn(async move {
            handle_connection(socket, files).await;
        });
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 dropdashd started");
    
    let ip = local_ip_address::local_ip().expect("Could not get local IP");
    let addr: std::net::SocketAddr = (ip, 8080).into();
    
    let health = warp::path("health").map(|| "OK");
    let responder = ServiceDaemon::new().expect("Failed to create mDNS responder");
    
    let service_type = "_http._tcp.local.";
    let instance_name = format!("{}-dropdash", gethostname().to_string_lossy());
    let hostname = format!("{}.local.", &instance_name); 
    let host_ipv4 = ip.to_string();
    let properties = [("property_1", "test"), ("property_2", "1234")];
    
    let service_info = ServiceInfo::new(
        service_type,
        &instance_name,
        &hostname,
        host_ipv4,
        addr.port(),
        &properties[..],
    )
    .expect("Invalid service info");
    
    responder
        .register(service_info)
        .expect("Failed to register mDNS service");
    
    println!("📡 Serving at http://{}:8080", ip);
    println!("🔊 Broadcasting as http://{}.local:8080", &instance_name);
    let code = QrCode::new(format!("http://{}:8080", ip)).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .quiet_zone(false)
        .build();
    println!("{}", image);
    let shared_files: SharedFiles = Arc::new(Mutex::new(HashMap::new()));
    tokio::spawn(start_control_server(shared_files.clone()));
    warp::serve(health).run(addr).await;
}