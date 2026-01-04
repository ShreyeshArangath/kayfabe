use crate::error::Result;
use std::path::Path;

pub struct AgentInstaller;

impl AgentInstaller {
    pub fn install_global(agent: &str, target_dir: &Path) -> Result<()> {
        match agent {
            "claude" => Self::install_claude_global(target_dir),
            "cursor" => Self::install_cursor_global(target_dir),
            "windsurf" => Self::install_windsurf_global(target_dir),
            _ => Err(crate::error::KayfabeError::Other(format!(
                "Unknown agent: {}",
                agent
            ))),
        }
    }

    fn install_claude_global(target_dir: &Path) -> Result<()> {
        let global_path = dirs::home_dir()
            .ok_or_else(|| {
                crate::error::KayfabeError::Other("Could not find home directory".to_string())
            })?
            .join(".claude")
            .join("CLAUDE.md");

        let local_path = target_dir.join("CLAUDE.md");

        // Create global config directory
        if let Some(parent) = global_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Generate universal Claude config
        let content = Self::generate_universal_config()?;

        // Install globally and locally
        std::fs::write(&global_path, &content)?;
        std::fs::write(&local_path, &content)?;

        println!("✓ Claude agent installed globally and locally");
        Ok(())
    }

    fn install_cursor_global(target_dir: &Path) -> Result<()> {
        let local_path = target_dir.join(".cursorrules");
        let content = Self::generate_universal_config()?;
        std::fs::write(&local_path, &content)?;
        println!("✓ Cursor agent installed locally");
        Ok(())
    }

    fn install_windsurf_global(target_dir: &Path) -> Result<()> {
        let local_path = target_dir.join(".windsurfrules");
        let content = Self::generate_universal_config()?;
        std::fs::write(&local_path, &content)?;

        println!("✓ Windsurf agent installed locally");
        Ok(())
    }

    fn generate_universal_config() -> Result<String> {
        Ok(crate::agents::templates::UNIVERSAL_AGENT_TEMPLATE.to_string())
    }
}
