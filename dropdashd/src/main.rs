use warp::Filter;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use gethostname::gethostname;
use qrcode::QrCode;
use qrcode::render::unicode;

#[tokio::main]
async fn main() {
    println!("🚀 dropdashd started");
    
    let ip = local_ip_address::local_ip().expect("Could not get local IP");
    let addr: std::net::SocketAddr = (ip, 8080).into();
    
    let health = warp::path("health").map(|| "OK");
    let responder = ServiceDaemon::new().expect("Failed to create mDNS responder");
    
    let service_type = "_http._tcp.local.";
    let instance_name = format!("{}-dropdash", gethostname().to_string_lossy());
    let hostname = format!("{}.local.", &instance_name); // Add .local. suffix
    let host_ipv4 = ip.to_string();
    let properties = [("property_1", "test"), ("property_2", "1234")];
    
    let service_info = ServiceInfo::new(
        service_type,
        &instance_name,
        &hostname, // Use hostname with .local. suffix
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
    
    warp::serve(health).run(addr).await;
}