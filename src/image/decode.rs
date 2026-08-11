use crate::error::LRatio;
use rand::seq::IndexedRandom;
use std::path::{Path, PathBuf};

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "bmp", "gif", "tiff", "tif",
    "avif", // fixed: needs to enable avif-native in image crate
];

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|ext| e.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false)
}

pub fn resolve(path: &Path) -> Result<PathBuf, LRatio> {
    match path {
        p if !p.exists() => Err(LRatio::ImageNotFound(p.to_path_buf())),
        p if p.is_file() => Ok(p.to_path_buf()),
        p if p.is_dir() => pick_from_dir(p),
        _ => Err(LRatio::ImageNotFound(path.to_path_buf())),
    }
}

fn pick_from_dir(dir: &Path) -> Result<PathBuf, LRatio> {
    let images: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_image(p))
        .collect();

    let mut random = rand::rng(); // refactor: instead of latest modified file, choose randomly from a valid directory
    images
        .choose(&mut random)
        .cloned()
        .ok_or_else(|| LRatio::EmptyDir(dir.to_path_buf()))
}
