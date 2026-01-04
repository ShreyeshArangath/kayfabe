use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: ProjectInfo,
    #[serde(default)]
    pub worktree: ProjectWorktreeConfig,
    #[serde(default)]
    pub agents: ProjectAgentsConfig,
    #[serde(default)]
    pub hooks: ProjectHooks,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectInfo {
    pub name: Option<String>,
    pub project_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectWorktreeConfig {
    pub base_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectAgentsConfig {
    pub preferred: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectHooks {
    #[serde(default)]
    pub post_create: Vec<String>,
    #[serde(default)]
    pub pre_open: Vec<String>,
}

impl ProjectConfig {
    pub fn path(repo_root: &Path) -> PathBuf {
        repo_root.join(".kayfabe").join("config.toml")
    }

    pub fn load(repo_root: &Path) -> Result<Self> {
        let path = Self::path(repo_root);

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: ProjectConfig = toml::from_str(&content).map_err(|e| {
            crate::error::KayfabeError::Other(format!("Failed to parse project config: {}", e))
        })?;

        Ok(config)
    }

    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let path = Self::path(repo_root);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::error::KayfabeError::Other(format!("Failed to serialize project config: {}", e))
        })?;

        std::fs::write(&path, content)?;

        Ok(())
    }
}
