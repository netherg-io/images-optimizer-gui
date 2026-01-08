use std::fs;
use std::path::Path;
use rayon::prelude::*;
use base64::{engine::general_purpose, Engine as _};
use image::ImageFormat;
use std::io::Cursor;
use std::sync::atomic::Ordering;
use tauri::{command, Emitter, State, Window};

use crate::image_ops::ImageCache;
use crate::optimizer::perform_optimization;
use crate::types::{AppState, FinalResult, OptimizeConfig, FileNode};

#[command]
pub fn get_last_result(state: State<'_, AppState>) -> Option<FinalResult> {
    let is_running = *state
        .is_processing
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    if is_running {
        return None;
    }

    let lock = state.last_result.lock().unwrap_or_else(|e| e.into_inner());
    lock.as_ref().cloned()
}

#[command]
pub fn get_processing_state(state: State<'_, AppState>) -> bool {
    *state
        .is_processing
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[command]
pub fn cancel_optimization(state: State<'_, AppState>) {
    state.should_cancel.store(true, Ordering::Relaxed);
}

#[command]
pub async fn generate_thumbnail(
    path: String,
    state: State<'_, ImageCache>,
) -> Result<String, String> {
    if let Some(cached_b64) = state.0.get(&path).await {
        return Ok(cached_b64);
    }

    let path_clone = path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let img = image::open(&path_clone).map_err(|e| e.to_string())?;
        let thumbnail = img.thumbnail(128, 128);
        let mut buffer = Cursor::new(Vec::new());
        thumbnail
            .write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        let encoded = general_purpose::STANDARD.encode(buffer.get_ref());
        Ok::<String, String>(format!("data:image/png;base64,{}", encoded))
    })
    .await
    .map_err(|e| e.to_string())??;

    state.0.insert(path, result.clone()).await;
    Ok(result)
}

#[command]
pub async fn run_optimization(
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

        state.should_cancel.store(false, Ordering::Relaxed);

        let mut last_res = state
            .last_result
            .lock()
            .map_err(|_| "Failed to lock state")?;
        *last_res = None;
    }

    let _ = window.emit("processing_state_change", true);

    let window_clone = window.clone();
    let cancel_flag = state.should_cancel.clone();

    let task_result = tauri::async_runtime::spawn_blocking(move || {
        perform_optimization(&window_clone, config, cancel_flag)
    })
    .await;

    let final_output = match task_result {
        Ok(Ok(res)) => {
            let mut last_res = state
                .last_result
                .lock()
                .map_err(|_| "Failed to lock state")?;
            *last_res = Some(res.clone());
            Ok(res)
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Task panicked or failed internally.".to_string()),
    };

    {
        let mut processing = state
            .is_processing
            .lock()
            .map_err(|_| "Failed to lock state")?;
        *processing = false;
    }

    let _ = window.emit("processing_state_change", false);

    final_output
}


fn is_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return ["jpg", "jpeg", "png", "webp", "avif"].contains(&ext_str.as_str());
    }
    false
}

fn scan_dir_parallel(path: &Path) -> Option<FileNode> {
    let Ok(entries) = fs::read_dir(path) else { return None };

    let entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();

    let children: Vec<FileNode> = entries.par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_parallel(&path)
            } else if is_image(&path) {
                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                Some(FileNode {
                    path: path.to_string_lossy().to_string(),
                    name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    is_dir: false,
                    children: None,
                    size,
                    file_count: 1,
                })
            } else {
                None
            }
        })
        .collect();

    if children.is_empty() {
        return None;
    }

    let total_size: u64 = children.iter().map(|c| c.size).sum();
    let total_count: usize = children.iter().map(|c| if c.is_dir { c.file_count } else { 1 }).sum();

    Some(FileNode {
        path: path.to_string_lossy().to_string(),
        name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        is_dir: true,
        children: Some(children),
        size: total_size,
        file_count: total_count,
    })
}

#[command]
pub async fn scan_dropped_paths(paths: Vec<String>) -> Result<Vec<FileNode>, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        paths.par_iter()
            .filter_map(|p| {
                let path = Path::new(p);
                if path.is_dir() {
                    scan_dir_parallel(path)
                } else if is_image(path) {
                    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                     Some(FileNode {
                        path: path.to_string_lossy().to_string(),
                        name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                        is_dir: false,
                        children: None,
                        size,
                        file_count: 1,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<FileNode>>()
    }).await.map_err(|e| e.to_string())?;

    Ok(result)
}
