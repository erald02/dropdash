use std::path::PathBuf;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddCommand {
    pub cmd: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}