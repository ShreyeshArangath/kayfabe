use crate::config::ProjectConfig;
use crate::error::Result;
use crate::git::GitRepo;
use console::style;
use std::path::PathBuf;

pub struct InitCommand;

impl InitCommand {
    pub fn execute(path: Option<PathBuf>) -> Result<()> {
        let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());

        println!(
            "{}",
            style("Initializing repository for AI-assisted development...").bold()
        );

        let repo = GitRepo::discover(&path)?;

        if repo.is_worktree_layout() {
            println!(
                "{}",
                style("✓ Repository already in worktree layout").green()
            );
        } else {
            println!("{}", style("Converting to worktree layout...").cyan());
            repo.convert_to_worktree_layout()?;
            println!(
                "{}",
                style("✓ Converted to worktree layout (main/ + wt/)").green()
            );
        }

        println!("{}", style("Creating project configuration...").cyan());
        let config = ProjectConfig::default();
        config.save(repo.layout_root())?;
        println!("{}", style("✓ Created .kayfabe/config.toml").green());

        println!(
            "\n{}",
            style("Repository ready for AI-assisted development!")
                .bold()
                .green()
        );
        println!(
            "Run {} to start a new task.",
            style("kayfabe worktree create <name>").cyan()
        );

        Ok(())
    }
}
