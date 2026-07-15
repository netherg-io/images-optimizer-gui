#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod image_ops;
mod optimizer;
mod output;
mod tools;
mod types;

use moka::future::Cache;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use commands::{
    cancel_optimization, generate_thumbnail, get_last_result, get_processing_state,
    open_local_path, run_optimization, scan_dropped_paths,
};
use image_ops::ImageCache;
use types::AppState;

fn main() {
    let cache = Cache::builder()
        .max_capacity(32 * 1024 * 1024)
        .weigher(|_: &String, value: &String| value.len().min(u32::MAX as usize) as u32)
        .time_to_idle(Duration::from_secs(30 * 60))
        .build();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(3))
                .max_file_size(1_000_000)
                .clear_targets()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("images-optimizer".into()),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
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
            get_last_result,
            scan_dropped_paths,
            open_local_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
