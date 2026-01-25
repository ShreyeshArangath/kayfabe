mod cli;
mod core;
mod db;
mod errors;
mod tui;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kayfabe")]
#[command(version, about = "AI-enhanced CLI task manager with git worktree orchestration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Kayfabe project
    Init {
        /// Project name (optional if using --interactive)
        name: Option<String>,
        /// Git repository URL (SSH or HTTPS)
        git_url: Option<String>,
        /// Interactive mode - wizard-style prompts
        #[arg(short, long)]
        interactive: bool,
    },

    /// Add a new task
    Add {
        /// Task name (optional if using --interactive)
        name: Option<String>,
        /// Task description
        #[arg(short, long)]
        description: Option<String>,
        /// Priority (high/medium/low)
        #[arg(short, long)]
        priority: Option<String>,
        /// Auto-assign to available agent
        #[arg(long)]
        auto_assign: bool,
        /// Interactive mode - wizard-style prompts
        #[arg(short, long)]
        interactive: bool,
    },

    /// List all tasks
    List {
        /// Filter by status (pending/active/completed/archived)
        #[arg(short, long)]
        status: Option<String>,
        /// Show TUI interface
        #[arg(long)]
        tui: bool,
    },

    /// Remove a task
    Remove {
        /// Task name (optional if using --interactive)
        name: Option<String>,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
        /// Interactive mode - select task from list
        #[arg(short, long)]
        interactive: bool,
    },

    /// Execute a task in tmux with Claude
    Execute {
        /// Task name (optional if using --interactive)
        name: Option<String>,
        /// Command to execute (default: claude)
        #[arg(short, long)]
        command: Option<String>,
        /// Interactive mode - select task from list
        #[arg(short, long)]
        interactive: bool,
    },

    /// Attach to a running task's tmux session
    Attach {
        /// Task name (optional if using --interactive)
        name: Option<String>,
        /// Interactive mode - select task from list
        #[arg(short, long)]
        interactive: bool,
    },

    /// Kill a running task
    Kill {
        /// Task name (optional if using --interactive)
        name: Option<String>,
        /// Force kill without confirmation
        #[arg(short, long)]
        force: bool,
        /// Interactive mode - select task from list
        #[arg(short, long)]
        interactive: bool,
    },

    /// Show logs for a task
    Logs {
        /// Task name (optional if using --interactive)
        name: Option<String>,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        /// Show all executions history
        #[arg(short, long)]
        all: bool,
        /// Interactive mode - select task from list
        #[arg(short, long)]
        interactive: bool,
    },

    /// Show project status
    Status,

    /// Worktree management commands
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommands,
    },

    /// Cleanup stale worktrees
    Clean {
        /// Remove worktrees older than N days
        #[arg(long, value_name = "DAYS")]
        stale: Option<i64>,
        /// Remove orphaned worktrees (no DB record)
        #[arg(long)]
        orphans: bool,
        /// Clean both stale and orphaned worktrees
        #[arg(long)]
        all: bool,
        /// Preview without deleting
        #[arg(long)]
        dry_run: bool,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum WorktreeCommands {
    /// List all worktrees
    List {
        /// Show worktrees older than N days
        #[arg(long)]
        stale: Option<i64>,
    },

    /// Remove a specific worktree
    Remove {
        /// Worktree name
        name: String,
    },

    /// Cleanup all stale worktrees
    Cleanup {
        /// Threshold in days (default: 14)
        #[arg(short, long, default_value = "14")]
        days: i64,
        /// Dry run mode
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // No subcommand provided - launch TUI
            tui::run_tui_standalone().await
        }
        Some(Commands::Init { name, git_url, interactive }) => {
            cli::init::init(name, git_url, interactive).await
        }
        Some(Commands::Add {
            name,
            description,
            priority,
            auto_assign,
            interactive,
        }) => cli::add_task(name, description, priority, auto_assign, interactive).await,
        Some(Commands::List { status, tui }) => {
            cli::list_tasks(status, tui).await
        }
        Some(Commands::Remove { name, force, interactive }) => {
            cli::remove_task(name, force, interactive).await
        }
        Some(Commands::Execute { name, command, interactive }) => {
            cli::execute_task(name, command, interactive).await
        }
        Some(Commands::Attach { name, interactive }) => {
            cli::attach_task(name, interactive).await
        }
        Some(Commands::Kill { name, force, interactive }) => {
            cli::kill_task(name, interactive, force).await
        }
        Some(Commands::Logs { name, follow, all, interactive }) => {
            cli::logs_task(name, follow, interactive, all).await
        }
        Some(Commands::Status) => {
            println!("📊 Project Status");
            println!("⚠️  Implementation coming in Phase 7");
            Ok(())
        }
        Some(Commands::Worktree { command }) => match command {
            WorktreeCommands::List { stale } => {
                println!("🌳 Listing worktrees");
                if let Some(days) = stale {
                    println!("   Stale filter: {}+ days", days);
                }
                println!("⚠️  Implementation coming in Phase 3");
                Ok(())
            }
            WorktreeCommands::Remove { name } => {
                println!("🗑️  Removing worktree '{}'", name);
                println!("⚠️  Implementation coming in Phase 3");
                Ok(())
            }
            WorktreeCommands::Cleanup { days, dry_run } => {
                println!("🧹 Cleaning up worktrees older than {} days", days);
                if dry_run {
                    println!("   Dry run: enabled");
                }
                println!("⚠️  Implementation coming in Phase 6");
                Ok(())
            }
        },
        Some(Commands::Clean {
            stale,
            orphans,
            all,
            dry_run,
            force,
        }) => {
            println!("🧹 Cleanup mode");
            if let Some(days) = stale {
                println!("   Stale threshold: {} days", days);
            }
            if orphans {
                println!("   Orphans: enabled");
            }
            if all {
                println!("   All: enabled");
            }
            if dry_run {
                println!("   Dry run: enabled");
            }
            if force {
                println!("   Force: enabled");
            }
            println!("⚠️  Implementation coming in Phase 6");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Verify CLI can be instantiated
        let _cli = Cli::parse_from(&["kayfabe", "--help"]);
    }
}
