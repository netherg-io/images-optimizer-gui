use base64::{engine::general_purpose, Engine as _};
use image::ImageFormat;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::{command, Emitter, State, Window};
use tauri_plugin_opener::OpenerExt;

use crate::image_ops::ImageCache;
use crate::optimizer::perform_optimization;
use crate::types::{AppState, FileNode, FinalResult, OptimizeConfig};

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
    let canonical = Path::new(&path)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let metadata = fs::metadata(&canonical).map_err(|error| error.to_string())?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let cache_key = format!("{}:{}:{modified}", canonical.display(), metadata.len());
    if let Some(cached_b64) = state.0.get(&cache_key).await {
        return Ok(cached_b64);
    }

    let path_clone = canonical.clone();
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

    state.0.insert(cache_key, result.clone()).await;
    Ok(result)
}

#[command]
pub fn open_local_path(window: Window, path: String) -> Result<(), String> {
    let canonical = Path::new(&path)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    window
        .opener()
        .open_path(canonical.to_string_lossy(), None::<String>)
        .map_err(|error| error.to_string())
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
        Ok(Ok(res)) => match state.last_result.lock() {
            Ok(mut last_result) => {
                *last_result = Some(res.clone());
                Ok(res)
            }
            Err(_) => Err("Failed to store optimization result.".to_string()),
        },
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Task panicked or failed internally.".to_string()),
    };

    if let Ok(mut processing) = state.is_processing.lock() {
        *processing = false;
    }

    let _ = window.emit("processing_state_change", false);

    final_output
}

fn is_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return ["jpg", "jpeg", "png"].contains(&ext_str.as_str());
    }
    false
}

fn scan_dir(path: &Path, on_image: &mut impl FnMut()) -> Result<Option<FileNode>, String> {
    let mut children = Vec::new();
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            continue;
        }
        let child_path = entry.path();
        if file_type.is_dir() {
            if let Some(child) = scan_dir(&child_path, on_image)? {
                children.push(child);
            }
        } else if file_type.is_file() && is_image(&child_path) {
            children.push(file_node(&child_path)?);
            on_image();
        }
    }

    if children.is_empty() {
        return Ok(None);
    }

    let total_size: u64 = children.iter().map(|c| c.size).sum();
    let total_count: usize = children
        .iter()
        .map(|c| if c.is_dir { c.file_count } else { 1 })
        .sum();

    Ok(Some(FileNode {
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        extension: String::new(),
        is_dir: true,
        children: Some(children),
        size: total_size,
        file_count: total_count,
    }))
}

fn file_node(path: &Path) -> Result<FileNode, String> {
    Ok(FileNode {
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        extension: path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase(),
        is_dir: false,
        children: None,
        size: fs::metadata(path).map_err(|error| error.to_string())?.len(),
        file_count: 1,
    })
}

#[command]
pub async fn scan_dropped_paths(
    window: Window,
    paths: Vec<String>,
) -> Result<Vec<FileNode>, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut nodes = Vec::new();
        let mut found = 0_u64;
        {
            let mut report = || {
                found += 1;
                if found == 1 || found.is_multiple_of(25) {
                    let _ = window.emit("scan_progress", found);
                }
            };
            for value in paths {
                let path = Path::new(&value);
                if path.is_dir() {
                    if let Some(node) = scan_dir(path, &mut report)? {
                        nodes.push(node);
                    }
                } else if is_image(path) {
                    nodes.push(file_node(path)?);
                    report();
                } else {
                    return Err(format!("Unsupported or missing path: {value}"));
                }
            }
        }
        let _ = window.emit("scan_progress", found);
        Ok::<Vec<FileNode>, String>(nodes)
    })
    .await
    .map_err(|e| e.to_string())?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_reports_each_image() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("one.png"), b"png").unwrap();
        fs::write(dir.path().join("two.jpg"), b"jpg").unwrap();
        fs::write(dir.path().join("ignore.txt"), b"text").unwrap();
        let mut found = 0;

        let node = scan_dir(dir.path(), &mut || found += 1).unwrap().unwrap();

        assert_eq!(found, 2);
        assert_eq!(node.file_count, 2);
    }
}
