use std::{collections::HashMap, sync::{Arc, Mutex}};
use crate::types::{FileEntry, PasteEntry};

pub type SharedClip = Arc<Mutex<HashMap<String, PasteEntry>>>;

pub type SharedFiles = Arc<Mutex<HashMap<String, FileEntry>>>;

pub fn available_files(shared_files: SharedFiles) -> Vec<(String, String, u64)> {
    shared_files.lock().unwrap().values()
        .map(|entry| (entry.id.clone(), entry.name.clone(), entry.size.clone()))
        .collect()
}


pub fn available_copies(shared_copy: SharedClip) -> Vec<(String, u64)> {
    shared_copy.lock().unwrap().values()
        .map(|entry| (entry.id.clone(), entry.size.clone()))
        .collect()
}

pub fn fetch_files_by_id(id: String, shared_files: SharedFiles) -> FileEntry {
    shared_files.lock().unwrap()
        .get(&id)
        .cloned()
        .expect("File not found")
}