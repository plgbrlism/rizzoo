use crate::error::LRatio;
use crate::paths::Paths;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub fn download(url: &str) -> Result<PathBuf, LRatio> {
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| LRatio::Custom(format!("failed to download image from URL: {e}")))?;

    let image_size = response
        .body_mut()
        .with_config()
        .limit(1024 * 1024 * 100)
        .read_to_vec()
        .map_err(|e| LRatio::Custom(format!("failed to read image data: {e}")))?;

    if image_size.is_empty() {
        return Err(LRatio::Custom("downloaded image is empty".into()));
    }

    let ext = guess_extension(&image_size);
    let hash = url_hash(url);
    let paths = Paths::resolve()?;
    let path = paths.downloads_dir.join(format!("{}{}", hash, ext));
    std::fs::write(&path, &image_size)
        .map_err(|e| LRatio::Custom(format!("failed to write image file: {e}")))?;
    Ok(path)
}

pub fn url_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

pub fn cached_url_path(url: &str, paths: &Paths) -> Option<PathBuf> {
    let hash = url_hash(url);
    for ext in [".png", ".jpg", ".jpeg", ".webp", ".gif", ".bmp"] {
        let p = paths.downloads_dir.join(format!("{}{}", hash, ext));
        if p.exists() {
            return Some(p);
        }
    }
    None
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
