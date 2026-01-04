# Kayfabe Global Agents Restructure Implementation Plan

## Overview

This plan addresses three critical bugs in kayfabe by restructuring it to work more like vibe-tools with global agent installation, proper root detection, and streamlined command structure.

## Current State Analysis

### Problems Identified:
1. **Agent config generation is inefficient** - generates project-specific configs that don't provide real benefits
2. **No root awareness** - `.kayfabe` root has no sense of managing configurations across worktrees
3. **Useless agents command** - current agents command does nothing meaningful

### Current Architecture:
- Agent configs generated per-project in `@/Users/sarangat/personal/kayfabe/src/cli/config.rs:12-80`
- Project-specific `.kayfabe/config.toml` in `@/Users/sarangat/personal/kayfabe/src/config/project.rs:42-44`
- Global config in `~/.config/kayfabe/config.toml` via `@/Users/sarangat/personal/kayfabe/src/config/global.rs:8-13`
- Agent generators: `@/Users/sarangat/personal/kayfabe/src/agents/claude.rs`, `@/Users/sarangat/personal/kayfabe/src/agents/cursor.rs`, `@/Users/sarangat/personal/kayfabe/src/agents/windsurf.rs`

### Key Discoveries:
- Current system generates basic, templated configs per project
- No global agent installation mechanism like vibe-tools
- Root detection exists but doesn't manage global agent state
- Agent configs are hardcoded and not extensible

## Desired End State

### Target Architecture (inspired by vibe-tools):
1. **Global agent installation** - `kayfabe install` sets up agents globally like `vibe-tools install`
2. **Root-aware operations** - kayfabe works from any subdirectory, detecting `.kayfabe` root
3. **Streamlined commands** - remove agents command, integrate functionality into install/config
4. **Universal agent configs** - agents work across all projects without per-project generation

### Success Criteria:
- `kayfabe install` command installs agents globally to appropriate locations
- Commands work from any directory within a kayfabe-managed repository
- Agent configs are reusable across projects
- No more per-project agent config generation
- Agents command is removed

## What We're NOT Doing

- Changing the worktree management system (that works well)
- Modifying the core git operations
- Changing the IDE launching functionality
- Altering the template system

## Implementation Approach

Transform kayfabe from a project-centric to a global-centric tool while maintaining worktree functionality. Follow vibe-tools pattern of global installation with local detection.

## Phase 1: Global Agent Installation System

### Overview
Create a new `install` command that sets up agents globally, similar to `vibe-tools install`.

### Changes Required:

#### 1. New Install Command
**File**: `src/cli/install.rs`
**Changes**: Create new install command implementation

```rust
use crate::agents::{AgentInstaller, GlobalAgentConfig};
use crate::config::GlobalConfig;
use crate::error::Result;
use console::style;
use dialoguer::{MultiSelect, Confirm};

pub struct InstallCommand;

impl InstallCommand {
    pub fn execute(target_dir: Option<PathBuf>) -> Result<()> {
        let target = target_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
        
        // Detect available agents
        let available_agents = vec!["claude", "cursor", "windsurf", "cline", "roo"];
        
        // Interactive selection
        let selections = MultiSelect::new()
            .with_prompt("Select agents to install globally")
            .items(&available_agents)
            .defaults(&[true, true, true, false, false])
            .interact()?;
            
        // Install selected agents globally
        for &idx in &selections {
            let agent = available_agents[idx];
            AgentInstaller::install_global(agent, &target)?;
        }
        
        // Update global config
        let mut config = GlobalConfig::load()?;
        config.update_installed_agents(selections.iter().map(|&i| available_agents[i].to_string()).collect());
        GlobalConfig::save(&config)?;
        
        println!("{}", style("✓ Global agent installation complete!").green().bold());
        Ok(())
    }
}
```

#### 2. Global Agent Installer
**File**: `src/agents/installer.rs`
**Changes**: New file for global agent installation logic

```rust
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct AgentInstaller;

impl AgentInstaller {
    pub fn install_global(agent: &str, target_dir: &Path) -> Result<()> {
        match agent {
            "claude" => Self::install_claude_global(target_dir),
            "cursor" => Self::install_cursor_global(target_dir),
            "windsurf" => Self::install_windsurf_global(target_dir),
            "cline" => Self::install_cline_global(target_dir),
            "roo" => Self::install_roo_global(target_dir),
            _ => Err(crate::error::KayfabeError::Other(format!("Unknown agent: {}", agent)))
        }
    }
    
    fn install_claude_global(target_dir: &Path) -> Result<()> {
        // Install to ~/.claude/CLAUDE.md or local CLAUDE.md
        let global_path = dirs::home_dir()
            .ok_or_else(|| crate::error::KayfabeError::Other("Could not find home directory".to_string()))?
            .join(".claude")
            .join("CLAUDE.md");
            
        let local_path = target_dir.join("CLAUDE.md");
        
        // Create global config directory
        if let Some(parent) = global_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Generate universal Claude config
        let content = Self::generate_universal_claude_config()?;
        
        // Install globally and locally
        std::fs::write(&global_path, &content)?;
        std::fs::write(&local_path, &content)?;
        
        println!("✓ Claude agent installed globally and locally");
        Ok(())
    }
    
    fn install_cursor_global(target_dir: &Path) -> Result<()> {
        // Install to .cursorrules or .cursor/rules/kayfabe.mdc
        let local_path = target_dir.join(".cursorrules");
        let content = Self::generate_universal_cursor_config()?;
        std::fs::write(&local_path, &content)?;
        
        println!("✓ Cursor agent installed locally");
        Ok(())
    }
    
    // Similar methods for other agents...
    
    fn generate_universal_claude_config() -> Result<String> {
        Ok(r#"# Kayfabe AI Assistant Configuration

## About This Project
This project uses kayfabe for AI-assisted development with worktree management.

## Available Commands
- `kayfabe worktree create <name>` - Create new isolated worktree
- `kayfabe worktree list` - List all worktrees
- `kayfabe status` - Show repository status
- `kayfabe worktree cleanup` - Clean up stale worktrees

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
"#.to_string())
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] New install command compiles: `cargo build`
- [ ] Unit tests pass: `cargo test install`
- [ ] Integration tests pass: `cargo test --test install_integration`

#### Manual Verification:
- [ ] `kayfabe install` creates global agent configs
- [ ] Configs are created in correct locations (~/claude/, .cursorrules, etc.)
- [ ] Interactive selection works properly
- [ ] Global config is updated with installed agents

## Phase 2: Root Detection and Context Management

### Overview
Implement root detection so kayfabe commands work from any subdirectory within a kayfabe-managed repository.

### Changes Required:

#### 1. Root Detection Utility
**File**: `src/git/root.rs`
**Changes**: New file for kayfabe root detection

```rust
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct KayfabeRoot;

impl KayfabeRoot {
    /// Find the kayfabe root by looking for .kayfabe directory
    pub fn discover(start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path.canonicalize()?;
        
        loop {
            let kayfabe_dir = current.join(".kayfabe");
            if kayfabe_dir.exists() && kayfabe_dir.is_dir() {
                return Ok(current);
            }
            
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }
        
        Err(crate::error::KayfabeError::Other(
            "Not in a kayfabe-managed repository. Run 'kayfabe init' first.".to_string()
        ))
    }
    
    /// Check if we're in a kayfabe-managed repository
    pub fn is_kayfabe_repo(path: &Path) -> bool {
        Self::discover(path).is_ok()
    }
    
    /// Get the worktree directory relative to kayfabe root
    pub fn worktree_dir(root: &Path) -> PathBuf {
        root.join("wt")
    }
    
    /// Get the main directory relative to kayfabe root
    pub fn main_dir(root: &Path) -> PathBuf {
        root.join("main")
    }
}
```

#### 2. Update All Commands to Use Root Detection
**File**: `src/cli/worktree.rs`
**Changes**: Update worktree commands to work from any directory

```rust
// Add to existing WorktreeCommand implementation
use crate::git::KayfabeRoot;

impl WorktreeCommand {
    pub fn create(name: String, base: Option<String>, open: Option<String>, no_open: bool) -> Result<()> {
        // Find kayfabe root from current directory
        let current_dir = std::env::current_dir()?;
        let kayfabe_root = KayfabeRoot::discover(&current_dir)?;
        
        // Use kayfabe_root for all operations instead of current_dir
        let repo = GitRepo::open(&kayfabe_root)?;
        // ... rest of implementation
    }
    
    // Update other methods similarly
}
```

#### 3. Update Configuration Loading
**File**: `src/config/project.rs`
**Changes**: Load config from detected root

```rust
impl ProjectConfig {
    pub fn load_from_current_dir() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let kayfabe_root = crate::git::KayfabeRoot::discover(&current_dir)?;
        Self::load(&kayfabe_root)
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Root detection tests pass: `cargo test root_detection`
- [ ] Commands work from subdirectories: integration tests
- [ ] Configuration loading works from any location

#### Manual Verification:
- [ ] `kayfabe status` works from any subdirectory
- [ ] `kayfabe worktree create` works from any subdirectory
- [ ] Error messages are clear when not in kayfabe repo

## Phase 3: Remove Agents Command and Streamline CLI

### Overview
Remove the agents command and integrate its functionality into install/config commands.

### Changes Required:

#### 1. Update Main CLI Structure
**File**: `src/main.rs`
**Changes**: Remove agents command, add install command

```rust
// Remove from Commands enum:
// #[command(about = "Manage agent configurations")]
// Config { ... },

// Add to Commands enum:
#[command(about = "Install kayfabe agents globally")]
Install {
    #[arg(help = "Target directory (default: current directory)")]
    path: Option<PathBuf>,
    
    #[arg(long, help = "Non-interactive mode")]
    non_interactive: bool,
    
    #[arg(long, help = "Agents to install [claude|cursor|windsurf|cline|roo|all]")]
    agents: Option<Vec<String>>,
},

// Update match statement to handle Install command
Commands::Install { path, non_interactive, agents } => {
    InstCommand::execute(path, non_interactive, agents)
},
```

#### 2. Simplify Config Command
**File**: `src/cli/config.rs`
**Changes**: Remove generate functionality, focus on show/edit

```rust
// Remove ConfigCommands::Generate
// Keep only: Show, Edit, Validate, Init

impl ConfigCommand {
    // Remove generate method
    // Keep show, edit, validate, init methods
    
    pub fn show(agent: Option<String>) -> Result<()> {
        // Update to work with global configs
        let current_dir = std::env::current_dir()?;
        let kayfabe_root = crate::git::KayfabeRoot::discover(&current_dir)?;
        
        // Show both global and local configs
        // ... implementation
    }
}
```

#### 3. Update CLI Module Exports
**File**: `src/cli/mod.rs`
**Changes**: Add install command export

```rust
pub mod install;
// ... existing modules

pub use install::InstallCommand;
// ... existing exports
```

### Success Criteria:

#### Automated Verification:
- [ ] CLI compiles without agents command: `cargo build`
- [ ] Help text shows install command: `cargo run -- --help`
- [ ] Install command works: `cargo run -- install --help`

#### Manual Verification:
- [ ] `kayfabe agents` command no longer exists
- [ ] `kayfabe install` command works as expected
- [ ] `kayfabe config generate` is removed
- [ ] All other commands still function

## Phase 4: Universal Agent Templates

### Overview
Create universal, reusable agent templates that work across all projects.

### Changes Required:

#### 1. Universal Agent Templates
**File**: `src/agents/templates/`
**Changes**: Create template directory with universal configs

```rust
// src/agents/templates/claude.md
pub const CLAUDE_TEMPLATE: &str = r#"
# Kayfabe AI Development Assistant

## Project Context
This project uses kayfabe for AI-assisted development with git worktree management.

## Available Kayfabe Commands
- `kayfabe worktree create <name> [--open <ide>]` - Create isolated development environment
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
- Use `--open cursor` or `--open windsurf` to launch appropriate IDE
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
```

#### 2. Template-Based Agent Generation
**File**: `src/agents/universal.rs`
**Changes**: New universal agent generator

```rust
use crate::error::Result;

pub struct UniversalAgentGenerator;

impl UniversalAgentGenerator {
    pub fn generate_claude() -> Result<String> {
        Ok(crate::agents::templates::CLAUDE_TEMPLATE.to_string())
    }
    
    pub fn generate_cursor() -> Result<String> {
        Ok(crate::agents::templates::CURSOR_TEMPLATE.to_string())
    }
    
    pub fn generate_windsurf() -> Result<String> {
        Ok(crate::agents::templates::WINDSURF_TEMPLATE.to_string())
    }
    
    // Add project-specific context if needed
    pub fn generate_with_context(agent: &str, project_context: Option<&ProjectContext>) -> Result<String> {
        let base_template = match agent {
            "claude" => Self::generate_claude()?,
            "cursor" => Self::generate_cursor()?,
            "windsurf" => Self::generate_windsurf()?,
            _ => return Err(crate::error::KayfabeError::Other(format!("Unknown agent: {}", agent)))
        };
        
        // Optionally append project-specific context
        if let Some(context) = project_context {
            Ok(format!("{}\n\n## Project-Specific Context\n- Type: {}\n- Build: {:?}\n- Test: {:?}", 
                base_template, context.project_type, context.build_cmd, context.test_cmd))
        } else {
            Ok(base_template)
        }
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Template generation tests pass: `cargo test templates`
- [ ] Universal configs are valid and complete
- [ ] Template system is extensible

#### Manual Verification:
- [ ] Generated configs work across different project types
- [ ] Configs provide useful kayfabe-specific guidance
- [ ] Templates are comprehensive and actionable

## Phase 5: Integration and Testing

### Overview
Integrate all changes, update documentation, and ensure everything works together.

### Changes Required:

#### 1. Update Documentation
**File**: `README.md`
**Changes**: Update to reflect new global installation approach

```markdown
## Quick Start

### Installation & Setup

**Install kayfabe:**
```bash
cargo install --git https://github.com/ShreyeshArangath/kayfabe.git
```

**Set up global agents:**
```bash
kayfabe install
```

**Initialize a repository:**
```bash
kayfabe init ~/projects/my-repo
```

**Create and work in worktrees:**
```bash
kayfabe worktree create feature-auth --open cursor
```

### Global Agent Installation

Kayfabe installs AI agent configurations globally, similar to vibe-tools:

- **Claude Code**: `~/.claude/CLAUDE.md` and local `CLAUDE.md`
- **Cursor**: `.cursorrules` in project root
- **Windsurf**: `.windsurfrules` in project root
- **Cline/Roo**: `.clinerules/kayfabe.md`

Run `kayfabe install` once to set up agents globally. They'll work across all your kayfabe-managed projects.
```

#### 2. Update Tests
**File**: `tests/integration/`
**Changes**: Add comprehensive integration tests

```rust
#[test]
fn test_global_installation() {
    // Test kayfabe install command
    // Verify global configs are created
    // Test from different directories
}

#[test] 
fn test_root_detection() {
    // Test commands work from subdirectories
    // Test error handling when not in kayfabe repo
}

#[test]
fn test_universal_agents() {
    // Test agent configs work across project types
    // Verify template quality and completeness
}
```

### Success Criteria:

#### Automated Verification:
- [ ] All unit tests pass: `cargo test`
- [ ] Integration tests pass: `cargo test --test integration`
- [ ] Documentation builds: `cargo doc`
- [ ] Clippy passes: `cargo clippy`

#### Manual Verification:
- [ ] Complete workflow works end-to-end
- [ ] Commands work from any directory in kayfabe repo
- [ ] Global agent installation works as expected
- [ ] Agent configs provide value across different projects
- [ ] No regression in existing worktree functionality

## Implementation Notes

### Migration Strategy
1. **Backward Compatibility**: Existing `.kayfabe` projects continue to work
2. **Gradual Migration**: Users can run `kayfabe install` on existing projects
3. **Clear Messaging**: Update help text and docs to guide users

### Risk Mitigation
- **Preserve Core Functionality**: Don't break existing worktree management
- **Comprehensive Testing**: Test all command combinations
- **User Communication**: Clear documentation of changes

### Performance Considerations
- **Root Detection**: Cache root discovery results
- **Global Config**: Minimize file system operations
- **Template Generation**: Pre-compile templates where possible

## Verification Plan

### End-to-End Test Scenario
1. Fresh install: `cargo install kayfabe`
2. Global setup: `kayfabe install` 
3. Repository init: `kayfabe init ~/test-project`
4. Worktree creation: `kayfabe worktree create feature-test --open cursor`
5. Directory navigation: `cd ~/test-project/wt/feature-test/src && kayfabe status`
6. Cleanup: `kayfabe worktree cleanup --dry-run`

### Success Metrics
- All commands work from any subdirectory
- Global agent configs are reusable across projects
- No more per-project config generation needed
- Agents command is successfully removed
- Installation process is smooth and intuitive
