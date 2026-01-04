use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub worktree: WorktreeConfig,
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    #[serde(default)]
    pub ui: UIConfig,
}

impl Default for Config {
    fn default() -> Self {
        let mut agents = HashMap::new();
        agents.insert(
            "windsurf".to_string(),
            AgentConfig {
                enabled: true,
                template: "default".to_string(),
                location: ".windsurfrules".to_string(),
            },
        );

        Self {
            defaults: DefaultsConfig::default(),
            worktree: WorktreeConfig::default(),
            agents,
            ui: UIConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_ide")]
    pub ide: String,
    #[serde(default = "default_base_branch")]
    pub base_branch: String,
    #[serde(default = "default_true")]
    pub auto_fetch: bool,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            ide: default_ide(),
            base_branch: default_base_branch(),
            auto_fetch: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_naming")]
    pub naming: String,
    #[serde(default = "default_stale_days")]
    pub stale_days: u64,
    #[serde(default)]
    pub auto_cleanup: bool,
    #[serde(default = "default_true")]
    pub protect_unmerged: bool,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            layout: default_layout(),
            naming: default_naming(),
            stale_days: default_stale_days(),
            auto_cleanup: false,
            protect_unmerged: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_template")]
    pub template: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default = "default_true")]
    pub progress: bool,
    #[serde(default = "default_true")]
    pub interactive: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            color: true,
            progress: true,
            interactive: true,
        }
    }
}

fn default_ide() -> String {
    "windsurf".to_string()
}

fn default_base_branch() -> String {
    "main".to_string()
}

fn default_layout() -> String {
    "standard".to_string()
}

fn default_naming() -> String {
    "branch".to_string()
}

fn default_stale_days() -> u64 {
    14
}

fn default_template() -> String {
    "default".to_string()
}

fn default_true() -> bool {
    true
}
