use crate::error::{KayfabeError, Result};
use crate::git::{GitRepo, Worktree};
use crate::ide::{IDELauncher, IDE};
use console::style;
use dialoguer::Confirm;

pub struct WorktreeCommand;

impl WorktreeCommand {
    pub fn create(
        name: String,
        base: Option<String>,
        open: Option<String>,
        no_open: bool,
    ) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let base_branch = base.unwrap_or_else(|| "main".to_string());

        println!("{}", style(format!("Creating worktree: {}", name)).bold());

        if !repo.is_worktree_layout() {
            println!("{}", style("Converting to worktree layout first...").cyan());
            repo.convert_to_worktree_layout()?;
        }

        println!("{}", style("[1/3] Fetching latest refs...").cyan());
        let _ = repo.fetch();

        println!(
            "{}",
            style(format!(
                "[2/3] Creating worktree from base: {}",
                base_branch
            ))
            .cyan()
        );
        let wt_path = repo.create_worktree(&name, &base_branch)?;

        println!("{}", style("✓ Worktree created").green());
        println!("  Path: {}", style(wt_path.display()).cyan());
        println!("  Branch: {}", style(&name).cyan());

        if !no_open {
            if let Some(ide_name) = open {
                if let Some(ide) = IDE::parse(&ide_name) {
                    println!(
                        "{}",
                        style(format!("[3/3] Launching {}...", ide_name)).cyan()
                    );
                    IDELauncher::launch(ide, &wt_path)?;
                    println!("{}", style(format!("✓ {} launched", ide_name)).green());
                } else {
                    return Err(KayfabeError::IdeNotFound(ide_name));
                }
            }
        }

        Ok(())
    }

    pub fn list(stale: Option<u64>) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let worktrees = repo.list_worktrees()?;

        if worktrees.is_empty() {
            println!("{}", style("No worktrees found").yellow());
            return Ok(());
        }

        println!(
            "{}",
            style(format!("Worktrees in {}:", repo.root().display())).bold()
        );
        println!();

        let mut stale_worktrees = Vec::new();

        for wt_path in worktrees {
            let info = Worktree::get_info(&wt_path, "main")?;

            let name = wt_path.file_name().unwrap().to_string_lossy();
            let branch = info.branch.as_deref().unwrap_or("(detached)");

            if let Some(stale_days) = stale {
                if let Some(days) = info.staleness_days {
                    if days >= stale_days as f64 {
                        stale_worktrees.push((name.to_string(), days, info));
                    }
                }
            } else {
                let status = if info.is_main {
                    style("(anchor)".to_string()).dim()
                } else if let Some(days) = info.staleness_days {
                    if days > 14.0 {
                        style(format!("({:.0} days stale)", days)).yellow()
                    } else {
                        style(format!("({:.0} days)", days)).dim()
                    }
                } else {
                    style("(new)".to_string()).dim()
                };

                println!(
                    "  {} → {} {}",
                    style(name).cyan(),
                    style(branch).white(),
                    status
                );
            }
        }

        if stale.is_some() {
            if stale_worktrees.is_empty() {
                println!("{}", style("No stale worktrees found").green());
            } else {
                println!("{}", style("Stale worktrees:").bold());
                println!();
                for (name, days, info) in stale_worktrees {
                    let safety_status = if info.safety.is_safe_to_remove {
                        style("✓ safe to remove").green()
                    } else {
                        style("✗ has changes").red()
                    };
                    println!(
                        "  {} ({:.0} days) {}",
                        style(name).cyan(),
                        days,
                        safety_status
                    );
                }
            }
        }

        Ok(())
    }

    pub fn remove(name: String, force: bool) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let wt_path = if repo.is_worktree_layout() {
            repo.root().join("wt").join(&name)
        } else {
            repo.root().join(&name)
        };

        if !wt_path.exists() {
            return Err(KayfabeError::WorktreeNotFound(name));
        }

        if !force {
            let info = Worktree::get_info(&wt_path, "main")?;
            if !info.safety.is_safe_to_remove {
                let reasons = info.safety.blocked_reasons();
                println!("{}", style("Cannot remove worktree:").red().bold());
                for reason in reasons {
                    println!("  • {}", style(reason).yellow());
                }
                println!("\nUse {} to force removal", style("--force").cyan());
                return Err(KayfabeError::Other(
                    "Worktree has uncommitted work".to_string(),
                ));
            }
        }

        if force {
            repo.remove_worktree_force(&wt_path)?;
        } else {
            repo.remove_worktree(&wt_path)?;
        }

        println!("{}", style(format!("✓ Removed worktree: {}", name)).green());

        Ok(())
    }

    pub fn cleanup(
        older_than: u64,
        dry_run: bool,
        force: bool,
        include_unmerged: bool,
    ) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let repo = GitRepo::discover(&current_dir)?;

        let worktrees = repo.list_worktrees()?;
        let mut to_remove = Vec::new();
        let mut skipped = Vec::new();

        for wt_path in worktrees {
            let info = Worktree::get_info(&wt_path, "main")?;

            if info.is_main {
                continue;
            }

            if let Some(days) = info.staleness_days {
                if days >= older_than as f64 {
                    if info.safety.is_safe_to_remove || include_unmerged {
                        to_remove.push((wt_path, info));
                    } else {
                        skipped.push((wt_path, info));
                    }
                }
            }
        }

        if to_remove.is_empty() {
            println!("{}", style("No stale worktrees to clean up").green());
            return Ok(());
        }

        if dry_run {
            println!(
                "{}",
                style("DRY RUN - No changes will be made").bold().yellow()
            );
            println!();
        }

        println!("{}", style("Would remove:").bold());
        for (path, info) in &to_remove {
            let name = path.file_name().unwrap().to_string_lossy();
            let days = info.staleness_days.unwrap_or(0.0);
            let status = if info.safety.is_safe_to_remove {
                "merged"
            } else {
                "unmerged"
            };
            println!(
                "  {} ({:.0} days stale, {})",
                style(name).cyan(),
                days,
                status
            );
        }

        if !skipped.is_empty() {
            println!();
            println!("{}", style("Skipping (unmerged work):").bold());
            for (path, info) in &skipped {
                let name = path.file_name().unwrap().to_string_lossy();
                let days = info.staleness_days.unwrap_or(0.0);
                println!("  {} ({:.0} days stale)", style(name).yellow(), days);
            }
        }

        if dry_run {
            println!();
            println!("To execute: {}", style("kayfabe worktree cleanup").cyan());
            return Ok(());
        }

        if !force {
            println!();
            if !Confirm::new()
                .with_prompt("Proceed with cleanup?")
                .default(false)
                .interact()?
            {
                return Err(KayfabeError::Cancelled);
            }
        }

        println!();
        for (path, info) in to_remove {
            let name = path.file_name().unwrap().to_string_lossy();
            if info.safety.is_safe_to_remove {
                repo.remove_worktree(&path)?;
            } else {
                repo.remove_worktree_force(&path)?;
            }
            println!("{}", style(format!("✓ Removed {}", name)).green());
        }

        println!();
        println!("{}", style("Cleanup complete").bold().green());

        Ok(())
    }
}
