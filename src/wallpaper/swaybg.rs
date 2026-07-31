use crate::error::LRatio;
use std::path::Path;
use std::process::Command;

pub fn set(path: &Path) -> Result<(), LRatio> {
    let _ = Command::new("pkill").arg("swaybg").status();

    Command::new("swaybg")
        .arg("-m")
        .arg("fill")
        .arg("-i")
        .arg(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| LRatio::WallpaperSet(format!("swaybg not found or failed to launch: {e}")))?;

    Ok(())
}
