use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool; // Import AtomicBool
use std::sync::{Arc, Mutex}; // Import Arc

pub struct AppState {
    pub is_processing: Mutex<bool>,
    pub should_cancel: Arc<AtomicBool>, // Add this field
    pub last_result: Mutex<Option<FinalResult>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileTask {
    pub path: String,
    pub root: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OptimizeConfig {
    pub tasks: Vec<FileTask>,
    pub jpg_q: u8,
    pub png_min: u8,
    pub png_max: u8,
    pub webp: bool,
    pub avif: bool,
    #[serde(default = "default_true")] // На случай старых вызовов
    pub optimize_original: bool,
    pub replace: bool,
    pub output_dir: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Serialize)]
pub struct ProgressPayload {
    pub total: u64,
    pub done: u64,
    pub current_file: String,
}

#[derive(Clone, Serialize)]
pub struct FinalResult {
    pub total_files: u64,
    pub processed_files: u64,
    pub is_canceled: bool,
    pub total_size_saved: u64,
    pub duration_total: f64,
    pub duration_opt: f64,
    pub duration_webp: f64,
    pub duration_avif: f64,
    pub total_size_original: u64,
    pub total_size_optimized: u64,
    pub total_size_webp: u64,
    pub total_size_avif: u64,
}

#[derive(Default)]
pub struct FileStats {
    pub bytes_saved: u64,
    pub original_size: u64,
    pub optimized_size: u64,
    pub webp_size: u64,
    pub avif_size: u64,
    pub duration_opt: f64,
    pub duration_webp: f64,
    pub duration_avif: f64,
}
