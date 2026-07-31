use sha2::{Digest, Sha256};
use std::path::Path;

use crate::color::types::ColorScheme;
use crate::error::LRatio;
use crate::paths::Paths;

pub fn cache_key(image_path: &Path, harmony: &str, is_light: bool, file_size: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(harmony.as_bytes());
    hasher.update([if is_light { 1u8 } else { 0u8 }]);
    hasher.update(file_size.to_le_bytes());
    if let Ok(mut file) = std::fs::File::open(image_path) {
        use std::io::Read;
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_ok() {
            hasher.update(&buffer);
        }
    }
    let hash = hasher.finalize();
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

pub fn load_scheme(paths: &Paths, key: &str) -> Result<Option<ColorScheme>, LRatio> {
    let path = paths.cache_colorscheme(key);
    if !path.exists() {
        return Ok(None);
    }
    match read_cache(&path) {
        Ok(dict) => Ok(Some(dict)),
        Err(_e) => {
            log::warn!("{}", LRatio::CacheCorrupted(path.clone()));
            let _ = std::fs::remove_file(&path);
            Ok(None)
        }
    }
}

pub fn save_scheme(paths: &Paths, key: &str, dict: &ColorScheme) -> Result<(), LRatio> {
    let path = paths.cache_colorscheme(key);
    let json = serde_json::to_string_pretty(dict)
        .map_err(|e| LRatio::CacheWrite(path.clone(), e.to_string()))?;
    std::fs::write(&path, json).map_err(|e| LRatio::CacheWrite(path.clone(), e.to_string()))?;
    Ok(())
}

fn read_cache(path: &Path) -> Result<ColorScheme, LRatio> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| LRatio::CacheRead(path.to_path_buf(), e.to_string()))?;
    serde_json::from_str(&contents).map_err(|_| LRatio::CacheCorrupted(path.to_path_buf()))
}

pub fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}
