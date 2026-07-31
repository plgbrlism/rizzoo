use crate::error::LRatio;
use std::path::Path;
use std::process::Command;

pub fn set(path: &Path) -> Result<(), LRatio> {
    let status = Command::new("feh")
        .args(["--bg-scale", "--no-fehbg"])
        .arg(path)
        .status()
        .map_err(|e| LRatio::WallpaperSet(format!("feh not found or failed to launch: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(LRatio::WallpaperSet(format!(
            "feh exited with code {}",
            status.code().unwrap_or(-1)
        )))
    }
}
