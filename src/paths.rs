use crate::error::LRatio;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Paths {
    pub cache_dir: PathBuf,
    pub output: PathBuf,
    pub sequences: PathBuf,
    pub schemes_dir: PathBuf,
    pub config_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub config_toml: PathBuf,
}

impl Paths {
    pub fn resolve() -> Result<Self, LRatio> {
        let proj = ProjectDirs::from("com", "rizzoo", "rizzoo").ok_or(LRatio::HomeDirNotFound)?;

        let cache_dir = proj.cache_dir().to_path_buf();
        let config_dir = proj.config_dir().to_path_buf();

        Ok(Self {
            output: cache_dir.join("colors.json"),
            sequences: cache_dir.join("sequences"),
            schemes_dir: cache_dir.join("schemes"),
            cache_dir,
            templates_dir: config_dir.join("templates"),
            config_toml: config_dir.join("config.toml"),
            config_dir,
        })
    }

    pub fn check_directory(&self) -> Result<(), LRatio> {
        for dir in [
            &self.cache_dir,
            &self.schemes_dir,
            &self.config_dir,
            &self.templates_dir,
        ] {
            if !dir.exists() {
                std::fs::create_dir_all(dir).map_err(|e| {
                    LRatio::DirectoryCreationFailed(dir.to_path_buf(), e.to_string())
                })?;
            }
        }
        Ok(())
    }

    pub fn check_config(&self) -> Result<(), LRatio> {
        self.check_directory()?;
        if !self.config_toml.exists() {
            let default = include_str!("../assets/config.toml");
            std::fs::write(&self.config_toml, default).map_err(|e| {
                LRatio::DirectoryCreationFailed(self.config_toml.clone(), e.to_string())
            })?;
        }
        Ok(())
    }

    pub fn cache_colorscheme(&self, hash: &str) -> PathBuf {
        self.schemes_dir.join(format!("{hash}.json"))
    }
}
