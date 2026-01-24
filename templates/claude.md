# Claude AI Instructions

This file provides context and instructions for Claude Code when working on tasks in this Kayfabe project.

## Project Context

This is a Kayfabe-managed project. Key things to know:

1. **Isolated Worktrees**: Each task runs in its own git worktree with a dedicated branch
2. **Parallel Development**: Multiple tasks can be worked on simultaneously without conflicts
3. **Thought Tracking**: Your reasoning and artifacts are saved in `thoughts/<task-name>/`

## Working with Kayfabe Tasks

When you're executing a task via `kayfabe execute <task-name>`:

- You're in an isolated worktree at `../<project>-wt/<task-name>/`
- Your branch is `kayfabe/<task-name>`
- Save important thoughts/artifacts to `../../thoughts/<task-name>/`
- The worktree is fully isolated - feel free to experiment

## Thought and Artifact Format

Save your reasoning and outputs as markdown files in the thoughts directory:

```markdown
---
type: thought
task: feature-x
timestamp: 2026-01-23T14:05:30Z
tags: [planning, architecture]
---

# Planning Feature X

Your reasoning and planning content here...
```

Types: `thought`, `artifact`, `error`, `summary`

## Development Workflow

1. Understand the task context from the task description
2. Plan your approach and save thoughts
3. Implement changes in the worktree
4. Document key decisions and learnings
5. Commit your work with clear messages

## Tips for AI Assistants

- **Be thorough**: Document your reasoning for complex decisions
- **Be organized**: Use clear file names and structure in thoughts/
- **Be cautious**: This is an isolated worktree, but still respect the codebase
- **Be helpful**: Leave good commit messages and documentation
- **Be efficient**: You have full context of the project via kayfabe.md

## Project-Specific Guidelines

[Add any project-specific coding standards, conventions, or requirements here]

## Additional Context

[Add any additional context that would help AI assistants work effectively on this project]
