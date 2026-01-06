#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fs_utils;
mod image_ops;
mod tools;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State, Window};
use walkdir::WalkDir;

use fs_utils::copy_dir_recursive;
use image_ops::{generate_avif, generate_webp, process_jpg, process_png};
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

    let (_tmp_dir, pq, oxi) =
        get_png_tools().map_err(|e| format!("Failed to setup tools: {}", e))?;

    let supported_exts = ["png", "jpg", "jpeg"];
    let mut files_to_process: Vec<(PathBuf, PathBuf)> = Vec::new();

    let _ = window.emit("status_update", "Scanning and prepping files...");

    let is_single_dir_mode = config.paths.len() == 1 && Path::new(&config.paths[0]).is_dir();

    if is_single_dir_mode {
        let input_path = PathBuf::from(&config.paths[0]);
        let target_dir: PathBuf;

        if config.replace {
            target_dir = input_path.clone();
        } else {
            let root_name = input_path.file_name().unwrap_or_default().to_string_lossy();
            let new_name = format!("{}__optimized", root_name);
            target_dir = input_path.parent().unwrap_or(Path::new(".")).join(new_name);

            copy_dir_recursive(&input_path, &target_dir)
                .map_err(|e| format!("Copy failed: {}", e))?;
        }

        let scanned: Vec<(PathBuf, PathBuf)> = WalkDir::new(&target_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| {
                        supported_exts.contains(&ext.to_string_lossy().to_lowercase().as_str())
                    })
                    .unwrap_or(false)
            })
            .map(|e| {
                let p = e.into_path();
                (p.clone(), p)
            })
            .collect();
        files_to_process.extend(scanned);
    } else {
        for p_str in &config.paths {
            let path = Path::new(p_str);
            if !path.exists() {
                continue;
            }

            if path.is_dir() {
                let target_dir_root = if config.replace {
                    path.to_path_buf()
                } else {
                    let root_name = path.file_name().unwrap_or_default().to_string_lossy();
                    let new_name = format!("{}__optimized", root_name);
                    path.parent().unwrap_or(Path::new(".")).join(new_name)
                };

                if !config.replace {
                    if let Err(e) = copy_dir_recursive(path, &target_dir_root) {
                        eprintln!("Error copying {:?}: {}", path, e);
                        continue;
                    }
                }

                let scanned: Vec<(PathBuf, PathBuf)> = WalkDir::new(&target_dir_root)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| {
                                supported_exts
                                    .contains(&ext.to_string_lossy().to_lowercase().as_str())
                            })
                            .unwrap_or(false)
                    })
                    .map(|e| {
                        let p = e.into_path();
                        (p.clone(), p)
                    })
                    .collect();
                files_to_process.extend(scanned);
                continue;
            }

            let ext = path
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_string_lossy()
                .to_lowercase();
            if !supported_exts.contains(&ext.as_str()) {
                continue;
            }

            let target_path = if config.replace {
                path.to_path_buf()
            } else {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                let new_name = format!("{}__optimized.{}", stem, ext);
                path.parent().unwrap_or(Path::new(".")).join(new_name)
            };

            let naming_base = if config.replace {
                target_path.clone()
            } else {
                path.to_path_buf()
            };

            if !config.replace {
                if let Err(_) = fs::copy(path, &target_path) {
                    continue;
                }
            }
            files_to_process.push((target_path, naming_base));
        }
    }

    if files_to_process.is_empty() {
        return Err("No supported files found.".to_string());
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

    files_to_process.par_iter().for_each(|(path, naming_path)| {
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let original_file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        total_input_size.fetch_add(original_file_size, Ordering::Relaxed);

        let _ = window.emit(
            "file_start",
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        if config.webp || config.avif {
            if let Ok(img) = image::open(path) {
                if config.webp {
                    let _ = generate_webp(&img, naming_path, 75.0, original_file_size);
                }
                if config.avif {
                    let _ = generate_avif(&img, naming_path, original_file_size);
                }
            }
        }

        let s_orig = if ext == "png" {
            process_png(path, &pq, &oxi, config.png_min, config.png_max)
        } else {
            process_jpg(path, config.jpg_q)
        };
        saved_orig.fetch_add(s_orig, Ordering::Relaxed);

        let done = files_done_counter.fetch_add(1, Ordering::Relaxed) + 1;

        let current_file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let _ = window.emit(
            "progress",
            ProgressPayload {
                total: total_files,
                done,
                current_file: current_file_name,
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
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            is_processing: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![run_optimization])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
