# Kayfabe Project

This project is managed by [Kayfabe](https://github.com/shreyesharangath/kayfabe), an AI-enhanced CLI task manager with git worktree orchestration.

## What is Kayfabe?

Kayfabe accelerates engineer productivity by combining task management with isolated git worktrees. Each task gets its own worktree and git branch, enabling parallel development with AI assistants like Claude.

## Available Commands

### Task Management
```bash
kayfabe add <task-name>              # Create a new task with isolated worktree
kayfabe list                          # List all tasks
kayfabe list --tui                    # Launch interactive TUI
kayfabe remove <task-name>            # Remove a task and its worktree
kayfabe status                        # Show project status
```

### Task Execution
```bash
kayfabe execute <task-name>           # Execute task in tmux with Claude
kayfabe attach <task-name>            # Attach to running task session
kayfabe kill <task-name>              # Kill running task
kayfabe logs <task-name>              # View task execution logs
```

### Worktree Management
```bash
kayfabe worktree list                 # List all worktrees
kayfabe worktree list --stale 14      # List worktrees older than 14 days
kayfabe worktree remove <name>        # Remove a specific worktree
kayfabe worktree cleanup              # Clean up stale worktrees
```

### Cleanup
```bash
kayfabe clean --stale 7d              # Clean worktrees older than 7 days
kayfabe clean --orphans               # Clean orphaned worktrees
kayfabe clean --all --dry-run         # Preview cleanup without deleting
```

## Workflow

1. **Create a task**: `kayfabe add feature-x --description "Add new feature"`
2. **Work on it**: The task automatically gets an isolated worktree and git branch
3. **Execute with AI**: `kayfabe execute feature-x` launches Claude in the task worktree
4. **Track progress**: AI thoughts and artifacts are saved in `thoughts/feature-x/`
5. **Clean up**: `kayfabe remove feature-x` removes the task and worktree when done

## Directory Structure

```
project/
├── .git/                    # Bare repository
├── project-wt/              # Worktree base directory
│   ├── feature-x/           # Task worktree (isolated)
│   ├── feature-y/           # Another task worktree
│   └── ...
├── thoughts/                # AI artifacts and thoughts
│   ├── feature-x/
│   │   ├── 2026-01-23_140530_thought.md
│   │   ├── 2026-01-23_141205_artifact.md
│   │   └── summary.md
│   └── ...
├── .kayfabe/                # Local Kayfabe state
├── kayfabe.md               # This file
└── claude.md                # Claude AI instructions
```

## Tips

- Each task has its own isolated worktree - no conflicts!
- Use `kayfabe execute` to run AI assistants in task context
- The TUI (`kayfabe list --tui`) provides a visual overview
- Stale worktrees can be cleaned up automatically
- All AI interactions are tracked in `thoughts/` for review

## Learn More

- GitHub: https://github.com/shreyesharangath/kayfabe
- Documentation: See implementation-plan.md for architecture details
