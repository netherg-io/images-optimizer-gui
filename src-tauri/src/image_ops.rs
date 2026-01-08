use crate::tools::{get_tool_ref, ToolPath};
use image::{DynamicImage, GenericImageView};
use moka::future::Cache;
use rgb::FromSlice;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct ImageCache(pub Cache<String, String>);

pub fn process_jpg(path: &Path, quality: u8) -> u64 {
    let img = match image::open(path) {
        Ok(i) => i.to_rgb8(),
        Err(_) => return fs::metadata(path).map(|m| m.len()).unwrap_or(0),
    };

    let (width, height) = img.dimensions();
    let pixels = img.as_raw();

    let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    comp.set_size(width as usize, height as usize);
    comp.set_quality(quality as f32);
    comp.set_progressive_mode();
    comp.set_optimize_scans(true);

    let mut comp = match comp.start_compress(Vec::new()) {
        Ok(c) => c,
        Err(_) => return fs::metadata(path).map(|m| m.len()).unwrap_or(0),
    };

    if comp.write_scanlines(pixels).is_err() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }

    let compressed_data = match comp.finish() {
        Ok(d) => d,
        Err(_) => return fs::metadata(path).map(|m| m.len()).unwrap_or(0),
    };

    if fs::write(path, &compressed_data).is_ok() {
        compressed_data.len() as u64
    } else {
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    }
}

pub fn process_png(path: &Path, pq: &ToolPath, oxi: &ToolPath, min: u8, max: u8) -> u64 {
    run_pngquant(path, pq, min, max);
    run_oxipng(path, oxi);
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn run_pngquant(path: &Path, tool: &ToolPath, min: u8, max: u8) {
    #[cfg(target_os = "windows")]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = Command::new(get_tool_ref(tool));
    cmd.args([
        &format!("--quality={}-{}", min, max),
        "--speed=3",
        "--force",
        "--ext=.png",
    ])
    .arg(path);

    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let _ = cmd.output();
}

fn run_oxipng(path: &Path, tool: &ToolPath) {
    #[cfg(target_os = "windows")]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = Command::new(get_tool_ref(tool));
    cmd.args(["-o", "4", "--strip", "all", "-t", "1"]).arg(path);

    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let _ = cmd.output();
}

pub fn generate_webp(img: &DynamicImage, path: &Path, quality: f32) -> u64 {
    let webp_path = path.with_extension("webp");
    let (width, height) = img.dimensions();

    let memory = match img {
        DynamicImage::ImageRgba8(buf) => {
            webp::Encoder::from_rgba(buf.as_raw(), width, height).encode(quality)
        }
        DynamicImage::ImageRgb8(buf) => {
            webp::Encoder::from_rgb(buf.as_raw(), width, height).encode(quality)
        }
        _ => {
            let buf = img.to_rgba8();
            webp::Encoder::from_rgba(buf.as_raw(), width, height).encode(quality)
        }
    };

    if fs::write(&webp_path, &*memory).is_ok() {
        memory.len() as u64
    } else {
        0
    }
}

pub fn generate_avif(img: &DynamicImage, path: &Path) -> u64 {
    let avif_path = path.with_extension("avif");
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    let src_img = imgref::Img::new(rgba.as_raw().as_rgba(), width as usize, height as usize);

    let enc = ravif::Encoder::new()
        .with_quality(65.0)
        .with_speed(4)
        .with_alpha_quality(70.0)
        .encode_rgba(src_img);

    match enc {
        Ok(encoded_image) => {
            if fs::write(&avif_path, &encoded_image.avif_file).is_ok() {
                encoded_image.avif_file.len() as u64
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("AVIF Error for {:?}: {}", path, e);
            0
        }
    }
}
