mod db;
mod errors;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kayfabe")]
#[command(version, about = "AI-enhanced CLI task manager with git worktree orchestration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Kayfabe project
    Init {
        /// Project name
        name: String,
        /// Git repository URL (SSH or HTTPS)
        git_url: String,
    },

    /// Add a new task
    Add {
        /// Task name
        name: String,
        /// Task description
        #[arg(short, long)]
        description: Option<String>,
        /// Priority (high/medium/low)
        #[arg(short, long)]
        priority: Option<String>,
        /// Auto-assign to available agent
        #[arg(long)]
        auto_assign: bool,
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
        /// Task name
        name: String,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Execute a task in tmux with Claude
    Execute {
        /// Task name
        name: String,
        /// Command to execute (default: claude)
        #[arg(short, long)]
        command: Option<String>,
    },

    /// Attach to a running task's tmux session
    Attach {
        /// Task name
        name: String,
    },

    /// Kill a running task
    Kill {
        /// Task name
        name: String,
    },

    /// Show logs for a task
    Logs {
        /// Task name
        name: String,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
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
        Commands::Init { name, git_url } => {
            println!("🚀 Initializing project '{}' from {}", name, git_url);
            println!("⚠️  Implementation coming in Phase 2");
            Ok(())
        }
        Commands::Add {
            name,
            description,
            priority,
            auto_assign,
        } => {
            println!("➕ Adding task '{}'", name);
            if let Some(desc) = description {
                println!("   Description: {}", desc);
            }
            if let Some(pri) = priority {
                println!("   Priority: {}", pri);
            }
            if auto_assign {
                println!("   Auto-assign: enabled");
            }
            println!("⚠️  Implementation coming in Phase 3");
            Ok(())
        }
        Commands::List { status, tui } => {
            if tui {
                println!("📊 Launching TUI...");
                println!("⚠️  Implementation coming in Phase 4");
            } else {
                println!("📋 Listing tasks");
                if let Some(s) = status {
                    println!("   Filter: {}", s);
                }
                println!("⚠️  Implementation coming in Phase 3");
            }
            Ok(())
        }
        Commands::Remove { name, force } => {
            println!("🗑️  Removing task '{}'", name);
            if force {
                println!("   Force: enabled");
            }
            println!("⚠️  Implementation coming in Phase 3");
            Ok(())
        }
        Commands::Execute { name, command } => {
            let cmd = command.unwrap_or_else(|| "claude".to_string());
            println!("▶️  Executing task '{}' with command: {}", name, cmd);
            println!("⚠️  Implementation coming in Phase 5");
            Ok(())
        }
        Commands::Attach { name } => {
            println!("🔗 Attaching to task '{}'", name);
            println!("⚠️  Implementation coming in Phase 5");
            Ok(())
        }
        Commands::Kill { name } => {
            println!("💀 Killing task '{}'", name);
            println!("⚠️  Implementation coming in Phase 5");
            Ok(())
        }
        Commands::Logs { name, follow } => {
            println!("📜 Showing logs for task '{}'", name);
            if follow {
                println!("   Following: enabled");
            }
            println!("⚠️  Implementation coming in Phase 5");
            Ok(())
        }
        Commands::Status => {
            println!("📊 Project Status");
            println!("⚠️  Implementation coming in Phase 7");
            Ok(())
        }
        Commands::Worktree { command } => match command {
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
        Commands::Clean {
            stale,
            orphans,
            all,
            dry_run,
            force,
        } => {
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
