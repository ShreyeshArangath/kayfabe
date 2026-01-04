use clap::{Parser, Subcommand};
use kayfabe::cli::{ConfigCommand, InitCommand, InstallCommand, StatusCommand, WorktreeCommand};
use std::path::PathBuf;
use std::process;

pub use clap::Parser as ClapParser;

#[derive(Parser)]
#[command(name = "kayfabe")]
#[command(about = "AI-assisted development CLI", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, help = "Enable verbose output")]
    verbose: bool,

    #[arg(short, long, global = true, help = "Suppress non-error output")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize a repo for AI-assisted development")]
    Init {
        #[arg(help = "Repository path (default: current directory)")]
        path: Option<PathBuf>,

        #[arg(long, help = "Don't convert to worktree layout")]
        no_convert: bool,

        #[arg(long, help = "Configure for specific agent [windsurf]")]
        agent: Option<String>,
    },

    #[command(about = "Install kayfabe agents globally")]
    Install {
        #[arg(help = "Target directory (default: current directory)")]
        path: Option<PathBuf>,

        #[arg(long, help = "Non-interactive mode")]
        non_interactive: bool,

        #[arg(long, help = "Agents to install [windsurf]")]
        agents: Option<Vec<String>>,
    },

    #[command(about = "Manage worktrees")]
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommands,
    },

    #[command(about = "Manage agent configurations")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    #[command(about = "Show current repo/worktree status")]
    Status,
}

#[derive(Subcommand)]
enum WorktreeCommands {
    #[command(about = "Create a new worktree")]
    Create {
        #[arg(help = "Name of the worktree/branch")]
        name: String,

        #[arg(long, help = "Base branch (default: main)")]
        base: Option<String>,

        #[arg(long, help = "Launch IDE [windsurf|idea|code]")]
        open: Option<String>,

        #[arg(long, help = "Don't launch any IDE")]
        no_open: bool,
    },

    #[command(about = "List worktrees")]
    List {
        #[arg(long, help = "Show only stale worktrees (days)")]
        stale: Option<u64>,
    },

    #[command(about = "Remove a worktree")]
    Remove {
        #[arg(help = "Name of the worktree to remove")]
        name: String,

        #[arg(long, help = "Force removal even if unmerged")]
        force: bool,
    },

    #[command(about = "Cleanup stale worktrees")]
    Cleanup {
        #[arg(long, default_value = "14", help = "Staleness threshold in days")]
        older_than: u64,

        #[arg(long, help = "Preview what would be removed")]
        dry_run: bool,

        #[arg(long, help = "Skip confirmation prompt")]
        force: bool,

        #[arg(long, help = "Also remove worktrees with unmerged commits")]
        include_unmerged: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    #[command(about = "Show current configuration")]
    Show {
        #[arg(help = "Agent to show [windsurf]")]
        agent: Option<String>,
    },

    #[command(about = "Edit configuration in editor")]
    Edit {
        #[arg(help = "Agent to edit [windsurf]")]
        agent: Option<String>,
    },

    #[command(about = "Validate agent configurations")]
    Validate,

    #[command(about = "Initialize global configuration")]
    Init,
}

#[derive(Subcommand)]
enum TemplateCommands {
    #[command(about = "List available templates")]
    List,

    #[command(about = "Create a new template")]
    Create {
        #[arg(help = "Template name")]
        name: String,

        #[arg(long, help = "Template description")]
        description: Option<String>,
    },

    #[command(about = "Show template contents")]
    Show {
        #[arg(help = "Template name")]
        name: String,
    },

    #[command(about = "Delete a template")]
    Delete {
        #[arg(help = "Template name")]
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init {
            path,
            no_convert: _,
            agent: _,
        } => InitCommand::execute(path),

        Commands::Install {
            path,
            non_interactive,
            agents,
        } => InstallCommand::execute(path, non_interactive, agents),

        Commands::Worktree { command } => match command {
            WorktreeCommands::Create {
                name,
                base,
                open,
                no_open,
            } => WorktreeCommand::create(name, base, open, no_open),
            WorktreeCommands::List { stale } => WorktreeCommand::list(stale),
            WorktreeCommands::Remove { name, force } => WorktreeCommand::remove(name, force),
            WorktreeCommands::Cleanup {
                older_than,
                dry_run,
                force,
                include_unmerged,
            } => WorktreeCommand::cleanup(older_than, dry_run, force, include_unmerged),
        },

        Commands::Config { command } => match command {
            ConfigCommands::Show { agent } => ConfigCommand::show(agent),
            ConfigCommands::Edit { agent } => ConfigCommand::edit(agent),
            ConfigCommands::Validate => ConfigCommand::validate(),
            ConfigCommands::Init => ConfigCommand::init(),
        },

        Commands::Status => StatusCommand::execute(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
