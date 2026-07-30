use crate::error::LRatio;
use std::path::Path;
use std::path::PathBuf;

pub fn download(url: &str) -> Result<PathBuf, LRatio> {
    let body = ureq::get(url)
        .call()
        .map_err(|e| LRatio::Custom(format!("failed to download image from URL: {e}")))?
        .into_body()
        .read_to_vec()
        .map_err(|e| LRatio::Custom(format!("failed to read image data: {e}")))?;

    if body.is_empty() {
        return Err(LRatio::Custom("downloaded image is empty".into()));
    }

    let ext = guess_extension(&body);
    let pid = std::process::id();
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!("rizzoora-url-{}-{}", pid, ext));

    std::fs::write(&tmp_path, &body)
        .map_err(|e| LRatio::Custom(format!("failed to write temp image: {e}")))?;

    Ok(tmp_path)
}

/// Returns `true` if `path` was created by a previous call to `download()`.
pub fn is_downloaded_temp_file(path: &Path) -> bool {
    path.starts_with(std::env::temp_dir()) && {
        path.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("rizzoora-url-"))
    }
}

fn guess_extension(bytes: &[u8]) -> &'static str {
    if bytes.len() < 4 {
        return ".png";
    }
    match &bytes[..4] {
        [0x89, 0x50, 0x4E, 0x47] => ".png",
        [0xFF, 0xD8, 0xFF, _] => ".jpg",
        [0x52, 0x49, 0x46, 0x46] => ".webp",
        [0x47, 0x49, 0x46, _] => ".gif",
        [0x42, 0x4D, _, _] => ".bmp",
        _ => ".png",
    }
}
