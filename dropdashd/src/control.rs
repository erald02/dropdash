use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use uuid::Uuid;
use crate::types::{FileEntry, AddCommand};
use crate::files::SharedFiles;

pub async fn handle_connection(mut socket: TcpStream, shared_files: SharedFiles) {
    let mut reader = BufReader::new(&mut socket);
    let mut line = String::new();
    
    if reader.read_line(&mut line).await.is_ok() {
        if let Ok(add_cmd) = serde_json::from_str::<AddCommand>(&line) {
            if add_cmd.cmd == "add" {
                let path = PathBuf::from(&add_cmd.path);
                let size = add_cmd.size;
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let id = Uuid::new_v4().to_string();
                
                shared_files.lock().unwrap().insert(
                    id.clone(),
                    FileEntry { id: id.clone(), name: name.clone(), path, size},
                );
                
                let response = serde_json::json!({ "status": "ok", "id": id, "name": name });
                let response_str = format!("{}\n", response);
                let _ = socket.write_all(response_str.as_bytes()).await;
            }
        }
    }
}

pub async fn start_control_server(shared_files: SharedFiles) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:59123").await.expect("failed to bind TCP");
    println!("📡 Listening for control commands on 127.0.0.1:59123");
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let files = shared_files.clone();
        tokio::spawn(async move {
            handle_connection(socket, files).await;
        });
    }
}
