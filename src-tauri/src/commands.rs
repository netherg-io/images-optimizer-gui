use base64::{engine::general_purpose, Engine as _};
use image::ImageFormat;
use std::io::Cursor;
use std::sync::atomic::Ordering;
use tauri::{command, Emitter, State, Window};

use crate::image_ops::ImageCache;
use crate::optimizer::perform_optimization;
use crate::types::{AppState, FinalResult, OptimizeConfig};

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
