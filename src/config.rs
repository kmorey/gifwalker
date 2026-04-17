use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub giphy_api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<AppConfig> {
        if !self.path.exists() {
            return Ok(AppConfig::default());
        }

        let raw = fs::read_to_string(&self.path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = toml::to_string_pretty(config)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }
}

pub fn default_config_path() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("", "", "gifwalker") {
        return dirs.config_dir().join("config.toml");
    }

    PathBuf::from("config.toml")
}
