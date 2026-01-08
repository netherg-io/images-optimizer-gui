#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod image_ops;
mod tools;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use moka::future::Cache;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State, Window};
use walkdir::WalkDir;

use image_ops::{
    generate_avif, generate_thumbnail, generate_webp, process_jpg, process_png, ImageCache,
};
use tools::get_png_tools;

struct AppState {
    is_processing: Mutex<bool>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizeConfig {
    pub paths: Vec<String>,
    pub jpg_q: u8,
    pub png_min: u8,
    pub png_max: u8,
    pub webp: bool,
    pub avif: bool,
    pub replace: bool,
    pub output_dir: Option<String>,
}

#[derive(Clone, Serialize)]
struct ProgressPayload {
    total: u64,
    done: u64,
    current_file: String,
}

#[derive(Serialize)]
struct FinalResult {
    total_files: u64,
    total_size_saved: u64,
    duration_secs: f64,
}

#[tauri::command]
async fn run_optimization(
    window: Window,
    config: OptimizeConfig,
    state: State<'_, AppState>,
) -> Result<FinalResult, String> {
    {
        let mut processing = state
            .is_processing
            .lock()
            .map_err(|_| "Failed to lock state")?;
        if *processing {
            return Err("Optimization is already in progress.".to_string());
        }
        *processing = true;
    }

    let window_clone = window.clone();
    let result =
        tauri::async_runtime::spawn_blocking(move || perform_optimization(&window_clone, config))
            .await;

    {
        let mut processing = state
            .is_processing
            .lock()
            .map_err(|_| "Failed to lock state")?;
        *processing = false;
    }

    match result {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Task panicked or failed internally.".to_string()),
    }
}

fn perform_optimization(window: &Window, config: OptimizeConfig) -> Result<FinalResult, String> {
    let total_start_time = Instant::now();

    let (_tmp_dir, pq, oxi) = get_png_tools().map_err(|e| format!("Failed to setup tools: {}", e))?;
    let supported_exts = ["png", "jpg", "jpeg"];
    let _ = window.emit("status_update", "Scanning files...");

    let is_supported = |p: &Path| {
        p.extension()
            .map(|ext| {
                supported_exts.contains(&ext.to_string_lossy().to_lowercase().as_str())
            })
            .unwrap_or(false)
    };

    let mut inputs: Vec<(PathBuf, PathBuf)> = Vec::new();

    for p_str in &config.paths {
        let root_path = Path::new(p_str);
        if !root_path.exists() {
            continue;
        }

        if root_path.is_dir() {
            for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
                if is_supported(entry.path()) {
                    inputs.push((entry.into_path(), root_path.to_path_buf()));
                }
            }
        } else if is_supported(root_path) {
            inputs.push((root_path.to_path_buf(), root_path.to_path_buf()));
        }
    }

    if inputs.is_empty() {
        return Err("No supported files found.".to_string());
    }

    let mut files_to_process: Vec<(PathBuf, PathBuf)> = Vec::new();

    for (src_path, root_source) in inputs {
        let output_path = if let Some(ref out_dir_str) = config.output_dir {
            let out_base = Path::new(out_dir_str);
            if root_source.is_dir() {
                let dir_name = root_source.file_name().unwrap_or_default();
                let relative_path = src_path.strip_prefix(&root_source).unwrap_or(&src_path);
                out_base.join(dir_name).join(relative_path)
            } else {
                out_base.join(src_path.file_name().unwrap())
            }
        } else if config.replace {
            src_path.clone()
        } else {
            let stem = src_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = src_path.extension().unwrap_or_default().to_string_lossy();
            let new_name = format!("{}__optimized.{}", stem, ext);
            src_path.parent().unwrap_or(Path::new(".")).join(new_name)
        };

        if src_path != output_path {
            if let Some(parent) = output_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(_) = fs::copy(&src_path, &output_path) {
                continue;
            }
        }

        files_to_process.push((src_path, output_path));
    }

    let total_files = files_to_process.len() as u64;
    let files_done_counter = Arc::new(AtomicU64::new(0));
    let total_input_size = AtomicU64::new(0);
    let saved_orig = AtomicU64::new(0);

    let _ = window.emit(
        "progress",
        ProgressPayload {
            total: total_files,
            done: 0,
            current_file: "Starting...".into(),
        },
    );

    files_to_process.par_iter().for_each(|(src_path, dest_path)| {
        let ext = dest_path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let original_file_size = fs::metadata(src_path).map(|m| m.len()).unwrap_or(0);
        total_input_size.fetch_add(original_file_size, Ordering::Relaxed);

        let _ = window.emit(
            "file_start",
            src_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        if config.webp || config.avif {
            if let Ok(img) = image::open(src_path) {
                if config.webp {
                    let _ = generate_webp(&img, dest_path, 75.0, original_file_size);
                }
                if config.avif {
                    let _ = generate_avif(&img, dest_path, original_file_size);
                }
            }
        }

        let s_orig = if ext == "png" {
            process_png(dest_path, &pq, &oxi, config.png_min, config.png_max)
        } else if ["jpg", "jpeg"].contains(&ext.as_str()) {
            process_jpg(dest_path, config.jpg_q)
        } else {
            0
        };

        saved_orig.fetch_add(s_orig, Ordering::Relaxed);

        let done = files_done_counter.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = window.emit(
            "progress",
            ProgressPayload {
                total: total_files,
                done,
                current_file: src_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            },
        );
    });

    Ok(FinalResult {
        total_files,
        total_size_saved: saved_orig.load(Ordering::Relaxed),
        duration_secs: total_start_time.elapsed().as_secs_f64(),
    })
}

fn main() {
    let cache = Cache::builder()
        .max_capacity(500)
        .time_to_idle(Duration::from_secs(30 * 60))
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            is_processing: Mutex::new(false),
        })
        .manage(ImageCache(cache))
        .invoke_handler(tauri::generate_handler![
            run_optimization,
            generate_thumbnail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
