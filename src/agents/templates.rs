pub const CLAUDE_TEMPLATE: &str = r#"# Kayfabe AI Development Assistant

## About This Project
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
2. Use appropriate IDE integration (cursor, windsurf, etc.)
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
"#;

pub const CURSOR_TEMPLATE: &str = r#"# Kayfabe Cursor Rules

This project uses kayfabe for AI-assisted development with git worktree management.

## Kayfabe Commands
- `kayfabe worktree create <name> [--open cursor]` - Create isolated development environment
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
- Use `--open cursor` to launch Cursor in the new worktree
- Run `kayfabe status` to understand current repository state
- Clean up merged worktrees regularly with `kayfabe worktree cleanup`
- Work from any directory - kayfabe auto-detects the repository root

## Code Quality
- Write comprehensive tests for new functionality
- Follow existing project conventions and style
- Keep commits atomic and well-described
- Ensure all tests pass before merging
- Document complex logic and public APIs
"#;

pub const WINDSURF_TEMPLATE: &str = r#"# Kayfabe Windsurf Rules

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
"#;
