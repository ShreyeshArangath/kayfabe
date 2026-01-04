use crate::agents::AgentInstaller;
use crate::config::GlobalConfig;
use crate::error::Result;
use console::style;
use dialoguer::MultiSelect;
use std::path::PathBuf;

pub struct InstallCommand;

impl InstallCommand {
    pub fn execute(
        target_dir: Option<PathBuf>,
        non_interactive: bool,
        agents: Option<Vec<String>>,
    ) -> Result<()> {
        let target = target_dir.unwrap_or_else(|| std::env::current_dir().unwrap());

        let available_agents = vec!["windsurf"];

        let selected_agents = if non_interactive {
            // In non-interactive mode, use provided agents or default to all
            if let Some(agent_list) = agents {
                agent_list
            } else {
                available_agents.iter().map(|s| s.to_string()).collect()
            }
        } else {
            // Interactive mode
            let defaults = vec![true];
            let selections = MultiSelect::new()
                .with_prompt("Select agents to install globally")
                .items(&available_agents)
                .defaults(&defaults)
                .interact()?;

            selections
                .iter()
                .map(|&idx| available_agents[idx].to_string())
                .collect()
        };

        // Install selected agents globally
        for agent in &selected_agents {
            AgentInstaller::install_global(agent, &target)?;
        }

        // Update global config - mark agents as enabled
        let mut config = GlobalConfig::load()?;
        for agent in &selected_agents {
            if let Some(agent_config) = config.agents.get_mut(agent) {
                agent_config.enabled = true;
            }
        }
        GlobalConfig::save(&config)?;

        println!(
            "{}",
            style("✓ Global agent installation complete!").green().bold()
        );
        Ok(())
    }
}
