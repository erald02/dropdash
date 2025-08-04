use std::path::PathBuf;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddCommand {
    pub cmd: String,
    pub path: String,
    pub size: u64,
}

#[derive(Deserialize)]
pub struct PasteCommand {
    pub cmd: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}


#[derive(Debug, Clone)]
pub struct PasteEntry {
    pub id: String,
    pub content: String,
    pub size: u64,
}