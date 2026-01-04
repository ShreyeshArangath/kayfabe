pub const UNIVERSAL_AGENT_TEMPLATE: &str = r#"# Kayfabe AI Development Assistant

This project uses kayfabe for AI-assisted development with git worktree management.

## Available Kayfabe Commands
- `kayfabe worktree create <name>` - Create new isolated worktree
- `kayfabe worktree list` - List all worktrees
- `kayfabe worktree list --stale 14` - List stale worktrees (older than 14 days)
- `kayfabe worktree remove <name>` - Remove a worktree
- `kayfabe worktree cleanup` - Clean up stale worktrees
- `kayfabe status` - Show repository status

## Development Workflow
1. Create feature worktrees for isolated development
2. Use appropriate IDE integration for AI assistance
3. Maintain clean commit history
4. Clean up merged worktrees regularly

## Code Quality Standards
- Write tests for new functionality
- Follow project-specific style guides
- Keep commits atomic and well-described
- Ensure all tests pass before merging

## Kayfabe Integration
This project is managed by kayfabe. You can:
- Create worktrees from any directory in the repository
- Switch between worktrees seamlessly
- Access project context from any worktree location

## Working with Worktrees
When you create a worktree, you get a completely isolated environment:
- Each worktree has its own branch
- No conflicts with other worktrees
- Can work on multiple features in parallel
- Easy to merge back to main when done

## Best Practices
- Create descriptive worktree names (e.g., `feature-auth`, `fix-memory-leak`)
- Run `kayfabe status` to understand current repository state
- Clean up merged worktrees regularly with `kayfabe worktree cleanup`
- Work from any directory - kayfabe auto-detects the repository root
"#;
