use crate::error::Result;
use std::path::Path;

pub struct AgentInstaller;

impl AgentInstaller {
    pub fn install_global(agent: &str, target_dir: &Path) -> Result<()> {
        match agent {
            "windsurf" => Self::install_windsurf_global(target_dir),
            _ => Err(crate::error::KayfabeError::Other(format!(
                "Unknown agent: {}",
                agent
            ))),
        }
    }

    fn install_windsurf_global(target_dir: &Path) -> Result<()> {
        let local_path = target_dir.join(".windsurfrules");
        let content = Self::generate_windsurf_config()?;
        std::fs::write(&local_path, &content)?;

        println!("✓ Windsurf agent installed locally");
        Ok(())
    }

    fn generate_windsurf_config() -> Result<String> {
        Ok(r#"# Kayfabe Windsurf Rules

This project uses kayfabe for AI-assisted development with git worktree management.

## Kayfabe Commands
- `kayfabe worktree create <name> [--open windsurf]` - Create isolated development environment
- `kayfabe worktree list [--stale]` - List all worktrees and their status  
- `kayfabe worktree remove <name>` - Remove completed worktree
- `kayfabe worktree cleanup` - Clean up stale worktrees
- `kayfabe status` - Show repository and worktree status

## Development Workflow
1. **Feature Development**: Create dedicated worktrees for each feature/task
2. **Isolation**: Each worktree is completely isolated - no branch conflicts
3. **Parallel Work**: Multiple agents can work on different features simultaneously
4. **Clean Merging**: Merge completed work back to main branch
5. **Cleanup**: Remove merged worktrees to keep workspace tidy

## Best Practices
- Create descriptive worktree names (e.g., `feature-auth`, `fix-memory-leak`)
- Use `--open windsurf` to launch Windsurf in the new worktree
- Run `kayfabe status` to understand current repository state
- Clean up merged worktrees regularly with `kayfabe worktree cleanup`
- Work from any directory - kayfabe auto-detects the repository root

## Code Quality
- Write comprehensive tests for new functionality
- Follow existing project conventions and style
- Keep commits atomic and well-described
- Ensure all tests pass before merging
- Document complex logic and public APIs
"#
        .to_string())
    }
}
