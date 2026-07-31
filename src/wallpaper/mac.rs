use crate::error::LRatio;
use std::path::Path;
use std::process::Command;

pub fn set(path: &Path) -> Result<String, LRatio> {
    let escaped = path.display().to_string().replace('"', "\\\"");
    let script = format!(
        "tell application \"Finder\" to set desktop picture to POSIX file \"{}\"",
        escaped
    );
    let status = Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map_err(|e| LRatio::WallpaperSet(format!("osascript failed: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(LRatio::WallpaperSet(format!(
            "osascript exited with code {}",
            status.code().unwrap_or(-1)
        )))
    }
}
