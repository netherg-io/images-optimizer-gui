use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(target_os = "windows")]
const PNGQUANT_BIN: &[u8] = include_bytes!("../bin/pngquant.exe");
#[cfg(target_os = "windows")]
const OXIPNG_BIN: &[u8] = include_bytes!("../bin/oxipng.exe");

pub enum ToolPath {
    Path(PathBuf),
    #[allow(dead_code)]
    Command(String),
}

pub fn get_tool_ref(t: &ToolPath) -> &OsStr {
    match t {
        ToolPath::Path(p) => p.as_os_str(),
        ToolPath::Command(c) => OsStr::new(c),
    }
}

pub fn get_png_tools() -> Result<(Option<TempDir>, ToolPath, ToolPath), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        let dir = tempfile::tempdir()?;
        let pngquant_path = dir.path().join("pngquant.exe");
        let oxi_path = dir.path().join("oxipng.exe");
        let mut file = fs::File::create(&pngquant_path)?;
        file.write_all(PNGQUANT_BIN)?;
        let mut file = fs::File::create(&oxi_path)?;
        file.write_all(OXIPNG_BIN)?;
        Ok((
            Some(dir),
            ToolPath::Path(pngquant_path),
            ToolPath::Path(oxi_path),
        ))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok((
            None,
            ToolPath::Command("pngquant".to_string()),
            ToolPath::Command("oxipng".to_string()),
        ))
    }
}
