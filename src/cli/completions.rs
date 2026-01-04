use crate::error::Result;
use clap::Command;
use clap_complete::{generate, Shell};
use std::io;

pub struct CompletionsCommand;

impl CompletionsCommand {
    pub fn generate(shell: Shell) -> Result<()> {
        let mut cmd = build_cli();
        let bin_name = "kayfabe";

        generate(shell, &mut cmd, bin_name, &mut io::stdout());

        Ok(())
    }
}

fn build_cli() -> Command {
    Command::new("kayfabe")
        .about("AI-assisted development CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .about("Initialize a repo for AI-assisted development")
                .arg(clap::Arg::new("path").help("Repository path"))
                .arg(
                    clap::Arg::new("no-convert")
                        .long("no-convert")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(clap::Arg::new("agent").long("agent").value_name("AGENT")),
        )
        .subcommand(
            Command::new("worktree")
                .about("Manage worktrees")
                .subcommand_required(true)
                .subcommand(
                    Command::new("create")
                        .about("Create a new worktree")
                        .arg(clap::Arg::new("name").required(true))
                        .arg(clap::Arg::new("base").long("base"))
                        .arg(clap::Arg::new("open").long("open"))
                        .arg(
                            clap::Arg::new("no-open")
                                .long("no-open")
                                .action(clap::ArgAction::SetTrue),
                        ),
                )
                .subcommand(Command::new("list").about("List worktrees"))
                .subcommand(
                    Command::new("remove")
                        .about("Remove a worktree")
                        .arg(clap::Arg::new("name").required(true)),
                )
                .subcommand(Command::new("cleanup").about("Cleanup stale worktrees")),
        )
        .subcommand(
            Command::new("config")
                .about("Manage agent configurations")
                .subcommand_required(true)
                .subcommand(Command::new("generate").about("Generate agent configuration files"))
                .subcommand(Command::new("show").about("Show current configuration"))
                .subcommand(Command::new("edit").about("Edit configuration"))
                .subcommand(Command::new("validate").about("Validate agent configurations"))
                .subcommand(Command::new("init").about("Initialize global configuration")),
        )
        .subcommand(
            Command::new("template")
                .about("Manage workflow templates")
                .subcommand_required(true)
                .subcommand(Command::new("list").about("List available templates"))
                .subcommand(
                    Command::new("create")
                        .about("Create a new template")
                        .arg(clap::Arg::new("name").required(true)),
                )
                .subcommand(
                    Command::new("show")
                        .about("Show template contents")
                        .arg(clap::Arg::new("name").required(true)),
                )
                .subcommand(
                    Command::new("delete")
                        .about("Delete a template")
                        .arg(clap::Arg::new("name").required(true)),
                ),
        )
        .subcommand(
            Command::new("completions")
                .about("Generate shell completions")
                .arg(clap::Arg::new("shell").required(true).value_parser([
                    "bash",
                    "zsh",
                    "fish",
                    "powershell",
                ])),
        )
        .subcommand(Command::new("status").about("Show current repo/worktree status"))
}
