use crate::color::types::ColorScheme;
use crate::error::LRatio;
use crate::paths::Paths;

pub fn write(paths: &Paths, scheme: &ColorScheme) -> Result<(), LRatio> {
    let json = serde_json::to_string_pretty(scheme)
        .map_err(|e| LRatio::ColorsJsonWrite(paths.output.clone(), e.to_string()))?;
    std::fs::write(&paths.output, json)
        .map_err(|e| LRatio::ColorsJsonWrite(paths.output.clone(), e.to_string()))?;
    Ok(())
}

pub fn read(paths: &Paths) -> Result<ColorScheme, LRatio> {
    let contents = std::fs::read_to_string(&paths.output)
        .map_err(|e| LRatio::CacheRead(paths.output.clone(), e.to_string()))?;
    serde_json::from_str(&contents).map_err(|_| LRatio::CacheCorrupted(paths.output.clone()))
}
