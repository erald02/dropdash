use clap::Parser;
use std::{fs, net::{IpAddr, SocketAddr}, path::Path, time::Duration};
use warp::Filter;
use tokio::time;
use qrcode::QrCode;
use qrcode::render::unicode;
use local_ip_address::local_ip;
use tokio::sync::oneshot;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of file to share
    #[arg(short, long)]
    path: String,

    /// How long to publish file for
    #[arg(short, long, default_value = "1m")]
    time: String,

    /// Make it oneshot (close after downloaded)
    #[arg(short, long, default_value_t = false)]
    oneshot: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let file_name = Path::new(&args.path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let file_contents = fs::read(&args.path)
        .expect("Failed to read file!");

    if args.oneshot {
        oneshot_server(file_contents, file_name, args.time).await;
    } else {
        up_server(file_contents, file_name, args.time).await;
    }
}

async fn oneshot_server(file_contents: Vec<u8>, file_name: String, timeout: String) {
    use std::sync::{Arc, Mutex};

    let addr = {
        let ip = local_ip().expect("Could not get local IP");
        let addr: SocketAddr = (ip, 8080).into();

        let url = format!("http://{}:{}", ip, 8080);
        println!("File available at: {}", url);

        let code = QrCode::new(url.as_bytes()).unwrap();
        let image = code.render::<unicode::Dense1x2>()
            .quiet_zone(false)
            .build();
        println!("{}", image);

        addr
    };

    let duration: Duration = humantime::parse_duration(&timeout)
        .expect("Invalid duration");

    let (download_tx, download_rx) = oneshot::channel::<()>();
    let shutdown_signal = Arc::new(Mutex::new(Some(download_tx)));

    let filter = warp::any().map({
        let shutdown_signal = shutdown_signal.clone();
        let file_contents = file_contents.clone();
        let file_name = file_name.clone();

        move || {
            if let Some(tx) = shutdown_signal.lock().unwrap().take() {
                let _ = tx.send(());
            }

            warp::http::Response::builder()
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", file_name),
                )
                .body(file_contents.clone())
                .unwrap()
        }
    });

    let shutdown = async {
        tokio::select! {
            _ = download_rx => {
                println!("File downloaded — shutting down.");
            }
            _ = time::sleep(duration) => {
                println!("Timeout reached — shutting down.");
            }
        }
    };

    warp::serve(filter)
        .bind_with_graceful_shutdown(addr, shutdown)
        .1
        .await;
}

async fn up_server(file_contents: Vec<u8>, file_name: String, timeout: String) {
    let (shutdown_signal, addr) = prepare_stuff(&timeout);

    let file = warp::any().map(move || {
        warp::http::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", file_name),
            )
            .body(file_contents.clone())
            .unwrap()
    });

    warp::serve(file)
        .bind_with_graceful_shutdown(addr, shutdown_signal)
        .1
        .await;
}

fn prepare_stuff(timeout: &str) -> (impl std::future::Future<Output = ()>, SocketAddr) {
    let ip: IpAddr = local_ip().expect("Could not get local IP");
    let addr: SocketAddr = (ip, 8080).into();

    let url = format!("http://{}:{}", ip, 8080);
    println!("📡 File available at: {}", url);

    let code = QrCode::new(url.as_bytes()).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .quiet_zone(false)
        .build();
    println!("{}", image);

    let duration: Duration = humantime::parse_duration(timeout)
        .expect("Invalid duration");

    let shutdown = async move {
        time::sleep(duration).await;
        println!("⏱ Timeout reached. Shutting down.");
    };

    (shutdown, addr)
}
