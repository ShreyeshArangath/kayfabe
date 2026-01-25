use anyhow::{anyhow, Result};

/// Command that can be executed in the TUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Init { name: Option<String>, git_url: Option<String> },
    Add { name: Option<String>, description: Option<String>, interactive: bool },
    Remove { name: String, force: bool },
    Execute { name: String },
    Attach { name: String },
    Kill { name: String },
    Status,
    Help,
    Refresh,
    Quit,
}

/// Parse a command string (e.g., ":add task-name" or "/init project-name git-url")
pub fn parse_command(input: &str) -> Result<Command> {
    let input = input.trim();

    if !input.starts_with('/') && !input.starts_with(':') {
        return Err(anyhow!("Commands must start with ':' or '/'"));
    }

    let parts: Vec<&str> = input[1..].split_whitespace().collect();

    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    let cmd = parts[0].to_lowercase();
    let args = &parts[1..];

    match cmd.as_str() {
        "init" => {
            // Support both direct mode (/init name url) and interactive mode (/init)
            if args.is_empty() {
                Ok(Command::Init {
                    name: None,
                    git_url: None,
                })
            } else if args.len() >= 2 {
                Ok(Command::Init {
                    name: Some(args[0].to_string()),
                    git_url: Some(args[1].to_string()),
                })
            } else {
                return Err(anyhow!("Usage: /init [<project-name> <git-url>] (no args for interactive)"));
            }
        }
        "add" => {
            // Support both direct mode (/add name desc) and interactive mode (/add)
            if args.is_empty() {
                // Interactive mode - no arguments provided
                Ok(Command::Add {
                    name: None,
                    description: None,
                    interactive: true,
                })
            } else {
                // Direct mode - arguments provided
                let name = args[0].to_string();
                let description = if args.len() > 1 {
                    Some(args[1..].join(" "))
                } else {
                    None
                };
                Ok(Command::Add {
                    name: Some(name),
                    description,
                    interactive: false,
                })
            }
        }
        "remove" | "rm" => {
            if args.is_empty() {
                return Err(anyhow!("Usage: /remove <task-name> [--force]"));
            }
            let name = args[0].to_string();
            let force = args.len() > 1 && args[1] == "--force";
            Ok(Command::Remove { name, force })
        }
        "execute" | "exec" => {
            if args.is_empty() {
                return Err(anyhow!("Usage: /execute <task-name>"));
            }
            Ok(Command::Execute {
                name: args[0].to_string(),
            })
        }
        "attach" => {
            if args.is_empty() {
                return Err(anyhow!("Usage: /attach <task-name>"));
            }
            Ok(Command::Attach {
                name: args[0].to_string(),
            })
        }
        "kill" => {
            if args.is_empty() {
                return Err(anyhow!("Usage: /kill <task-name>"));
            }
            Ok(Command::Kill {
                name: args[0].to_string(),
            })
        }
        "status" => Ok(Command::Status),
        "help" => Ok(Command::Help),
        "refresh" => Ok(Command::Refresh),
        "quit" | "q" | "exit" => Ok(Command::Quit),
        _ => Err(anyhow!("Unknown command: {}. Type /help for available commands", cmd)),
    }
}

/// Get help text for all available commands
pub fn get_command_help() -> Vec<(&'static str, &'static str)> {
    vec![
        ("/init [name] [url]", "Initialize project (no args for wizard)"),
        ("/add [name] [desc]", "Add task (no args for wizard)"),
        ("/remove <name>", "Remove a task"),
        ("/execute <name>", "Execute a task"),
        ("/attach <name>", "Attach to running task"),
        ("/kill <name>", "Kill running task"),
        ("/status", "Show project status"),
        ("/refresh", "Refresh task list"),
        ("/help", "Show this help"),
        ("/quit", "Exit Kayfabe"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_init() {
        let cmd = parse_command("/init myproject https://github.com/user/repo.git").unwrap();
        match cmd {
            Command::Init { name, git_url } => {
                assert_eq!(name, Some("myproject".to_string()));
                assert_eq!(git_url, Some("https://github.com/user/repo.git".to_string()));
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_interactive() {
        let cmd = parse_command("/init").unwrap();
        match cmd {
            Command::Init { name, git_url } => {
                assert_eq!(name, None);
                assert_eq!(git_url, None);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_add() {
        let cmd = parse_command("/add task1 Fix the bug").unwrap();
        match cmd {
            Command::Add { name, description, interactive } => {
                assert_eq!(name, Some("task1".to_string()));
                assert_eq!(description, Some("Fix the bug".to_string()));
                assert_eq!(interactive, false);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_interactive() {
        let cmd = parse_command("/add").unwrap();
        match cmd {
            Command::Add { name, description, interactive } => {
                assert_eq!(name, None);
                assert_eq!(description, None);
                assert_eq!(interactive, true);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_remove() {
        let cmd = parse_command("/remove task1 --force").unwrap();
        match cmd {
            Command::Remove { name, force } => {
                assert_eq!(name, "task1");
                assert!(force);
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_invalid_command() {
        let result = parse_command("/invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_slash() {
        let result = parse_command("add task1");
        assert!(result.is_err());
    }
}
