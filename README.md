# kayfabe

<img src="assets/logo.png" alt="kayfabe logo" width="400" />

> **AI-Assisted Development CLI** — Zero-friction worktree management + agent configuration for Claude Code, Cursor, and Windsurf

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/ShreyeshArangath/kayfabe/ci.yml?branch=main)](https://github.com/ShreyeshArangath/kayfabe/actions)

---

## What is Kayfabe?

**Kayfabe** (wrestling term): The art of maintaining the illusion that staged events are real. In AI-assisted development, we do the same—we maintain the fiction that AI agents deeply understand your codebase by giving them the context they need to perform convincingly.

Kayfabe is a CLI that automates the setup and management of isolated development environments for AI agents (Claude, Cursor, Windsurf). It handles worktree creation, agent configuration, IDE launching, and workspace hygiene—so you can focus on shipping code with AI, not managing environments.

**Why it matters:**
- 🎯 **One command** to set up a fully configured AI-ready worktree
- 🤖 **Agent-aware** — auto-generates `CLAUDE.md`, `.cursorrules`, `.windsurfrules`
- 🔀 **Multi-agent workflows** — parallel AI sessions on isolated branches
- ⚡ **Zero configuration** — smart defaults for Rust, Python, TypeScript, and more
- 🧹 **Intelligent cleanup** — staleness detection keeps your worktrees tidy

---

## Quick Start

### Installation

**With Cargo** (requires Rust):

```bash
cargo install --git https://github.com/ShreyeshArangath/kayfabe.git
```

**With Homebrew** (macOS):

```bash
brew install ShreyeshArangath/tap/kayfabe
```

### Get Started

```bash
kayfabe init ~/projects/my-repo --agent all && kayfabe worktree create feature-auth --open cursor
```

That's it. Your IDE opens with a fully configured worktree, agent configs, and project context ready to go.

---

## Core Concepts

### Worktree Layout

Kayfabe organizes your repository into a clean structure:

```
my-repo/
├── main/                    # Main checkout (anchor point)
│   ├── src/
│   ├── Cargo.toml
│   └── CLAUDE.md           # Auto-generated agent config
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
| `CLAUDE.md` | Claude Code | Project overview, commands, architecture |
| `.cursorrules` | Cursor | Code style, conventions, testing guidelines |
| `.windsurfrules` | Windsurf | Similar to Cursor rules |

These are generated from your codebase — no manual editing needed.

---

## Commands

### `kayfabe init` — Set Up Your Repository

Initialize a repository for AI-assisted development.

```bash
kayfabe init [PATH] [OPTIONS]
```

**Options:**
- `--agent <AGENT>` — Configure for specific agent: `claude`, `cursor`, `windsurf`, or `all` (default: `all`)
- `--no-convert` — Don't convert to worktree layout (use existing structure)
- `--force` — Overwrite existing configurations

**Examples:**
```bash
# Initialize current directory with all agents
kayfabe init

# Initialize specific repo for Cursor only
kayfabe init ~/projects/myapp --agent cursor
```

**What it does:**
1. Detects project type (Rust, Python, TypeScript, etc.)
2. Converts to worktree layout (`main/` + `wt/`)
3. Generates agent configuration files
4. Creates `.kayfabe/config.toml` for project settings

---

### `kayfabe worktree` — Manage Worktrees

#### Create a new worktree

```bash
kayfabe worktree create <NAME> [OPTIONS]
```

**Options:**
- `--base <BRANCH>` — Base branch (default: `main`)
- `--open <IDE>` — Launch IDE: `idea`, `cursor`, `windsurf`, `claude`, `code`
- `--no-open` — Don't launch any IDE
- `--from-ticket <ID>` — Name from ticket ID (e.g., `ENG-1234`)

**Examples:**
```bash
# Create and open in Cursor
kayfabe worktree create feature-auth --open cursor

# Create from ticket ID
kayfabe worktree create --from-ticket ENG-1234 --open claude

# Create without opening IDE
kayfabe worktree create spike-redis --no-open

# Create from custom base branch
kayfabe worktree create feature-api --base develop --open windsurf
```

#### List worktrees

```bash
kayfabe worktree list [OPTIONS]
```

**Options:**
- `--json` — Output as JSON (for scripting)
- `--remote` — Include remote branch status
- `--stale [DAYS]` — Show only stale worktrees (default: 14 days)

**Examples:**
```bash
# List all worktrees with status
kayfabe worktree list

# Show only stale worktrees
kayfabe worktree list --stale

# Show stale after 30 days
kayfabe worktree list --stale 30

# JSON output for automation
kayfabe worktree list --json | jq '.[] | select(.stale)'
```

#### Remove a worktree

```bash
kayfabe worktree remove <NAME> [OPTIONS]
```

**Options:**
- `--force` — Force removal even if unmerged
- `--delete-branch` — Also delete the associated branch

**Examples:**
```bash
# Remove a merged worktree
kayfabe worktree remove feature-auth

# Force remove (dangerous!)
kayfabe worktree remove spike-redis --force

# Remove and delete branch
kayfabe worktree remove feature-old --delete-branch
```

#### Clean up stale worktrees

```bash
kayfabe worktree cleanup [OPTIONS]
```

**Options:**
- `--older-than <DAYS>` — Staleness threshold (default: 14)
- `--dry-run` — Preview without making changes
- `--force` — Skip confirmation prompt
- `--include-unmerged` — Also remove worktrees with unmerged commits (⚠️ dangerous)
- `--delete-branches` — Also delete associated branches
- `--json` — Output results as JSON

**Examples:**
```bash
# Preview what would be cleaned
kayfabe worktree cleanup --dry-run

# Clean up worktrees inactive for 30+ days
kayfabe worktree cleanup --older-than 30

# Nuclear option (requires confirmation)
kayfabe worktree cleanup --older-than 90 --include-unmerged --delete-branches
```

**Safety Features:**
- ✓ Merged worktrees only (by default)
- ✓ Confirmation prompt for destructive operations
- ✓ Dry-run mode to preview changes
- ✓ Staleness detection using multiple signals (commits, file mtime, reflog)

#### Switch to a worktree

```bash
kayfabe worktree switch <NAME> [OPTIONS]
```

**Options:**
- `--open <IDE>` — Launch IDE after switching

**Examples:**
```bash
# Switch to existing worktree
kayfabe worktree switch feature-auth

# Switch and open in IDE
kayfabe worktree switch feature-api --open cursor
```

---

### `kayfabe config` — Manage Agent Configurations

#### Generate agent configs

```bash
kayfabe config generate [OPTIONS]
```

**Options:**
- `--agent <AGENT>` — Target agent: `claude`, `cursor`, `windsurf`, or `all`
- `--analyze` — Analyze codebase for smart defaults
- `--output <PATH>` — Custom output path
- `--force` — Overwrite existing files

**Examples:**
```bash
# Generate all agent configs with analysis
kayfabe config generate --agent all --analyze

# Generate Cursor rules only
kayfabe config generate --agent cursor
```

#### Show current configuration

```bash
kayfabe config show [AGENT]
```

**Examples:**
```bash
# Show all agent configs
kayfabe config show

# Show Cursor rules only
kayfabe config show cursor
```

#### Edit configuration

```bash
kayfabe config edit [AGENT]
```

Opens configuration in your default editor.

#### Validate configurations

```bash
kayfabe config validate
```

Checks all agent config files for correctness.

---


### `kayfabe status` — Show Repository Status

```bash
kayfabe status [OPTIONS]
```

**Options:**
- `--json` — Output as JSON

**Example output:**
```
Repository: ~/projects/my-repo
Layout: Worktree (main/ + wt/)
Project Type: Rust (Cargo workspace)

Worktrees:
  main              → main (anchor)
  feature-auth      → feature-auth (2 commits ahead)
  feature-api       → feature-api (0 commits)

Agent Configs:
  ✓ CLAUDE.md
  ✓ .cursorrules
  ✓ .windsurfrules

Configuration:
  Global: ~/.config/kayfabe/config.toml
  Project: .kayfabe/config.toml
```

---

## Configuration

### Global Config: `~/.config/kayfabe/config.toml`

Set your personal defaults:

```toml
[defaults]
ide = "cursor"                    # Default IDE to launch
base_branch = "main"              # Default base branch
auto_fetch = true                 # Fetch refs before creating worktree

[worktree]
layout = "standard"               # "standard" (main/wt/) or "flat"
naming = "branch"                 # "branch", "ticket", or "custom"
stale_days = 14                   # Days of inactivity before considered stale
auto_cleanup = false              # Prompt to cleanup stale worktrees on `list`
protect_unmerged = true           # Never auto-cleanup worktrees with unmerged work

[agents.claude]
enabled = true

[agents.cursor]
enabled = true

[agents.windsurf]
enabled = true

[ui]
color = true
progress = true
interactive = true
```

### Project Config: `.kayfabe/config.toml`

Override defaults per project:

```toml
[project]
name = "my-project"
type = "rust"                     # Auto-detected or manual

[worktree]
base_branch = "develop"           # Override default

[agents]
preferred = ["claude", "cursor"]

[hooks]
post_create = ["./scripts/setup-env.sh"]
```

---

## Real-World Workflows

### Flow 1: Solo Feature Development

```bash
# Start a new feature
kayfabe worktree create feature-auth --open cursor

# Work in isolation
# ... make commits ...

# When done, merge back
git -C wt/feature-auth push origin feature-auth
# Create PR, merge, then cleanup
kayfabe worktree remove feature-auth --delete-branch
```

### Flow 2: Multi-Agent Collaboration

```bash
# Agent 1: Claude Code for architecture
kayfabe worktree create feature-auth-design --open claude

# Agent 2: Cursor for implementation (based on design)
kayfabe worktree create feature-auth-impl --base feature-auth-design --open cursor

# Agent 3: Windsurf for testing
kayfabe worktree create feature-auth-test --base feature-auth-impl --open windsurf

# Check status
kayfabe worktree list
```

### Flow 3: Repository Hygiene

```bash
# Weekly cleanup
kayfabe worktree list --stale

# Preview what would be removed
kayfabe worktree cleanup --dry-run

# Remove stale merged worktrees
kayfabe worktree cleanup --older-than 21

# Aggressive cleanup (quarterly)
kayfabe worktree cleanup --older-than 90 --include-unmerged --delete-branches
```

---

## Supported Stacks

Kayfabe auto-detects and configures for:

| Language | Detection | Build | Test | Lint |
|----------|-----------|-------|------|------|
| **Rust** | `Cargo.toml` | `cargo build` | `cargo test` | `cargo clippy` |
| **Python** | `pyproject.toml`, `requirements.txt` | `pip install` | `pytest` | `ruff check` |
| **TypeScript** | `package.json` | `npm/yarn/pnpm build` | `npm test` | `eslint` |
| **Go** | `go.mod` | `go build` | `go test` | `golangci-lint` |
| **Java** | `pom.xml`, `build.gradle` | `mvn/gradle build` | `mvn/gradle test` | `checkstyle` |

---

## Comparison with Similar Tools

| Feature | kayfabe | git-worktree | direnv | humanlayer |
|---------|---------|--------------|--------|------------|
| Worktree management | ✓ | ✓ | ✗ | ✗ |
| IDE launching | ✓ | ✗ | ✗ | ✗ |
| Agent config generation | ✓ | ✗ | ✗ | ✗ |
| Remote sync | Planned | ✗ | ✗ | ✓ |

---

## Troubleshooting

### "Repository not found"

```bash
# Make sure you're in a git repository
git status

# Or specify the path explicitly
kayfabe init /path/to/repo
```

### "IDE not found"

Kayfabe looks for IDEs in your PATH. Make sure your IDE is installed and accessible:

```bash
# Check if Cursor is installed
which cursor

# Add to PATH if needed
export PATH="/Applications/Cursor.app/Contents/MacOS:$PATH"
```

### "Worktree already exists"

```bash
# List existing worktrees
kayfabe worktree list

# Remove the conflicting one
kayfabe worktree remove <name>

# Or use a different name
kayfabe worktree create <new-name>
```

### "Stale detection not working"

Staleness uses multiple signals (file mtime, git commits, reflog). If a worktree isn't detected as stale:

```bash
# Check the staleness report
kayfabe worktree list --json | jq '.[] | select(.name=="wt/old-feature")'

# Force removal if you're sure
kayfabe worktree remove wt/old-feature --force
```

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

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
├── ui/               # User interface
└── error.rs          # Error types
```

---

## License

MIT — See [LICENSE](LICENSE) for details.

---

## Questions?

- 💬 [GitHub Discussions](https://github.com/ShreyeshArangath/kayfabe/discussions)
- 🐛 [Report Issues](https://github.com/ShreyeshArangath/kayfabe/issues)
- 🌟 [Star us on GitHub](https://github.com/ShreyeshArangath/kayfabe)
