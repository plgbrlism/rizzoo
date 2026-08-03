use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LRatio {
    // CLI
    #[error("Invalid args: {0}")]
    Cli(String),
    #[error("Image not found: {0}")]
    ImageNotFound(PathBuf),
    #[error("No images in: {0}")]
    EmptyDir(PathBuf),
    #[error("Decode: {0}")]
    ImageDecode(String),
    #[error("No colors")]
    NoColors,
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Home dir not found")]
    HomeDirNotFound,
    #[error("Create dir {0}: {1}")]
    DirectoryCreationFailed(PathBuf, String),
    #[error("No compositor detected")]
    NoCompositorDetected,

    // Template
    #[error("Parse: {0}")]
    Parse(String),
    #[error("Render: {0}")]
    Render(String),
    #[error("Unknown var: {0}")]
    UndefinedVar(String),
    #[error("Unknown array: {0}")]
    UndefinedArray(String),
    #[error("Filter '{0}' not found")]
    FilterNotFound(String),
    #[error("Bad filter arg: {0}")]
    InvalidFilterArg(String),
    #[error("Template read: {0} - {1}")]
    TemplateRead(PathBuf, String),
    #[error("Template write: {0} - {1}")]
    TemplateWrite(PathBuf, String),

    // Color
    #[error("Bad color: {0}")]
    InvalidColor(String),
    #[error("Scheme: {0}")]
    Scheme(String),

    // Caching
    #[error("Cache read: {0} - {1}")]
    CacheRead(PathBuf, String),
    #[error("Cache write: {0} - {1}")]
    CacheWrite(PathBuf, String),
    #[error("Cache corrupted: {0}")]
    CacheCorrupted(PathBuf),

    // Wallpaper
    #[error("Wallpaper set: {0}")]
    WallpaperSet(String),
    #[error("Wallpaper hook: {0}")]
    WallpaperHook(String),

    // Output
    #[error("Colors.json write: {0} - {1}")]
    ColorsJsonWrite(PathBuf, String),
    #[error("Output write {0} -> {1}: {2}")]
    OutputWriteFailed(PathBuf, PathBuf, String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, LRatio>;
