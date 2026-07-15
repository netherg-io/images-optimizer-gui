use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Clone)]
pub struct FileNode {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
    pub size: u64,
    pub file_count: usize,
}

pub struct AppState {
    pub is_processing: Mutex<bool>,
    pub should_cancel: Arc<AtomicBool>,
    pub last_result: Mutex<Option<FinalResult>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileTask {
    pub path: String,
    pub root: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExistingFilePolicy {
    Error,
    Skip,
    #[default]
    Rename,
    Overwrite,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OptimizeConfig {
    pub tasks: Vec<FileTask>,
    pub jpg_q: u8,
    pub png_min: u8,
    pub png_max: u8,
    pub webp: bool,
    pub avif: bool,
    #[serde(default = "default_true")]
    pub optimize_original: bool,
    pub replace: bool,
    pub output_dir: Option<String>,
    #[serde(default)]
    pub existing_file_policy: ExistingFilePolicy,
    #[serde(default)]
    pub overwrite_confirmed: bool,
    #[serde(default = "default_webp_quality")]
    pub webp_quality: f32,
    #[serde(default = "default_avif_quality")]
    pub avif_quality: f32,
    #[serde(default = "default_avif_speed")]
    pub avif_speed: u8,
    #[serde(default = "default_true")]
    pub skip_if_larger: bool,
}

fn default_true() -> bool {
    true
}

fn default_webp_quality() -> f32 {
    75.0
}

fn default_avif_quality() -> f32 {
    65.0
}

fn default_avif_speed() -> u8 {
    4
}

#[derive(Clone, Serialize)]
pub struct ProgressPayload {
    pub total: u64,
    pub done: u64,
    pub current_file: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputOperation {
    OptimizeOriginal,
    Webp,
    Avif,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileOperationStatus {
    Succeeded,
    Failed,
    Skipped,
    Canceled,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationErrorCode {
    SourceMissing,
    SourceChanged,
    UnsupportedFormat,
    DestinationConflict,
    PermissionDenied,
    ToolMissing,
    ToolFailed,
    ProcessTimeout,
    DecodeFailed,
    EncodeFailed,
    ValidationFailed,
    DestinationWriteFailed,
    Canceled,
}

#[derive(Debug, Clone)]
pub struct OperationError {
    pub code: OptimizationErrorCode,
    pub message: String,
}

impl OperationError {
    pub fn new(code: OptimizationErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn io(error: std::io::Error, fallback: OptimizationErrorCode) -> Self {
        let code = match error.kind() {
            std::io::ErrorKind::NotFound => OptimizationErrorCode::SourceMissing,
            std::io::ErrorKind::PermissionDenied => OptimizationErrorCode::PermissionDenied,
            _ => fallback,
        };
        Self::new(code, error.to_string())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct FileOperationResult {
    pub source: String,
    pub destination: Option<String>,
    pub operation: OutputOperation,
    pub status: FileOperationStatus,
    pub original_size: u64,
    pub output_size: Option<u64>,
    pub bytes_saved: Option<i64>,
    pub duration_seconds: f64,
    pub error_code: Option<OptimizationErrorCode>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FinalResult {
    pub operation_id: String,
    pub destination_root: Option<String>,
    pub total_files: u64,
    pub total_operations: u64,
    pub processed_files: u64,
    pub succeeded_files: u64,
    pub failed_files: u64,
    pub skipped_files: u64,
    pub canceled_files: u64,
    pub is_canceled: bool,
    pub existing_file_policy: ExistingFilePolicy,
    pub operations: Vec<FileOperationResult>,
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
