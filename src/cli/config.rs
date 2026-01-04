use crate::agents::{
    AgentGenerator, ClaudeGenerator, CursorGenerator, ProjectContext, WindsurfGenerator,
};
use crate::config::GlobalConfig;
use crate::error::Result;
use crate::git::GitRepo;
use console::style;

pub struct ConfigCommand;

impl ConfigCommand {
    pub fn generate(agent: Option<String>, analyze: bool) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let context = if analyze {
            println!("{}", style("Analyzing project...").cyan());
            ProjectContext::detect(repo.root())?
        } else {
            ProjectContext {
                name: repo
                    .root()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                project_type: "unknown".to_string(),
                build_cmd: None,
                test_cmd: None,
                lint_cmd: None,
                is_workspace: false,
                workspace_members: Vec::new(),
            }
        };

        let agents_to_generate = match agent.as_deref() {
            Some("claude") => vec!["claude"],
            Some("cursor") => vec!["cursor"],
            Some("windsurf") => vec!["windsurf"],
            Some("all") | None => vec!["claude", "cursor", "windsurf"],
            Some(other) => {
                return Err(crate::error::KayfabeError::Other(format!(
                    "Unknown agent: {}. Valid options: claude, cursor, windsurf, all",
                    other
                )));
            }
        };

        for agent_name in agents_to_generate {
            let (generator, file_path): (Box<dyn AgentGenerator>, &str) = match agent_name {
                "claude" => (Box::new(ClaudeGenerator), ClaudeGenerator.file_path()),
                "cursor" => (Box::new(CursorGenerator), CursorGenerator.file_path()),
                "windsurf" => (Box::new(WindsurfGenerator), WindsurfGenerator.file_path()),
                _ => continue,
            };

            let content = generator.generate(&context)?;
            let target_path = repo.root().join(file_path);

            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&target_path, content)?;
            println!(
                "{} {}",
                style("✓ Generated:").green(),
                style(file_path).cyan()
            );
        }

        println!(
            "\n{}",
            style("Agent configurations generated successfully!")
                .bold()
                .green()
        );

        Ok(())
    }

    pub fn show(agent: Option<String>) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let files = match agent.as_deref() {
            Some("claude") => vec!["CLAUDE.md"],
            Some("cursor") => vec![".cursorrules"],
            Some("windsurf") => vec![".windsurf/rules/rules.md"],
            None => vec!["CLAUDE.md", ".cursorrules", ".windsurf/rules/rules.md"],
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
            ("CLAUDE.md", "Claude Code"),
            (".cursorrules", "Cursor"),
            (".windsurf/rules/rules.md", "Windsurf"),
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
            Some("claude") => "CLAUDE.md",
            Some("cursor") => ".cursorrules",
            Some("windsurf") => ".windsurf/rules/rules.md",
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
                "Configuration file not found: {}. Run 'kayfabe config generate' first.",
                file
            )));
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        std::process::Command::new(editor).arg(&path).status()?;

        Ok(())
    }
}
