use crate::tools::{get_tool_ref, ToolPath};
use crate::types::{OperationError, OptimizationErrorCode};
use image::{DynamicImage, GenericImageView};
use moka::future::Cache;
use rgb::FromSlice;
use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const PROCESS_TIMEOUT: Duration = Duration::from_secs(120);

pub struct ImageCache(pub Cache<String, String>);

pub fn oriented_dimensions(path: &Path) -> Result<(u32, u32), image::ImageError> {
    let (width, height) = image::image_dimensions(path)?;
    Ok(if matches!(read_orientation(path), 5..=8) {
        (height, width)
    } else {
        (width, height)
    })
}

fn open_oriented(path: &Path) -> Result<DynamicImage, image::ImageError> {
    let image = image::open(path)?;
    Ok(apply_orientation(image, read_orientation(path)))
}

fn read_orientation(path: &Path) -> u32 {
    let Ok(file) = fs::File::open(path) else {
        return 1;
    };
    exif::Reader::new()
        .read_from_container(&mut BufReader::new(file))
        .ok()
        .and_then(|exif| {
            exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
                .and_then(|field| field.value.get_uint(0))
        })
        .unwrap_or(1)
}

fn apply_orientation(image: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => image.fliph(),
        3 => image.rotate180(),
        4 => image.flipv(),
        5 => image.rotate90().fliph(),
        6 => image.rotate90(),
        7 => image.rotate270().fliph(),
        8 => image.rotate270(),
        _ => image,
    }
}

pub fn optimize_jpeg(source: &Path, output: &Path, quality: u8) -> Result<(), OperationError> {
    let img = open_oriented(source)
        .map_err(|error| {
            OperationError::new(OptimizationErrorCode::DecodeFailed, error.to_string())
        })?
        .to_rgb8();
    let (width, height) = img.dimensions();

    let mut compressor = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    compressor.set_size(width as usize, height as usize);
    compressor.set_quality(quality as f32);
    compressor.set_progressive_mode();
    compressor.set_optimize_scans(true);

    let mut compressor = compressor.start_compress(Vec::new()).map_err(|error| {
        OperationError::new(OptimizationErrorCode::EncodeFailed, error.to_string())
    })?;
    compressor.write_scanlines(img.as_raw()).map_err(|error| {
        OperationError::new(OptimizationErrorCode::EncodeFailed, error.to_string())
    })?;
    let compressed = compressor.finish().map_err(|error| {
        OperationError::new(OptimizationErrorCode::EncodeFailed, error.to_string())
    })?;
    fs::write(output, compressed)
        .map_err(|error| OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed))
}

pub fn optimize_png(
    source: &Path,
    output: &Path,
    pngquant: &ToolPath,
    oxipng: &ToolPath,
    minimum_quality: u8,
    maximum_quality: u8,
    should_cancel: &AtomicBool,
) -> Result<(), OperationError> {
    fs::copy(source, output).map_err(|error| {
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;
    run_pngquant(
        output,
        pngquant,
        minimum_quality,
        maximum_quality,
        should_cancel,
    )?;
    run_oxipng(output, oxipng, should_cancel)
}

fn run_pngquant(
    path: &Path,
    tool: &ToolPath,
    minimum_quality: u8,
    maximum_quality: u8,
    should_cancel: &AtomicBool,
) -> Result<(), OperationError> {
    let mut command = Command::new(get_tool_ref(tool));
    command
        .arg(format!("--quality={minimum_quality}-{maximum_quality}"))
        .args(["--speed", "3", "--force", "--ext", ".png"])
        .arg(path);
    hide_console(&mut command);
    run_command(command, tool, should_cancel)
}

fn run_oxipng(
    path: &Path,
    tool: &ToolPath,
    should_cancel: &AtomicBool,
) -> Result<(), OperationError> {
    let mut command = Command::new(get_tool_ref(tool));
    command
        .args(["-o", "4", "--strip", "all", "-t", "1"])
        .arg(path);
    hide_console(&mut command);
    run_command(command, tool, should_cancel)
}

#[cfg(target_os = "windows")]
fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x08000000);
}

#[cfg(not(target_os = "windows"))]
fn hide_console(_: &mut Command) {}

fn run_command(
    mut command: Command,
    tool: &ToolPath,
    should_cancel: &AtomicBool,
) -> Result<(), OperationError> {
    if should_cancel.load(Ordering::Relaxed) {
        return Err(OperationError::new(
            OptimizationErrorCode::Canceled,
            "Optimization was canceled",
        ));
    }

    let stdout = tempfile::tempfile().map_err(|error| {
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;
    let stderr = tempfile::tempfile().map_err(|error| {
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;
    command
        .stdout(Stdio::from(stdout.try_clone().map_err(|error| {
            OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
        })?))
        .stderr(Stdio::from(stderr.try_clone().map_err(|error| {
            OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
        })?));

    let tool_name = get_tool_ref(tool)
        .to_string_lossy()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("codec")
        .to_string();
    let mut child = command.spawn().map_err(|error| {
        let code = match error.kind() {
            std::io::ErrorKind::NotFound => OptimizationErrorCode::ToolMissing,
            std::io::ErrorKind::PermissionDenied => OptimizationErrorCode::PermissionDenied,
            _ => OptimizationErrorCode::ToolFailed,
        };
        OperationError::new(code, format!("Failed to start {tool_name}: {error}"))
    })?;
    let started = Instant::now();

    let status = loop {
        if should_cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(OperationError::new(
                OptimizationErrorCode::Canceled,
                format!("{tool_name} was canceled"),
            ));
        }
        if started.elapsed() >= PROCESS_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(OperationError::new(
                OptimizationErrorCode::ProcessTimeout,
                format!("{tool_name} timed out after 120 seconds"),
            ));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                let _ = child.kill();
                return Err(OperationError::new(
                    OptimizationErrorCode::ToolFailed,
                    format!("Failed while waiting for {tool_name}: {error}"),
                ));
            }
        }
    };

    if status.success() {
        return Ok(());
    }

    let mut stderr_text = String::new();
    let _ = (&stderr).read_to_string(&mut stderr_text);
    let mut stdout_text = String::new();
    let _ = (&stdout).read_to_string(&mut stdout_text);
    let detail = if stderr_text.trim().is_empty() {
        stdout_text.trim()
    } else {
        stderr_text.trim()
    };
    Err(OperationError::new(
        OptimizationErrorCode::ToolFailed,
        format!(
            "{tool_name} exited with code {}{}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            if detail.is_empty() {
                String::new()
            } else {
                format!(": {detail}")
            }
        ),
    ))
}

pub fn encode_webp(source: &Path, output: &Path, quality: f32) -> Result<(), OperationError> {
    let image = open_oriented(source).map_err(|error| {
        OperationError::new(OptimizationErrorCode::DecodeFailed, error.to_string())
    })?;
    let (width, height) = image.dimensions();
    let memory = match image {
        DynamicImage::ImageRgba8(buffer) => {
            webp::Encoder::from_rgba(buffer.as_raw(), width, height).encode(quality)
        }
        DynamicImage::ImageRgb8(buffer) => {
            webp::Encoder::from_rgb(buffer.as_raw(), width, height).encode(quality)
        }
        image => {
            let buffer = image.to_rgba8();
            webp::Encoder::from_rgba(buffer.as_raw(), width, height).encode(quality)
        }
    };
    fs::write(output, &*memory)
        .map_err(|error| OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed))
}

pub fn encode_avif(
    source: &Path,
    output: &Path,
    quality: f32,
    speed: u8,
) -> Result<(), OperationError> {
    let image = open_oriented(source).map_err(|error| {
        OperationError::new(OptimizationErrorCode::DecodeFailed, error.to_string())
    })?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    let source_image = imgref::Img::new(rgba.as_raw().as_rgba(), width as usize, height as usize);
    let encoded = ravif::Encoder::new()
        .with_quality(quality)
        .with_speed(speed)
        .with_alpha_quality(quality)
        .encode_rgba(source_image)
        .map_err(|error| {
            OperationError::new(OptimizationErrorCode::EncodeFailed, error.to_string())
        })?;
    fs::write(output, encoded.avif_file)
        .map_err(|error| OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exif_rotation_swaps_dimensions() {
        let image = DynamicImage::new_rgb8(2, 3);
        assert_eq!(apply_orientation(image, 6).dimensions(), (3, 2));
    }
}
