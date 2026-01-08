#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod image_ops;
mod optimizer;
mod tools;
mod types;

use moka::future::Cache;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use commands::{
    cancel_optimization, generate_thumbnail, get_last_result, get_processing_state,
    run_optimization,
};
use image_ops::ImageCache;
use types::AppState;

fn main() {
    let cache = Cache::builder()
        .max_capacity(500)
        .time_to_idle(Duration::from_secs(30 * 60))
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            is_processing: Mutex::new(false),
            should_cancel: Arc::new(AtomicBool::new(false)),
            last_result: Mutex::new(None),
        })
        .manage(ImageCache(cache))
        .invoke_handler(tauri::generate_handler![
            run_optimization,
            cancel_optimization,
            generate_thumbnail,
            get_processing_state,
            get_last_result
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
