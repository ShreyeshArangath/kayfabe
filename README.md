# kayfabe

<img src="assets/logo.png" alt="kayfabe logo" width="400" />

> **AI-Assisted Development CLI** — Git worktree management and agent configuration for modern development workflows

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/ShreyeshArangath/kayfabe/ci.yml?branch=main)](https://github.com/ShreyeshArangath/kayfabe/actions)

---

## What is Kayfabe?

**Kayfabe** (wrestling term): The art of maintaining the illusion that staged events are real. In AI-assisted development, we do the same—we maintain the fiction that AI agents deeply understand your codebase by giving them the context they need to perform convincingly.

Kayfabe is a CLI that automates the setup and management of isolated development environments for AI agents. It handles worktree creation, agent configuration, IDE launching, and workspace hygiene—so you can focus on shipping code with AI, not managing environments.

**The vision:**
- 🎯 **One command** to set up a fully configured AI-ready worktree
- 🤖 **Agent-aware** — auto-generates `.windsurfrules` for better AI context
- 🔀 **Multi-agent workflows** — parallel AI sessions on isolated branches
- ⚡ **Zero configuration** — smart defaults that just work
- 🧹 **Intelligent cleanup** — staleness detection keeps your worktrees tidy

---

## Installation

```bash
cargo install --git https://github.com/ShreyeshArangath/kayfabe.git
```

---

## Quick Start

```bash
# Initialize repository for AI-assisted development
kayfabe init

# Create a new worktree and open in Windsurf
kayfabe worktree create feature-auth --open windsurf

# List all worktrees
kayfabe worktree list

# Clean up stale worktrees
kayfabe worktree cleanup
```

## Commands

### `kayfabe init`
Initialize a repository for AI-assisted development.

```bash
kayfabe init [PATH] [OPTIONS]
```

Options:
- `--no-convert` - Don't convert to worktree layout
- `--agent <AGENT>` - Configure for specific agent (windsurf)

### `kayfabe worktree`
Manage git worktrees.

#### Create worktree
```bash
kayfabe worktree create <NAME> [OPTIONS]
```

Options:
- `--base <BASE>` - Base branch (default: main)
- `--open <OPEN>` - Launch IDE (windsurf|idea|code)
- `--no-open` - Don't launch any IDE

#### List worktrees
```bash
kayfabe worktree list [--stale DAYS]
```

#### Remove worktree
```bash
kayfabe worktree remove <NAME> [--force]
```

#### Cleanup stale worktrees
```bash
kayfabe worktree cleanup [OPTIONS]
```

Options:
- `--older-than <DAYS>` - Staleness threshold (default: 14)
- `--dry-run` - Preview without making changes
- `--force` - Skip confirmation prompt
- `--include-unmerged` - Also remove worktrees with unmerged commits

### `kayfabe config`
Manage agent configurations.

```bash
kayfabe config show [AGENT]
kayfabe config edit [AGENT]
kayfabe config validate
kayfabe config init
```

### `kayfabe install`
Install kayfabe agents globally.

```bash
kayfabe install [PATH] [OPTIONS]
```

Options:
- `--non-interactive` - Non-interactive mode
- `--agents <AGENTS>` - Agents to install (windsurf)

### `kayfabe status`
Show current repository and worktree status.

```bash
kayfabe status
```

## Development

```bash
# Clone the repo
git clone https://github.com/ShreyeshArangath/kayfabe.git
cd kayfabe

# Run tests
cargo test

# Build release binary
cargo build --release

# Run with local changes
cargo run -- init ~/test-repo
```

### Project Structure

```
src/
├── cli/              # Command implementations
├── git/              # Git operations
├── agents/           # Agent config generation
├── config/           # Configuration management
├── ide/              # IDE launching
└── error.rs          # Error types
```

---

## Core Concepts

### Worktree Layout

Kayfabe organizes your repository into a clean structure:

```
my-repo/
├── main/                    # Main checkout (anchor point)
│   ├── src/
│   ├── Cargo.toml
│   └── .windsurfrules       # Auto-generated agent config
├── wt/                      # Worktree directory
│   ├── feature-auth/        # Isolated feature branch
│   ├── feature-api/         # Another feature
│   └── spike-redis/         # Experimental work
└── .kayfabe/
    └── config.toml          # Project-level settings
```

**Why this layout?**
- Each worktree is a separate checkout — no branch conflicts
- `main/` is your anchor point — always up-to-date
- Clean separation between experimental and stable work
- IDE can open multiple worktrees simultaneously

### Agent Configuration

Kayfabe auto-generates context files for AI assistants:

| File | Agent | Purpose |
|------|-------|---------|
| `.windsurfrules` | Windsurf | Project overview, commands, architecture |

These are generated from your codebase — no manual editing needed.

---

## Workflows

### Feature Development

```bash
# Start a new feature
kayfabe worktree create feature-auth --open windsurf

# Work in isolation
# ... make commits ...

# When done, merge back and cleanup
kayfabe worktree remove feature-auth
```

### Repository Hygiene

```bash
# Check for stale worktrees
kayfabe worktree list --stale

# Preview cleanup
kayfabe worktree cleanup --dry-run

# Clean up old worktrees
kayfabe worktree cleanup --older-than 30
```

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT — See [LICENSE](LICENSE) for details.

---

## Questions?

- 💬 [GitHub Discussions](https://github.com/ShreyeshArangath/kayfabe/discussions)
- 🐛 [Report Issues](https://github.com/ShreyeshArangath/kayfabe/issues)
- 🌟 [Star us on GitHub](https://github.com/ShreyeshArangath/kayfabe)
