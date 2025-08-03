use mdns_sd::{ServiceDaemon, ServiceInfo};
use gethostname::gethostname;
use std::net::IpAddr;

pub fn register_mdns(ip: IpAddr, port: u16) {
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
        port,
        &properties[..],
    )
    .expect("Invalid service info");

    responder
        .register(service_info)
        .expect("Failed to register mDNS service");

    println!("🔊 Broadcasting as http://{}.local:{}", instance_name, port);
}