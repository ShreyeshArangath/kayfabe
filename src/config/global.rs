use crate::config::Config;
use crate::error::Result;
use std::path::PathBuf;

pub struct GlobalConfig;

impl GlobalConfig {
    pub fn path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            crate::error::KayfabeError::Other("Could not find config directory".to_string())
        })?;
        Ok(config_dir.join("kayfabe").join("config.toml"))
    }

    pub fn load() -> Result<Config> {
        let path = Self::path()?;

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            crate::error::KayfabeError::Other(format!("Failed to parse config: {}", e))
        })?;

        Ok(config)
    }

    pub fn save(config: &Config) -> Result<()> {
        let path = Self::path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(config).map_err(|e| {
            crate::error::KayfabeError::Other(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(&path, content)?;

        Ok(())
    }

    pub fn init() -> Result<()> {
        let config = Config::default();
        Self::save(&config)?;
        Ok(())
    }
}
