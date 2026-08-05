use crate::error::LRatio;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Paths {
    pub cache_dir: PathBuf,
    pub output: PathBuf,
    pub schemes_dir: PathBuf,
    pub config_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub config_toml: PathBuf,
    pub downloads_dir: PathBuf,
}

impl Paths {
    pub fn resolve() -> Result<Self, LRatio> {
        let proj = ProjectDirs::from("com", "rizzoo", "rizzoo").ok_or(LRatio::HomeDirNotFound)?;

        let cache_dir = proj.cache_dir().to_path_buf();
        let config_dir = proj.config_dir().to_path_buf();

        Ok(Self {
            output: cache_dir.join("colors.json"),
            schemes_dir: cache_dir.join("schemes"),
            downloads_dir: cache_dir.join("url-images"),
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
            &self.downloads_dir,
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

    pub fn check_config(&self, force: bool) -> Result<(), LRatio> {
        self.check_directory()?;
        if self.config_toml.exists() && !force {
            log::warn!("config already exists at {}", self.config_toml.display());
            log::warn!("use --init-overwrite to reset to defaults");
            return Ok(());
        }
        let default = include_str!("../assets/config.toml");
        std::fs::write(&self.config_toml, default).map_err(|e| {
            LRatio::DirectoryCreationFailed(self.config_toml.clone(), e.to_string())
        })?;
        Ok(())
    }

    pub fn cache_colorscheme(&self, hash: &str) -> PathBuf {
        self.schemes_dir.join(format!("{hash}.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths_in(tmp: &std::path::Path) -> Paths {
        Paths {
            cache_dir: tmp.join("cache"),
            output: tmp.join("cache/colors.json"),
            schemes_dir: tmp.join("cache/schemes"),
            config_dir: tmp.join("config"),
            templates_dir: tmp.join("config/templates"),
            config_toml: tmp.join("config/config.toml"),
            downloads_dir: tmp.join("cache/url-images"),
        }
    }

    #[test]
    fn check_directory_creates_all_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths_in(tmp.path());
        p.check_directory().unwrap();
        for d in [&p.cache_dir, &p.schemes_dir, &p.downloads_dir, &p.config_dir, &p.templates_dir] {
            assert!(d.exists(), "{d:?} missing");
        }
    }

    #[test]
    fn check_config_creates_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths_in(tmp.path());
        p.check_config(false).unwrap();
        let content = std::fs::read_to_string(&p.config_toml).unwrap();
        assert!(content.contains("[alacritty]"));
        assert!(content.contains("style = \"tonal-spot\""));
    }

    #[test]
    fn check_config_no_overwrite_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths_in(tmp.path());
        p.check_directory().unwrap();
        std::fs::write(&p.config_toml, "my custom config").unwrap();
        p.check_config(false).unwrap();
        let content = std::fs::read_to_string(&p.config_toml).unwrap();
        assert_eq!(content, "my custom config");
    }

    #[test]
    fn check_config_overwrites_with_force() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths_in(tmp.path());
        p.check_directory().unwrap();
        std::fs::write(&p.config_toml, "my custom config").unwrap();
        p.check_config(true).unwrap();
        let content = std::fs::read_to_string(&p.config_toml).unwrap();
        assert_eq!(content, include_str!("../assets/config.toml"));
    }
}
