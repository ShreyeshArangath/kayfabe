# kayfabe

<img src="assets/logo.png" alt="kayfabe logo" width="400" />

> **AI-Assisted Development CLI** — Zero-friction worktree management + agent configuration for Claude Code, Cursor, and Windsurf

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/sarangat/kayfabe/ci.yml?branch=main)](https://github.com/sarangat/kayfabe/actions)

---

## Why Kayfabe?

AI coding assistants are powerful, but they need context to perform their role convincingly. **Kayfabe** automates the setup:

- 🎯 **One command** transforms any repo into an AI-ready worktree structure
- 🤖 **Agent-aware** — auto-generates `CLAUDE.md`, `.cursorrules`, `.windsurfrules`
- 🔀 **Multi-agent workflows** — parallel AI sessions on isolated branches
- ⚡ **Zero configuration** — smart defaults for Rust, Python, TypeScript, and more
- 🧹 **Intelligent cleanup** — staleness detection keeps your worktrees tidy

### The Problem

```bash
# Without kayfabe: Manual setup for each AI session
git clone my-repo
cd my-repo
git worktree add wt/feature-x main
# Now manually create CLAUDE.md, .cursorrules, etc.
code wt/feature-x
```

### The Solution

```bash
# With kayfabe: One command, fully configured
kayfabe init my-repo --agent all
kayfabe worktree create feature-x --open cursor
# ✓ Worktree created
# ✓ Agent configs generated
# ✓ IDE launched
```

---

## Quick Start

### Installation

**macOS** (Homebrew)
```bash
brew install sarangat/tap/kayfabe
```

**Linux / macOS** (Cargo)
```bash
cargo install kayfabe
```

**From source**
```bash
git clone https://github.com/sarangat/kayfabe.git
cd kayfabe
cargo install --path .
```

### 30-Second Setup

```bash
# Initialize your repository
kayfabe init ~/projects/my-repo --agent all

# Create your first worktree
kayfabe worktree create feature-auth --open cursor

# Done! Your IDE opens with:
# ✓ New git worktree
# ✓ CLAUDE.md with project context
# ✓ .cursorrules with conventions
# ✓ .windsurfrules for Windsurf
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
- `--template <TPL>` — Apply workflow template during setup
- `--force` — Overwrite existing configurations

**Examples:**
```bash
# Initialize current directory with all agents
kayfabe init

# Initialize specific repo for Cursor only
kayfabe init ~/projects/myapp --agent cursor

# Use a team template
kayfabe init ~/projects/myapp --template team-rust-standard
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
- `--template <TPL>` — Apply workflow template

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
- `--template <TPL>` — Use specific template
- `--analyze` — Analyze codebase for smart defaults
- `--output <PATH>` — Custom output path
- `--force` — Overwrite existing files

**Examples:**
```bash
# Generate all agent configs with analysis
kayfabe config generate --agent all --analyze

# Generate Cursor rules only
kayfabe config generate --agent cursor

# Use custom template
kayfabe config generate --template team-standards
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

### `kayfabe template` — Manage Workflow Templates

#### List available templates

```bash
kayfabe template list [OPTIONS]
```

**Options:**
- `--remote` — Include remote templates
- `--category <CAT>` — Filter by category

#### Create a template

```bash
kayfabe template create <NAME> [OPTIONS]
```

**Options:**
- `--from-current` — Create from current worktree setup
- `--description <DESC>` — Template description

**Examples:**
```bash
# Create from current setup
kayfabe template create team-rust-standard --from-current \
  --description "Team Rust project defaults"
```

#### Apply a template

```bash
kayfabe template apply <NAME> [PATH]
```

**Examples:**
```bash
# Apply to current worktree
kayfabe template apply team-rust-standard

# Apply to specific path
kayfabe template apply team-rust-standard ~/projects/newrepo
```

#### Export/Import templates

```bash
kayfabe template export <NAME> <OUTPUT>
kayfabe template import <PATH>
```

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
template = "default"

[agents.cursor]
enabled = true
template = "default"

[agents.windsurf]
enabled = true
template = "default"

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

### Flow 3: Team Onboarding

```bash
# Team lead: Create and share template
kayfabe init ~/projects/company-api --agent all --analyze
kayfabe template create company-api-standard --from-current \
  --description "Company API project standards"
kayfabe template export company-api-standard ~/templates/company-api.toml

# Team member: Use template
kayfabe init ~/projects/company-api --template company-api-standard
kayfabe worktree create feature-users --open cursor
```

### Flow 4: Repository Hygiene

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
| Workflow templates | ✓ | ✗ | ✗ | ✓ |
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
git clone https://github.com/sarangat/kayfabe.git
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
├── templates/        # Template system
├── ide/              # IDE launching
├── ui/               # User interface
└── error.rs          # Error types
```

---

## Roadmap

### v1.0 (Current)
- ✓ Core worktree management
- ✓ Agent configuration generation
- ✓ IDE launching
- ✓ Staleness detection & cleanup
- ✓ Configuration system

### v1.1 (Planned)
- [ ] Shell completions (bash, zsh, fish)
- [ ] Interactive mode with fuzzy selection
- [ ] More built-in templates (Go, Java, Ruby)
- [ ] Workflow hooks (pre/post operations)

### v2.0 (Future)
- [ ] Remote template repository
- [ ] Team configuration sharing
- [ ] Session tracking & analytics
- [ ] VS Code extension

---

## License

MIT — See [LICENSE](LICENSE) for details.

---

## Etymology

**Kayfabe** (wrestling term): The portrayal of staged events as real. In AI-assisted coding, we maintain the "kayfabe" that AI agents are knowledgeable about our codebase—by properly configuring them with the context they need to perform their role convincingly.

---

## Questions?

- 📖 [Full Documentation](docs/)
- 💬 [GitHub Discussions](https://github.com/sarangat/kayfabe/discussions)
- 🐛 [Report Issues](https://github.com/sarangat/kayfabe/issues)
- 🌟 [Star us on GitHub](https://github.com/sarangat/kayfabe)
