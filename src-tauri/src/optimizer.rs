use image::ImageFormat;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::time::SystemTime;
use tauri::{Emitter, Window};
use walkdir::WalkDir;

use crate::image_ops::{
    encode_avif, encode_webp, optimize_jpeg, optimize_png, oriented_dimensions,
};
use crate::output::{write_validated, WriteOutcome};
use crate::tools::{get_png_tools, ToolPath};
use crate::types::{
    ExistingFilePolicy, FileOperationResult, FileOperationStatus, FinalResult, OperationError,
    OptimizationErrorCode, OptimizeConfig, OutputOperation, ProgressPayload,
};

const MAX_FILES: usize = 10_000;
const MAX_TOTAL_INPUT_SIZE: u64 = 10 * 1024 * 1024 * 1024;
const MAX_IMAGE_DIMENSION: u32 = 50_000;
const MAX_IMAGE_PIXELS: u64 = 100_000_000;

#[derive(Debug, Clone)]
struct SourceFile {
    source: PathBuf,
    root: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct PlannedOperation {
    kind: OutputOperation,
    destination: PathBuf,
    allow_overwrite: bool,
    skip_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct PlannedFile {
    source: PathBuf,
    original_size: u64,
    modified: Option<SystemTime>,
    operations: Vec<PlannedOperation>,
}

pub fn perform_optimization(
    window: &Window,
    config: OptimizeConfig,
    should_cancel: Arc<AtomicBool>,
) -> Result<FinalResult, String> {
    let started = Instant::now();
    let operation_id = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    validate_config(&config)?;
    let sources = collect_sources(&config)?;
    validate_output_directory(&config, &sources)?;
    let destination_root = if let Some(output) = config.output_dir.as_deref() {
        Some(absolute_path(Path::new(output))?)
    } else {
        sources.first().and_then(|source| {
            if source.root.is_dir() {
                Some(source.root.clone())
            } else {
                source.root.parent().map(Path::to_path_buf)
            }
        })
    }
    .map(|path| path.to_string_lossy().into_owned());
    let plan = build_output_plan(&config, &sources)?;
    let total_operations = plan
        .iter()
        .map(|file| file.operations.len() as u64)
        .sum::<u64>();
    log::info!(
        "operation_id={operation_id} started files={} operations={total_operations}",
        plan.len()
    );

    let (_temporary_tools, pngquant, oxipng) =
        get_png_tools().map_err(|error| format!("Failed to prepare PNG codecs: {error}"))?;
    let _ = window.emit("status_update", "Optimizing images safely...");
    let _ = window.emit(
        "progress",
        ProgressPayload {
            total: total_operations,
            done: 0,
            current_file: "Starting...".into(),
        },
    );

    let done = Arc::new(AtomicU64::new(0));
    let workers = std::thread::available_parallelism()
        .map(|count| (count.get() / 2).clamp(1, 4))
        .unwrap_or(1);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .map_err(|error| format!("Failed to create worker pool: {error}"))?;
    let results = pool.install(|| {
        plan.par_iter()
            .flat_map(|file| {
                process_file(
                    file,
                    &config,
                    &pngquant,
                    &oxipng,
                    window,
                    &done,
                    total_operations,
                    &should_cancel,
                )
            })
            .collect::<Vec<_>>()
    });

    let result = summarize(
        &plan,
        results,
        operation_id,
        destination_root,
        config.existing_file_policy,
        should_cancel.load(Ordering::Relaxed),
        started.elapsed().as_secs_f64(),
    );
    for failure in result
        .operations
        .iter()
        .filter(|result| result.status == FileOperationStatus::Failed)
    {
        log::warn!(
            "operation_id={} codec={:?} error={:?}",
            result.operation_id,
            failure.operation,
            failure.error_code
        );
    }
    log::info!(
        "operation_id={} completed succeeded={} failed={} skipped={} canceled={}",
        result.operation_id,
        result.succeeded_files,
        result.failed_files,
        result.skipped_files,
        result.canceled_files
    );
    Ok(result)
}

fn validate_config(config: &OptimizeConfig) -> Result<(), String> {
    if config.tasks.is_empty() {
        return Err("Select at least one image.".into());
    }
    if !config.optimize_original && !config.webp && !config.avif {
        return Err("Select at least one optimization operation.".into());
    }
    if !(1..=100).contains(&config.jpg_q) {
        return Err("JPEG quality must be between 1 and 100.".into());
    }
    if config.png_min > config.png_max || config.png_max > 100 {
        return Err("PNG quality range must be between 0 and 100.".into());
    }
    if !(1.0..=100.0).contains(&config.webp_quality) || !config.webp_quality.is_finite() {
        return Err("WebP quality must be between 1 and 100.".into());
    }
    if !(1.0..=100.0).contains(&config.avif_quality) || !config.avif_quality.is_finite() {
        return Err("AVIF quality must be between 1 and 100.".into());
    }
    if config.avif_speed > 10 {
        return Err("AVIF speed must be between 0 and 10.".into());
    }
    if config
        .output_dir
        .as_deref()
        .is_some_and(|path| path.trim().is_empty())
    {
        return Err("Choose a destination folder.".into());
    }
    if (config.replace || config.existing_file_policy == ExistingFilePolicy::Overwrite)
        && !config.overwrite_confirmed
    {
        return Err("Overwrite requires explicit confirmation.".into());
    }
    Ok(())
}

fn collect_sources(config: &OptimizeConfig) -> Result<Vec<SourceFile>, String> {
    let mut sources = Vec::new();
    let mut seen = HashSet::new();
    let mut total_size = 0_u64;

    for task in &config.tasks {
        let task_path = canonical_existing(Path::new(&task.path))?;
        let root = canonical_existing(Path::new(&task.root))?;
        if root.is_dir() && !task_path.starts_with(&root) {
            return Err("A selected image is outside its selected root.".into());
        }
        if root.is_file() && task_path != root {
            return Err("A selected image does not match its selected root.".into());
        }

        if task_path.is_dir() {
            for entry in WalkDir::new(&task_path).follow_links(false) {
                let entry = entry.map_err(|error| format!("Failed to scan input: {error}"))?;
                if entry.file_type().is_symlink() || !entry.file_type().is_file() {
                    continue;
                }
                push_source(
                    entry.path(),
                    &root,
                    &mut sources,
                    &mut seen,
                    &mut total_size,
                )?;
            }
        } else {
            push_source(&task_path, &root, &mut sources, &mut seen, &mut total_size)?;
        }
    }

    if sources.is_empty() {
        return Err("No supported images were found.".into());
    }
    sources.sort_by(|left, right| left.source.cmp(&right.source));
    Ok(sources)
}

fn push_source(
    path: &Path,
    root: &Path,
    sources: &mut Vec<SourceFile>,
    seen: &mut HashSet<String>,
    total_size: &mut u64,
) -> Result<(), String> {
    if path
        .file_name()
        .is_some_and(|name| name.to_string_lossy().contains("__optimized"))
    {
        return Ok(());
    }
    if !is_supported(path) {
        return Err(format!("Unsupported image format: {}", display_name(path)));
    }
    let source = canonical_existing(path)?;
    let key = normalized_path(&source);
    if !seen.insert(key) {
        return Err(format!("Duplicate image selected: {}", display_name(path)));
    }
    let metadata = fs::metadata(&source)
        .map_err(|error| format!("Cannot read {}: {error}", display_name(path)))?;
    let size = metadata.len();
    *total_size = total_size.saturating_add(size);
    if sources.len() >= MAX_FILES {
        return Err(format!(
            "At most {MAX_FILES} images can be processed at once."
        ));
    }
    if *total_size > MAX_TOTAL_INPUT_SIZE {
        return Err("Selected images exceed the 10 GiB safety limit.".into());
    }
    sources.push(SourceFile {
        source,
        root: root.to_path_buf(),
        size,
        modified: metadata.modified().ok(),
    });
    Ok(())
}

fn validate_output_directory(
    config: &OptimizeConfig,
    sources: &[SourceFile],
) -> Result<(), String> {
    let Some(output) = config.output_dir.as_deref() else {
        return Ok(());
    };
    let output = absolute_path(Path::new(output))?;
    if output.is_file() {
        return Err("Destination must be a directory.".into());
    }
    for root in sources
        .iter()
        .map(|source| &source.root)
        .filter(|root| root.is_dir())
    {
        if output.starts_with(root) || root.starts_with(&output) {
            return Err("Destination and input folders must not contain each other.".into());
        }
    }
    Ok(())
}

fn build_output_plan(
    config: &OptimizeConfig,
    sources: &[SourceFile],
) -> Result<Vec<PlannedFile>, String> {
    let output_dir = config
        .output_dir
        .as_deref()
        .map(|path| absolute_path(Path::new(path)))
        .transpose()?;
    let source_paths = sources
        .iter()
        .map(|source| normalized_path(&source.source))
        .collect::<HashSet<_>>();
    let mut reserved = HashSet::new();
    let mut files = Vec::with_capacity(sources.len());

    for source in sources {
        let base = resolve_base_destination(source, output_dir.as_deref(), config.replace);
        let mut candidates = Vec::new();
        if config.webp {
            candidates.push((OutputOperation::Webp, base.with_extension("webp")));
        }
        if config.avif {
            candidates.push((OutputOperation::Avif, base.with_extension("avif")));
        }
        if config.optimize_original {
            candidates.push((OutputOperation::OptimizeOriginal, base));
        }

        let mut operations = Vec::with_capacity(candidates.len());
        for (kind, mut destination) in candidates {
            let replaces_original = kind == OutputOperation::OptimizeOriginal
                && config.replace
                && destination == source.source;
            let mut skip_reason = None;

            loop {
                let key = normalized_path(&destination);
                let conflicts_source = source_paths.contains(&key) && !replaces_original;
                let conflicts_plan = reserved.contains(&key);
                let conflicts_existing = destination.exists() && !replaces_original;
                if !conflicts_source && !conflicts_plan && !conflicts_existing {
                    break;
                }

                match config.existing_file_policy {
                    ExistingFilePolicy::Rename => {
                        destination = unique_destination(&destination, &reserved, &source_paths);
                    }
                    ExistingFilePolicy::Skip => {
                        skip_reason = Some(
                            "Destination already exists or conflicts with another output.".into(),
                        );
                        break;
                    }
                    ExistingFilePolicy::Error => {
                        return Err(format!(
                            "Output conflict for {}: {}",
                            display_name(&source.source),
                            display_name(&destination)
                        ));
                    }
                    ExistingFilePolicy::Overwrite => {
                        if conflicts_source || conflicts_plan {
                            return Err(format!(
                                "Two selected inputs cannot overwrite the same output: {}",
                                display_name(&destination)
                            ));
                        }
                        break;
                    }
                }
            }

            if skip_reason.is_none() {
                reserved.insert(normalized_path(&destination));
            }
            operations.push(PlannedOperation {
                kind,
                destination,
                allow_overwrite: replaces_original
                    || config.existing_file_policy == ExistingFilePolicy::Overwrite,
                skip_reason,
            });
        }
        files.push(PlannedFile {
            source: source.source.clone(),
            original_size: source.size,
            modified: source.modified,
            operations,
        });
    }
    Ok(files)
}

fn resolve_base_destination(
    source: &SourceFile,
    output_dir: Option<&Path>,
    replace: bool,
) -> PathBuf {
    if let Some(output_dir) = output_dir {
        if source.root.is_dir() {
            let relative = source
                .source
                .strip_prefix(&source.root)
                .unwrap_or(&source.source);
            return output_dir
                .join(source.root.file_name().unwrap_or_default())
                .join(relative);
        }
        return output_dir.join(source.source.file_name().unwrap_or_default());
    }
    if replace {
        return source.source.clone();
    }
    let stem = source
        .source
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let extension = source
        .source
        .extension()
        .unwrap_or_default()
        .to_string_lossy();
    source
        .source
        .with_file_name(format!("{stem}__optimized.{extension}"))
}

fn unique_destination(
    destination: &Path,
    reserved: &HashSet<String>,
    sources: &HashSet<String>,
) -> PathBuf {
    let stem = destination
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let extension = destination.extension().map(|value| value.to_string_lossy());
    for index in 2.. {
        let name = match &extension {
            Some(extension) => format!("{stem}__{index}.{extension}"),
            None => format!("{stem}__{index}"),
        };
        let candidate = destination.with_file_name(name);
        let key = normalized_path(&candidate);
        if !candidate.exists() && !reserved.contains(&key) && !sources.contains(&key) {
            return candidate;
        }
    }
    unreachable!()
}

#[allow(clippy::too_many_arguments)]
fn process_file(
    file: &PlannedFile,
    config: &OptimizeConfig,
    pngquant: &ToolPath,
    oxipng: &ToolPath,
    window: &Window,
    done: &AtomicU64,
    total: u64,
    should_cancel: &AtomicBool,
) -> Vec<FileOperationResult> {
    if !source_is_unchanged(file) {
        return file
            .operations
            .iter()
            .map(|operation| {
                let result = failed_result(
                    file,
                    operation,
                    0.0,
                    OperationError::new(
                        OptimizationErrorCode::SourceChanged,
                        "Source changed after the output plan was created",
                    ),
                );
                emit_progress(window, done, total, &file.source);
                result
            })
            .collect();
    }
    let dimensions = oriented_dimensions(&file.source);
    file.operations
        .iter()
        .map(|operation| {
            if should_cancel.load(Ordering::Relaxed) {
                return canceled_result(file, operation);
            }
            if let Some(reason) = &operation.skip_reason {
                let result = skipped_result(file, operation, reason.clone());
                emit_progress(window, done, total, &file.source);
                return result;
            }
            let dimensions = match dimensions {
                Ok(dimensions)
                    if dimensions.0 <= MAX_IMAGE_DIMENSION
                        && dimensions.1 <= MAX_IMAGE_DIMENSION
                        && u64::from(dimensions.0) * u64::from(dimensions.1)
                            <= MAX_IMAGE_PIXELS =>
                {
                    dimensions
                }
                Ok(_) => {
                    let result = failed_result(
                        file,
                        operation,
                        0.0,
                        OperationError::new(
                            OptimizationErrorCode::ValidationFailed,
                            "Image dimensions exceed the 50,000-pixel or 100-megapixel safety limit",
                        ),
                    );
                    emit_progress(window, done, total, &file.source);
                    return result;
                }
                Err(ref error) => {
                    let result = failed_result(
                        file,
                        operation,
                        0.0,
                        OperationError::new(OptimizationErrorCode::DecodeFailed, error.to_string()),
                    );
                    emit_progress(window, done, total, &file.source);
                    return result;
                }
            };

            let started = Instant::now();
            let outcome = execute_operation(
                file,
                operation,
                config,
                pngquant,
                oxipng,
                dimensions,
                should_cancel,
            );
            let duration = started.elapsed().as_secs_f64();
            let result = match outcome {
                Ok(WriteOutcome::Written(output_size)) => FileOperationResult {
                    source: file.source.to_string_lossy().into_owned(),
                    destination: Some(operation.destination.to_string_lossy().into_owned()),
                    operation: operation.kind,
                    status: FileOperationStatus::Succeeded,
                    original_size: file.original_size,
                    output_size: Some(output_size),
                    bytes_saved: Some(file.original_size as i64 - output_size as i64),
                    duration_seconds: duration,
                    error_code: None,
                    error_message: None,
                },
                Ok(WriteOutcome::SkippedLarger) => skipped_result(
                    file,
                    operation,
                    "Output was not smaller than the source.".into(),
                ),
                Err(error) if error.code == OptimizationErrorCode::Canceled => {
                    return canceled_result(file, operation)
                }
                Err(error) => failed_result(file, operation, duration, error),
            };
            emit_progress(window, done, total, &file.source);
            result
        })
        .collect()
}

fn source_is_unchanged(file: &PlannedFile) -> bool {
    fs::metadata(&file.source).is_ok_and(|metadata| {
        metadata.len() == file.original_size && metadata.modified().ok() == file.modified
    })
}

fn execute_operation(
    file: &PlannedFile,
    operation: &PlannedOperation,
    config: &OptimizeConfig,
    pngquant: &ToolPath,
    oxipng: &ToolPath,
    dimensions: (u32, u32),
    should_cancel: &AtomicBool,
) -> Result<WriteOutcome, OperationError> {
    let format = match operation.kind {
        OutputOperation::Webp => ImageFormat::WebP,
        OutputOperation::Avif => ImageFormat::Avif,
        OutputOperation::OptimizeOriginal => source_format(&file.source)?,
    };
    write_validated(
        &operation.destination,
        format,
        dimensions,
        file.original_size,
        config.skip_if_larger,
        operation.allow_overwrite,
        |temporary| match operation.kind {
            OutputOperation::Webp => encode_webp(&file.source, temporary, config.webp_quality),
            OutputOperation::Avif => encode_avif(
                &file.source,
                temporary,
                config.avif_quality,
                config.avif_speed,
            ),
            OutputOperation::OptimizeOriginal => match format {
                ImageFormat::Png => optimize_png(
                    &file.source,
                    temporary,
                    pngquant,
                    oxipng,
                    config.png_min,
                    config.png_max,
                    should_cancel,
                ),
                ImageFormat::Jpeg => optimize_jpeg(&file.source, temporary, config.jpg_q),
                _ => Err(OperationError::new(
                    OptimizationErrorCode::UnsupportedFormat,
                    "Unsupported source format",
                )),
            },
        },
    )
}

fn emit_progress(window: &Window, done: &AtomicU64, total: u64, source: &Path) {
    let done = done.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = window.emit(
        "progress",
        ProgressPayload {
            total,
            done,
            current_file: display_name(source),
        },
    );
}

fn canceled_result(file: &PlannedFile, operation: &PlannedOperation) -> FileOperationResult {
    FileOperationResult {
        source: file.source.to_string_lossy().into_owned(),
        destination: Some(operation.destination.to_string_lossy().into_owned()),
        operation: operation.kind,
        status: FileOperationStatus::Canceled,
        original_size: file.original_size,
        output_size: None,
        bytes_saved: None,
        duration_seconds: 0.0,
        error_code: Some(OptimizationErrorCode::Canceled),
        error_message: Some("Optimization was canceled.".into()),
    }
}

fn skipped_result(
    file: &PlannedFile,
    operation: &PlannedOperation,
    reason: String,
) -> FileOperationResult {
    FileOperationResult {
        source: file.source.to_string_lossy().into_owned(),
        destination: Some(operation.destination.to_string_lossy().into_owned()),
        operation: operation.kind,
        status: FileOperationStatus::Skipped,
        original_size: file.original_size,
        output_size: None,
        bytes_saved: None,
        duration_seconds: 0.0,
        error_code: None,
        error_message: Some(reason),
    }
}

fn failed_result(
    file: &PlannedFile,
    operation: &PlannedOperation,
    duration_seconds: f64,
    error: OperationError,
) -> FileOperationResult {
    FileOperationResult {
        source: file.source.to_string_lossy().into_owned(),
        destination: Some(operation.destination.to_string_lossy().into_owned()),
        operation: operation.kind,
        status: FileOperationStatus::Failed,
        original_size: file.original_size,
        output_size: None,
        bytes_saved: None,
        duration_seconds,
        error_code: Some(error.code),
        error_message: Some(error.message),
    }
}

fn summarize(
    plan: &[PlannedFile],
    operations: Vec<FileOperationResult>,
    operation_id: String,
    destination_root: Option<String>,
    existing_file_policy: ExistingFilePolicy,
    cancel_requested: bool,
    duration_total: f64,
) -> FinalResult {
    let mut per_source: HashMap<&str, Vec<FileOperationStatus>> = HashMap::new();
    for result in &operations {
        per_source
            .entry(&result.source)
            .or_default()
            .push(result.status);
    }

    let mut succeeded_files = 0;
    let mut failed_files = 0;
    let mut skipped_files = 0;
    let mut canceled_files = 0;
    for statuses in per_source.values() {
        if statuses.contains(&FileOperationStatus::Failed) {
            failed_files += 1;
        } else if statuses.contains(&FileOperationStatus::Canceled) {
            canceled_files += 1;
        } else if statuses
            .iter()
            .all(|status| *status == FileOperationStatus::Skipped)
        {
            skipped_files += 1;
        } else {
            succeeded_files += 1;
        }
    }

    let sum_size = |kind| {
        operations
            .iter()
            .filter(|result| {
                result.operation == kind && result.status == FileOperationStatus::Succeeded
            })
            .filter_map(|result| result.output_size)
            .sum()
    };
    let sum_duration = |kind| {
        operations
            .iter()
            .filter(|result| result.operation == kind)
            .map(|result| result.duration_seconds)
            .sum()
    };
    let total_size_saved = operations
        .iter()
        .filter(|result| result.operation == OutputOperation::OptimizeOriginal)
        .filter_map(|result| result.bytes_saved)
        .filter(|saved| *saved > 0)
        .map(|saved| saved as u64)
        .sum();

    FinalResult {
        operation_id,
        destination_root,
        total_files: plan.len() as u64,
        total_operations: operations.len() as u64,
        processed_files: succeeded_files + failed_files + skipped_files,
        succeeded_files,
        failed_files,
        skipped_files,
        canceled_files,
        is_canceled: cancel_requested || canceled_files > 0,
        existing_file_policy,
        total_size_saved,
        duration_total,
        duration_opt: sum_duration(OutputOperation::OptimizeOriginal),
        duration_webp: sum_duration(OutputOperation::Webp),
        duration_avif: sum_duration(OutputOperation::Avif),
        total_size_original: plan.iter().map(|file| file.original_size).sum(),
        total_size_optimized: sum_size(OutputOperation::OptimizeOriginal),
        total_size_webp: sum_size(OutputOperation::Webp),
        total_size_avif: sum_size(OutputOperation::Avif),
        operations,
    }
}

fn source_format(path: &Path) -> Result<ImageFormat, OperationError> {
    match path
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => Ok(ImageFormat::Png),
        Some("jpg" | "jpeg") => Ok(ImageFormat::Jpeg),
        _ => Err(OperationError::new(
            OptimizationErrorCode::UnsupportedFormat,
            "Only PNG and JPEG inputs are supported",
        )),
    }
}

fn is_supported(path: &Path) -> bool {
    matches!(
        path.extension()
            .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg")
    )
}

fn canonical_existing(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("Cannot access {}: {error}", display_name(path)))
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return canonical_existing(path);
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path)
    };
    let parent = absolute
        .parent()
        .ok_or_else(|| "Destination has no parent directory.".to_string())?;
    let canonical_parent = canonical_existing(parent)?;
    Ok(canonical_parent.join(absolute.file_name().unwrap_or_default()))
}

fn normalized_path(path: &Path) -> String {
    let value = path.to_string_lossy().replace('/', "\\");
    if cfg!(target_os = "windows") {
        value.to_lowercase()
    } else {
        value
    }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileTask;

    fn config(tasks: Vec<FileTask>) -> OptimizeConfig {
        OptimizeConfig {
            tasks,
            jpg_q: 80,
            png_min: 65,
            png_max: 80,
            webp: true,
            avif: false,
            optimize_original: false,
            replace: false,
            output_dir: None,
            existing_file_policy: ExistingFilePolicy::Error,
            overwrite_confirmed: false,
            webp_quality: 80.0,
            avif_quality: 65.0,
            avif_speed: 4,
            skip_if_larger: true,
        }
    }

    #[test]
    fn same_stem_conversion_collision_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let jpg = dir.path().join("photo.jpg");
        let png = dir.path().join("photo.png");
        fs::write(&jpg, b"jpg").unwrap();
        fs::write(&png, b"png").unwrap();
        let root = dir.path().to_string_lossy().into_owned();
        let config = config(vec![
            FileTask {
                path: jpg.to_string_lossy().into_owned(),
                root: root.clone(),
            },
            FileTask {
                path: png.to_string_lossy().into_owned(),
                root,
            },
        ]);
        let sources = collect_sources(&config).unwrap();

        assert!(build_output_plan(&config, &sources)
            .unwrap_err()
            .contains("Output conflict"));
    }

    #[test]
    fn rename_policy_makes_same_stem_outputs_unique() {
        let dir = tempfile::tempdir().unwrap();
        let jpg = dir.path().join("photo.jpg");
        let png = dir.path().join("photo.png");
        fs::write(&jpg, b"jpg").unwrap();
        fs::write(&png, b"png").unwrap();
        let root = dir.path().to_string_lossy().into_owned();
        let mut config = config(vec![
            FileTask {
                path: jpg.to_string_lossy().into_owned(),
                root: root.clone(),
            },
            FileTask {
                path: png.to_string_lossy().into_owned(),
                root,
            },
        ]);
        config.existing_file_policy = ExistingFilePolicy::Rename;
        let sources = collect_sources(&config).unwrap();
        let plan = build_output_plan(&config, &sources).unwrap();

        assert_ne!(
            plan[0].operations[0].destination,
            plan[1].operations[0].destination
        );
    }

    #[test]
    fn overwrite_requires_confirmation() {
        let mut config = config(Vec::new());
        config.existing_file_policy = ExistingFilePolicy::Overwrite;
        assert_eq!(
            validate_config(&config).unwrap_err(),
            "Select at least one image."
        );
        config.tasks.push(FileTask {
            path: "image.jpg".into(),
            root: "image.jpg".into(),
        });
        assert_eq!(
            validate_config(&config).unwrap_err(),
            "Overwrite requires explicit confirmation."
        );
    }

    #[test]
    fn planned_source_change_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("photo.jpg");
        fs::write(&source, b"first").unwrap();
        let mut config = config(vec![FileTask {
            path: source.to_string_lossy().into_owned(),
            root: source.to_string_lossy().into_owned(),
        }]);
        config.existing_file_policy = ExistingFilePolicy::Rename;
        let sources = collect_sources(&config).unwrap();
        let plan = build_output_plan(&config, &sources).unwrap();
        fs::write(&source, b"changed-size").unwrap();

        assert!(!source_is_unchanged(&plan[0]));
    }
}
