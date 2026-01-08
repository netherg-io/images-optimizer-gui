use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{Emitter, Window};
use walkdir::WalkDir;

use crate::image_ops::{generate_avif, generate_webp, process_jpg, process_png};
use crate::tools::{get_png_tools, ToolPath};
use crate::types::{FileStats, FinalResult, OptimizeConfig, ProgressPayload};

pub fn perform_optimization(
    window: &Window,
    config: OptimizeConfig,
    should_cancel: Arc<AtomicBool>,
) -> Result<FinalResult, String> {
    let start_time = Instant::now();

    let (_tmp_dir, pq, oxi) =
        get_png_tools().map_err(|e| format!("Failed to setup tools: {}", e))?;
    let _ = window.emit("status_update", "Preparing files...");

    let file_tasks = collect_file_tasks(&config)?;
    let total_files_count = file_tasks.len() as u64;

    let _ = window.emit(
        "progress",
        ProgressPayload {
            total: total_files_count,
            done: 0,
            current_file: "Starting...".into(),
        },
    );

    let done_counter = Arc::new(AtomicU64::new(0));

    let results: Vec<FileStats> = file_tasks
        .par_iter()
        .map(|(src, dest)| {
            if should_cancel.load(Ordering::Relaxed) {
                return FileStats::default();
            }

            process_single_file(
                src,
                dest,
                &config,
                &pq,
                &oxi,
                window,
                &done_counter,
                total_files_count,
                &should_cancel,
            )
        })
        .collect();

    let is_canceled = should_cancel.load(Ordering::Relaxed);
    let duration_total_wall = start_time.elapsed().as_secs_f64();

    let mut total_saved = 0;
    let mut total_original = 0;
    let mut total_optimized = 0;
    let mut total_webp_size = 0;
    let mut total_avif_size = 0;

    let mut sum_cpu_opt = 0.0;
    let mut sum_cpu_webp = 0.0;
    let mut sum_cpu_avif = 0.0;

    for s in results {
        total_saved += s.bytes_saved;
        total_original += s.original_size;
        total_optimized += s.optimized_size;
        total_webp_size += s.webp_size;
        total_avif_size += s.avif_size;

        sum_cpu_opt += s.duration_opt;
        sum_cpu_webp += s.duration_webp;
        sum_cpu_avif += s.duration_avif;
    }

    let total_cpu_time = sum_cpu_opt + sum_cpu_webp + sum_cpu_avif;
    let factor = if total_cpu_time > 0.0001 {
        duration_total_wall / total_cpu_time
    } else {
        0.0
    };

    let processed_count = done_counter.load(Ordering::Relaxed);

    Ok(FinalResult {
        total_files: total_files_count,
        processed_files: processed_count,
        is_canceled,
        total_size_saved: total_saved,
        duration_total: duration_total_wall,
        duration_opt: sum_cpu_opt * factor,
        duration_webp: sum_cpu_webp * factor,
        duration_avif: sum_cpu_avif * factor,
        total_size_original: total_original,
        total_size_optimized: total_optimized,
        total_size_webp: total_webp_size,
        total_size_avif: total_avif_size,
    })
}

fn collect_file_tasks(config: &OptimizeConfig) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let mut tasks = Vec::new();
    let supported_exts = ["png", "jpg", "jpeg"];

    let is_supported = |p: &Path| {
        p.extension()
            .map(|ext| supported_exts.contains(&ext.to_string_lossy().to_lowercase().as_str()))
            .unwrap_or(false)
    };

    for task in &config.tasks {
        let clean_path = task.path.replace("\"", "");
        let src_path = Path::new(&clean_path);
        let root_path = Path::new(&task.root);

        if src_path.to_string_lossy().contains("__optimized") {
            continue;
        }

        if !src_path.exists() {
            continue;
        }

        if src_path.is_dir() {
            for entry in WalkDir::new(src_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && is_supported(path) {
                    let dest = resolve_output_path(path, root_path, config);
                    tasks.push((path.to_path_buf(), dest));
                }
            }
        } else if src_path.is_file() && is_supported(src_path) {
            let dest = resolve_output_path(src_path, root_path, config);
            tasks.push((src_path.to_path_buf(), dest));
        }
    }

    if tasks.is_empty() {
        return Err("No supported files found.".to_string());
    }

    tasks.sort_by(|a, b| a.0.cmp(&b.0));
    tasks.dedup_by(|a, b| a.0 == b.0);

    Ok(tasks)
}

fn resolve_output_path(src: &Path, root_source: &Path, config: &OptimizeConfig) -> PathBuf {
    if let Some(ref out_dir_str) = config.output_dir {
        let out_base = Path::new(out_dir_str);

        if root_source.is_dir() {
            let relative = src.strip_prefix(root_source).unwrap_or(src);
            let root_name = root_source.file_name().unwrap_or_default();
            out_base.join(root_name).join(relative)
        } else {
            out_base.join(src.file_name().unwrap_or_default())
        }
    } else if config.replace {
        src.to_path_buf()
    } else {
        let stem = src.file_stem().unwrap_or_default().to_string_lossy();
        let ext = src.extension().unwrap_or_default().to_string_lossy();
        let new_name = format!("{}__optimized.{}", stem, ext);
        src.parent().unwrap_or(Path::new(".")).join(new_name)
    }
}

#[allow(clippy::too_many_arguments)]
fn process_single_file(
    src: &Path,
    dest: &Path,
    config: &OptimizeConfig,
    pq: &ToolPath,
    oxi: &ToolPath,
    window: &Window,
    done_counter: &Arc<AtomicU64>,
    total_files: u64,
    should_cancel: &Arc<AtomicBool>,
) -> FileStats {
    let t_start = Instant::now();
    let _ = window.emit(
        "file_start",
        src.file_name().unwrap_or_default().to_string_lossy(),
    );

    if should_cancel.load(Ordering::Relaxed) {
        return FileStats::default();
    }

    if src != dest {
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if config.optimize_original {
            if fs::copy(src, dest).is_err() {
                return FileStats::default();
            }
        }
    }

    let original_size = fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    let mut webp_size = 0;
    let mut avif_size = 0;
    let mut duration_webp = 0.0;
    let mut duration_avif = 0.0;

    if config.webp || config.avif {
        if let Ok(img) = image::open(src) {
            if config.webp && !should_cancel.load(Ordering::Relaxed) {
                let t = Instant::now();
                webp_size = generate_webp(&img, dest, 75.0);
                duration_webp = t.elapsed().as_secs_f64();
            }

            if config.avif && !should_cancel.load(Ordering::Relaxed) {
                let t = Instant::now();
                avif_size = generate_avif(&img, dest);
                duration_avif = t.elapsed().as_secs_f64();
            }
        }
    }

    if should_cancel.load(Ordering::Relaxed) {
        return FileStats {
            bytes_saved: 0,
            original_size,
            optimized_size: original_size,
            webp_size,
            avif_size,
            duration_opt: 0.0,
            duration_webp,
            duration_avif,
        };
    }

    let ext = dest
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    let t_opt_start = Instant::now();

    let (new_size, bytes_saved) = if config.optimize_original {
        if src != dest && !dest.exists() {
             (0, 0)
        } else {
            let size = if ext == "png" {
                process_png(dest, pq, oxi, config.png_min, config.png_max)
            } else if ["jpg", "jpeg"].contains(&ext.as_str()) {
                process_jpg(dest, config.jpg_q)
            } else {
                original_size
            };

            let saved = if original_size > size {
                original_size - size
            } else {
                0
            };
            (size, saved)
        }
    } else {
        (0, 0)
    };

    let duration_opt_pure = t_opt_start.elapsed().as_secs_f64();

    let done = done_counter.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = window.emit(
        "progress",
        ProgressPayload {
            total: total_files,
            done,
            current_file: src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        },
    );

    let total_file_time = t_start.elapsed().as_secs_f64();
    let overhead = (total_file_time - duration_opt_pure - duration_webp - duration_avif).max(0.0);

    FileStats {
        bytes_saved,
        original_size,
        optimized_size: new_size,
        webp_size,
        avif_size,
        duration_opt: if config.optimize_original {
            duration_opt_pure + overhead
        } else {
            0.0
        },
        duration_webp,
        duration_avif,
    }
}
