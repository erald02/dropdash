use warp::Filter;
use crate::files::{SharedFiles, available_files, fetch_files_by_id};
use std::fs;

pub fn build_routes(shared_files: SharedFiles) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    health_route()
        .or(file_list_route(shared_files.clone()))
        .or(file_download_route(shared_files.clone()))
}

fn health_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("health").map(|| "OK")
}

fn file_list_route(shared_files: SharedFiles) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::end().map(move || {
        let files = available_files(shared_files.clone());
        warp::reply::json(&files)
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
