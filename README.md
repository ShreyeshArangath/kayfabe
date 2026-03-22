# kayfabe

An AI-enhanced CLI task manager built in Rust that accelerates engineer productivity by combining task management with git worktree orchestration. Manage a swarm of AI agents working in parallel across different tasks, each in its own isolated git worktree.

## Claude Code Plugin

Kayfabe ships as a Claude Code plugin with skills and specialized agents for code review.

### Skills

- **code-review** — Orchestrates 4 parallel subagents (bug-catcher, style-enforcer, systems-auditor, ux-advocate) to provide comprehensive code reviews. Triggered by asking Claude to "review my code", "review this PR", etc.

### Installation

Add the kayfabe plugin to your Claude Code settings (`~/.claude/settings.json`):

```json
{
  "plugins": [
    {
      "name": "kayfabe",
      "marketplace": "kayfabe"
    }
  ]
}
```

Or install from a local clone:

```json
{
  "plugins": [
    {
      "name": "kayfabe",
      "source": "/path/to/kayfabe"
    }
  ]
}
```

### Usage

Once installed, the plugin's skills are available automatically in Claude Code:

```
> review my code
> review this PR
> audit my changes before merge
```

The code review skill will detect your changes (staged, unstaged, or branch diff), then launch 4 specialized agents in parallel to analyze correctness, style, system design, and UX.
