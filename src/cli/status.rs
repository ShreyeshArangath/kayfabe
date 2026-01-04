use crate::error::Result;
use crate::git::GitRepo;
use console::style;

pub struct StatusCommand;

impl StatusCommand {
    pub fn execute() -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        println!("{}", style("Repository Status").bold());
        println!();
        println!("  Root: {}", style(repo.root().display()).cyan());

        if repo.is_worktree_layout() {
            println!("  Layout: {}", style("worktree (main/ + wt/)").green());
        } else {
            println!("  Layout: {}", style("standard").yellow());
            println!();
            println!(
                "  Run {} to convert to worktree layout",
                style("kayfabe init").cyan()
            );
        }

        let worktrees = repo.list_worktrees()?;
        println!("  Worktrees: {}", style(worktrees.len()).cyan());

        Ok(())
    }
}
