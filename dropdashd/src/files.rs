use std::{collections::HashMap, sync::{Arc, Mutex}};
use crate::types::FileEntry;

pub type SharedFiles = Arc<Mutex<HashMap<String, FileEntry>>>;

pub fn available_files(shared_files: SharedFiles) -> Vec<(String, String)> {
    shared_files.lock().unwrap().values()
        .map(|entry| (entry.name.clone(), entry.id.clone()))
        .collect()
}

pub fn fetch_files_by_id(id: String, shared_files: SharedFiles) -> FileEntry {
    shared_files.lock().unwrap()
        .get(&id)
        .cloned()
        .expect("File not found")
}