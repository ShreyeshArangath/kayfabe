use crate::error::Result;
use std::path::Path;

pub struct AgentInstaller;

impl AgentInstaller {
    pub fn install_global(agent: &str, target_dir: &Path) -> Result<()> {
        match agent {
            "windsurf" => Self::install_windsurf_global(target_dir),
            _ => Err(crate::error::KayfabeError::Other(format!(
                "Unknown agent: {}",
                agent
            ))),
        }
    }

    fn install_windsurf_global(target_dir: &Path) -> Result<()> {
        let local_path = target_dir.join(".windsurfrules");
        let content = Self::generate_windsurf_config()?;
        std::fs::write(&local_path, &content)?;

        println!("✓ Windsurf agent installed locally");
        Ok(())
    }

    fn generate_windsurf_config() -> Result<String> {
        Ok(crate::agents::templates::WINDSURF_TEMPLATE.to_string())
    }
}
