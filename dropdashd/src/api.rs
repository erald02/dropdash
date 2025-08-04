use serde_json::json;
use warp::Filter;
use crate::files::{available_files, fetch_files_by_id, available_copies, SharedClip, SharedFiles};
use std::{fs};

pub fn build_routes(shared_files: SharedFiles, shared_clips: SharedClip) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    health_route()
        .or(file_list_route(shared_files.clone(), shared_clips))
        .or(file_download_route(shared_files))
}

fn health_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("health").map(|| "OK")
}

pub fn file_list_route(shared_files: SharedFiles, shared_clips: SharedClip) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::end().map(move || {
        let files = available_files(shared_files.clone());     // Vec<(String, String)>
        let clips = available_copies(shared_clips.clone());    // Vec<(String, String, String)>

        let response = json!({
            "files": files,
            "copies": clips
        });

        warp::reply::json(&response)
    })
}

fn file_download_route(shared_files: SharedFiles) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("file" / String).map(move |id| {
        let entry = fetch_files_by_id(id, shared_files.clone());
        let file_data = fs::read(&entry.path).expect("Failed to read file!");
        let filename = entry.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download");
        warp::http::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
            .body(file_data)
            .unwrap()
    })
}
