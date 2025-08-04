use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use uuid::Uuid;
use crate::types::{AddCommand, FileEntry, PasteCommand, PasteEntry};
use crate::files::{SharedFiles, SharedClip};

pub async fn handle_connection(mut socket: TcpStream, shared_files: &SharedFiles, shared_clip: &SharedClip) {
    let mut reader = BufReader::new(&mut socket);
    let mut line = String::new();
    
    if reader.read_line(&mut line).await.is_ok() {
        if let Ok(cmd) = serde_json::from_str::<AddCommand>(&line) {
            if cmd.cmd == "add" {
                let path = PathBuf::from(&cmd.path);
                let size = cmd.size;
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
        else if let Ok(paste_cmd) = serde_json::from_str::<PasteCommand>(&line) {
            if paste_cmd.cmd == "paste" {
                let id = Uuid::new_v4().to_string();
                let name = format!("clipboard-{}", &id[..8]);
                
                shared_clip.lock().unwrap().insert(
                    id.clone(),
                    PasteEntry { 
                        id: id.clone(), 
                        content: paste_cmd.content.clone(),
                        size: paste_cmd.content.len() as u64
                    },
                );
                
                let response = serde_json::json!({ 
                    "status": "ok", 
                    "id": id, 
                    "name": name,
                    "type": "clipboard" 
                });
                let response_str = format!("{}\n", response);
                let _ = socket.write_all(response_str.as_bytes()).await;
            }
        }
    }
}

pub async fn start_control_server(shared_files: SharedFiles, shared_clips: SharedClip) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:59123").await.expect("failed to bind TCP");
    println!("📡 Listening for control commands on 127.0.0.1:59123");
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let files = shared_files.clone();
        let clips = shared_clips.clone();
        tokio::spawn(async move {
            handle_connection(socket, &files, &clips).await;
        });
    }
}
