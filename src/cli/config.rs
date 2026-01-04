use crate::config::GlobalConfig;
use crate::error::Result;
use crate::git::GitRepo;
use console::style;

pub struct ConfigCommand;

impl ConfigCommand {
    pub fn show(agent: Option<String>) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let files = match agent.as_deref() {
            Some("windsurf") => vec![".windsurfrules"],
            None => vec![".windsurfrules"],
            Some(other) => {
                return Err(crate::error::KayfabeError::Other(format!(
                    "Unknown agent: {}",
                    other
                )));
            }
        };

        for file in files {
            let path = repo.root().join(file);
            if path.exists() {
                println!("\n{}", style(format!("=== {} ===", file)).bold().cyan());
                let content = std::fs::read_to_string(&path)?;
                println!("{}", content);
            } else {
                println!("{} {} (not found)", style("⚠").yellow(), style(file).dim());
            }
        }

        Ok(())
    }

    pub fn validate() -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let files = vec![
            (".windsurfrules", "Windsurf"),
        ];

        println!("{}", style("Validating agent configurations...").bold());
        println!();

        let mut all_valid = true;

        for (file, agent_name) in files {
            let path = repo.root().join(file);
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                if content.trim().is_empty() {
                    println!(
                        "{} {} - empty file",
                        style("✗").red(),
                        style(agent_name).cyan()
                    );
                    all_valid = false;
                } else {
                    println!(
                        "{} {} - valid",
                        style("✓").green(),
                        style(agent_name).cyan()
                    );
                }
            } else {
                println!(
                    "{} {} - not found",
                    style("⚠").yellow(),
                    style(agent_name).dim()
                );
            }
        }

        println!();
        if all_valid {
            println!("{}", style("All configurations are valid!").green().bold());
        } else {
            println!(
                "{}",
                style("Some configurations need attention").yellow().bold()
            );
        }

        Ok(())
    }

    pub fn init() -> Result<()> {
        println!("{}", style("Initializing global configuration...").cyan());

        GlobalConfig::init()?;
        let config_path = GlobalConfig::path()?;

        println!(
            "{} Created: {}",
            style("✓").green(),
            style(config_path.display()).cyan()
        );

        Ok(())
    }

    pub fn edit(agent: Option<String>) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let file = match agent.as_deref() {
            Some("windsurf") => ".windsurfrules",
            None => {
                let config_path = GlobalConfig::path()?;
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
                std::process::Command::new(editor)
                    .arg(&config_path)
                    .status()?;
                return Ok(());
            }
            Some(other) => {
                return Err(crate::error::KayfabeError::Other(format!(
                    "Unknown agent: {}",
                    other
                )));
            }
        };

        let path = repo.root().join(file);
        if !path.exists() {
            return Err(crate::error::KayfabeError::Other(format!(
                "Configuration file not found: {}.",
                file
            )));
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        std::process::Command::new(editor).arg(&path).status()?;

        Ok(())
    }
}
