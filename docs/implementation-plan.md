# Kayfabe Implementation Plan

**Date**: 2026-01-23
**Project**: Kayfabe - AI-Enhanced CLI Task Manager
**Language**: Rust
**Timeline**: 19-24 days for MVP (revised based on complexity analysis)

## Overview

Kayfabe is a CLI task manager that accelerates engineer productivity with AI assistants like Claude by combining task management with git worktree orchestration.

### Core Features (MVP)
- Initialize projects from git repositories
- CRUD task operations with SQLite persistence
- Isolated git worktrees per task
- TUI for task visualization
- Tmux integration for task execution
- Automated stale worktree cleanup
- Claude thought/artifact tracking

### Key Insights from vibe-kanban Research

**Database Evolution** (73 migrations):
- Schema will evolve - plan for migrations from day 1
- Use schema versioning table to track applied migrations
- Iterative approach is normal and expected
- SQLite provides simple, reliable persistence

**Process Execution**:
- Capturing stdout/stderr is essential for debugging
- Store exit codes to understand failures
- Process lifecycle tracking helps troubleshoot issues
- Stdout/stderr capture needs smart buffering (memory limits)

**Performance Considerations**:
- Composite indexes critical for query performance at scale
- Connection pooling for async operations
- Repository-level vs project-level separation improves multi-worktree coordination
- Worktree-based isolation works well for parallel AI tasks

**Cleanup Strategy**:
- Configurable thresholds needed (7d/14d/30d)
- Orphan detection prevents filesystem/DB inconsistency
- last_accessed_at tracking essential for stale detection
- Dry-run mode prevents accidental deletions

**Technology Choices**:
- Rust backend provides speed, reliability, excellent CLI ecosystem
- TUI improves UX significantly over basic CLI
- Tmux provides persistent sessions ideal for dev workflows
- git2-rs + git CLI hybrid approach for worktree management

## Architecture

```
kayfabe/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # Commands (init, add, list, etc.)
│   ├── core/                # Business logic
│   │   ├── task.rs
│   │   ├── worktree.rs
│   │   ├── project.rs
│   │   └── cleaner.rs
│   ├── db/                  # SQLite layer
│   │   ├── models.rs
│   │   └── migrations.rs
│   ├── tui/                 # Terminal UI (ratatui)
│   └── utils/               # Helpers (tmux, paths)
├── templates/               # Config templates
├── Cargo.toml
└── README.md
```

### Tech Stack
- `clap` - CLI parsing
- `rusqlite` - Database
- `ratatui` + `crossterm` - TUI
- `git2` - Git operations
- `serde` + `chrono` - Data handling
- `anyhow` - Error handling
- `tokio` - Async runtime for process management

## Process Execution & Monitoring

### Architecture

When a user executes `kayfabe execute <task>`, the following flow occurs:

1. **Tmux Session Creation**
   - Create or attach to tmux session named `kayfabe-<task-name>`
   - Set working directory to task worktree
   - Launch Claude process in the tmux window

2. **Process Spawning**
   - Use `tokio::process::Command` to launch Claude
   - Redirect stdout and stderr to pipes for capture
   - Store process ID and tmux session info in database

3. **Output Capture**
   - Stream stdout/stderr in real-time to both:
     - Terminal (via tmux)
     - Database (`execution_processes` table)
   - Buffer output intelligently to avoid memory issues
   - Handle line-buffering for live updates

4. **Status Tracking**
   - Update task status to "active" on launch
   - Monitor process exit codes
   - Update execution record on completion/failure
   - Handle SIGINT/SIGTERM for graceful shutdown

### Implementation Details

**Executor Module** (`src/core/executor.rs`):
```rust
pub struct Executor {
    task: Task,
    tmux_manager: TmuxManager,
    db: Database,
}

impl Executor {
    pub async fn execute(&mut self, command: &str) -> Result<ExecutionProcess> {
        // 1. Create tmux session
        let session = self.tmux_manager.create_or_attach(&self.task).await?;

        // 2. Spawn process with output capture
        let mut child = Command::new(command)
            .current_dir(&self.task.worktree_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // 3. Create execution record
        let exec = ExecutionProcess::new(&self.task, command, &session);
        self.db.insert_execution(&exec)?;

        // 4. Capture output in background task
        let exec_id = exec.id.clone();
        tokio::spawn(async move {
            capture_output(child, exec_id, db.clone()).await;
        });

        // 5. Update task status
        self.task.set_status(TaskStatus::Active)?;

        Ok(exec)
    }
}
```

**Process Monitor** (`src/core/process_monitor.rs`):
```rust
async fn capture_output(
    mut child: Child,
    exec_id: String,
    db: Database,
) -> Result<()> {
    let stdout = child.stdout.take().context("No stdout")?;
    let stderr = child.stderr.take().context("No stderr")?;

    // Stream outputs concurrently
    let (stdout_result, stderr_result) = tokio::join!(
        capture_stream(stdout, exec_id.clone(), db.clone(), StreamType::Stdout),
        capture_stream(stderr, exec_id.clone(), db.clone(), StreamType::Stderr),
    );

    // Wait for process completion
    let status = child.wait().await?;

    // Update execution record with exit code
    db.update_execution_completion(&exec_id, status.code())?;

    Ok(())
}
```

**Tmux Manager** (`src/utils/tmux.rs`):
```rust
pub struct TmuxManager;

impl TmuxManager {
    pub async fn create_or_attach(&self, task: &Task) -> Result<String> {
        let session_name = format!("kayfabe-{}", task.name);

        // Check if session exists
        if self.session_exists(&session_name)? {
            return Ok(session_name);
        }

        // Create new session
        Command::new("tmux")
            .args(&[
                "new-session",
                "-d",  // Detached
                "-s", &session_name,
                "-c", &task.worktree_path,  // Working directory
            ])
            .status()
            .await?;

        Ok(session_name)
    }

    pub fn attach(&self, session_name: &str) -> Result<()> {
        Command::new("tmux")
            .args(&["attach-session", "-t", session_name])
            .status()?;
        Ok(())
    }

    pub fn kill_session(&self, session_name: &str) -> Result<()> {
        Command::new("tmux")
            .args(&["kill-session", "-t", session_name])
            .status()?;
        Ok(())
    }
}
```

### Process Lifecycle

```
User runs: kayfabe execute <task>
    ↓
Create/attach tmux session
    ↓
Launch Claude process
    ↓
Capture stdout/stderr → Database
    ↓
Update task status → "active"
    ↓
[Process runs...]
    ↓
Process completes/fails
    ↓
Record exit code → Database
    ↓
Update task status → "completed"/"archived"
```

### Additional Commands

- `kayfabe attach <task>` - Attach to running tmux session
- `kayfabe kill <task>` - Kill running process and tmux session
- `kayfabe logs <task>` - Show stdout/stderr from last execution

### Error Handling

- **Process spawn failure**: Log error, keep task as "pending"
- **Tmux unavailable**: Graceful error with installation instructions
- **Output capture failure**: Continue execution, log warning
- **Process crash**: Record in database with stderr, mark task as failed

## Implementation Phases

### Phase 1: Foundation (2-3 days)
**Goal**: Project structure, dependencies, database schema with migrations

**Key Tasks**:
- Set up Cargo project with all dependencies
- Design complete SQLite schema (5 core tables + indexes)
- Implement migration system with versioning
- Create data models (Project, Task, Worktree, Activity, ExecutionProcess)
- Build CLI skeleton with clap
- Set up error handling patterns with anyhow

**Files Created**:
- `Cargo.toml` - All dependencies including tokio, rusqlite, clap, ratatui
- `src/main.rs` - CLI entry point with async main
- `src/db/mod.rs` - Database connection management
- `src/db/models.rs` - Struct definitions for all entities
- `src/db/migrations.rs` - Migration runner
- `src/db/migrations/001_initial_schema.sql` - Core tables
- `src/db/migrations/002_add_execution_processes.sql` - Process monitoring
- `src/db/migrations/003_add_composite_indexes.sql` - Performance indexes
- `src/utils/paths.rs` - Global ~/.kayfabe directory management
- `src/errors.rs` - Custom error types and contexts

**Implementation Details**:
```rust
// src/db/models.rs - Core data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_path: PathBuf,
    pub worktree_dir: PathBuf,
    pub git_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub git_branch: Option<String>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub cleanup_eligible: bool,
    // ... other fields
}
```

**Success Criteria**:
- [ ] `cargo build` succeeds with no warnings
- [ ] Database initializes with complete schema
- [ ] Migrations run successfully from scratch
- [ ] Schema versioning tracks applied migrations
- [ ] `kayfabe --help` shows all planned commands
- [ ] Can connect to ~/.kayfabe/kayfabe.db
- [ ] All data models serialize/deserialize correctly

**Testing**:
- Unit tests for migration runner
- Verify schema_migrations table created
- Test database initialization multiple times
- Validate all foreign key constraints work

### Phase 2: Project Init (2-3 days)
**Goal**: Implement `kayfabe init <name> <git_url>` with validation

**Key Tasks**:
- Implement git URL validation (SSH, HTTPS formats)
- Clone bare git repository using git2-rs
- Initialize worktree base directory structure
- Create config files from templates (kayfabe.md, claude.md)
- kayfabe.md should have instructions on how to use kayfabe within a kayfabe project
- Register project in global database with transaction
- Create thoughts/ directory with .gitkeep
- Validate repository has commits and default branch
- Handle errors gracefully (network, auth, permissions)

**Files Created**:
- `src/core/project.rs` - Project initialization logic
- `src/cli/init.rs` - CLI command handler
- `templates/kayfabe.md` - Project documentation template
- `templates/claude.md` - Claude instructions template
- `src/utils/git.rs` - Git helper functions

**Implementation Details**:
```rust
// src/core/project.rs
impl Project {
    pub async fn init(name: &str, git_url: &str) -> Result<Self> {
        // 1. Validate git URL format
        validate_git_url(git_url)?;

        // 2. Create project directory
        let project_path = PathBuf::from(name);
        fs::create_dir_all(&project_path)?;

        // 3. Clone bare repository
        let repo_path = project_path.join(".git");
        Repository::clone_bare(git_url, &repo_path)
            .context("Failed to clone repository")?;

        // 4. Detect default branch
        let repo = Repository::open(&repo_path)?;
        let default_branch = detect_default_branch(&repo)?;

        // 5. Create directory structure
        let worktree_dir = project_path.join(&format!("{}-wt", name));
        fs::create_dir_all(&worktree_dir)?;
        fs::create_dir_all(project_path.join("thoughts"))?;

        // 6. Copy templates
        create_config_files(&project_path)?;

        // 7. Register in database
        let project = Project::new(name, git_url, repo_path, worktree_dir);
        Database::insert_project(&project)?;

        Ok(project)
    }
}
```

**Directory Structure Created**:
```
myproject/
├── .git/                    # Bare repository
├── myproject-wt/            # Worktree base (empty initially)
├── thoughts/                # AI artifacts
│   └── .gitkeep
├── .kayfabe/                # Local state
│   └── config.toml
├── kayfabe.md               # Project documentation
└── claude.md                # Claude instructions
```

**Success Criteria**:
- [ ] Can initialize from SSH URLs (git@github.com:...)
- [ ] Can initialize from HTTPS URLs (https://github.com/...)
- [ ] Bare repo cloned correctly (verify with git commands)
- [ ] Default branch detected (main/master)
- [ ] All directories created with correct permissions
- [ ] Config files copied from templates
- [ ] Project registered in ~/.kayfabe/kayfabe.db
- [ ] Graceful error on invalid git URL
- [ ] Graceful error on network failure
- [ ] Graceful error if directory exists

**Testing**:
- Test with public and private repositories
- Test with both SSH and HTTPS URLs
- Test error handling for invalid URLs
- Test error handling for network issues
- Verify bare repository structure
- Test with repositories using main vs master branch

### Phase 3: Task CRUD & Worktree Management (4-5 days)
**Goal**: Implement add, list, remove commands with robust worktree handling

**Key Tasks**:
- Implement task creation with atomic worktree setup
- Build git worktree manager (create, remove, list, verify)
- Define branch naming strategy (kayfabe/<task-name>)
- Create task database operations (CRUD with transactions)
- Implement basic CLI list view with status
- Add task removal with comprehensive cleanup
- Create thoughts/<task> directories with structure
- Add worktree validation and repair utilities
- Implement last_accessed_at tracking

**Files Created**:
- `src/core/task.rs` - Task lifecycle management
- `src/core/worktree.rs` - Git worktree operations
- `src/cli/add.rs` - Task creation command
- `src/cli/remove.rs` - Task removal command
- `src/cli/list.rs` - Task listing command
- `src/utils/git.rs` - Git helper utilities

**Implementation Details**:

**Branch Naming Convention**:
- Format: `kayfabe/<task-name>`
- Sanitize task names: replace spaces with dashes, lowercase
- Example: "Feature X" → `kayfabe/feature-x`
- Ensures no conflicts with user branches

**Task Creation Flow**:
```rust
// src/core/task.rs
impl Task {
    pub async fn create(
        project: &Project,
        name: &str,
        description: Option<&str>,
    ) -> Result<Self> {
        // 1. Validate task name doesn't exist
        if Database::task_exists(project.id, name)? {
            bail!("Task '{}' already exists", name);
        }

        // 2. Create task record (transaction start)
        let task = Task::new(project.id, name, description);
        Database::insert_task(&task)?;

        // 3. Create git worktree
        let worktree = Worktree::create(&project, &task).await?;

        // 4. Update task with worktree path
        task.worktree_path = Some(worktree.path.clone());
        Database::update_task(&task)?;

        // 5. Create thoughts directory
        let thoughts_path = project.path.join("thoughts").join(&task.name);
        fs::create_dir_all(&thoughts_path)?;

        // 6. Create initial activity record
        Activity::log(&task, ActivityType::Created, "Task created")?;

        Ok(task)
    }
}
```

**Worktree Manager**:
```rust
// src/core/worktree.rs
impl Worktree {
    pub async fn create(project: &Project, task: &Task) -> Result<Self> {
        let branch_name = format!("kayfabe/{}", sanitize_name(&task.name));
        let worktree_path = project.worktree_dir.join(&task.name);

        // Open bare repository
        let repo = Repository::open(&project.repo_path)?;

        // Create branch from default branch
        let default_branch = detect_default_branch(&repo)?;
        let head = repo.find_reference(&format!("refs/heads/{}", default_branch))?;
        let commit = head.peel_to_commit()?;

        // Create new branch
        repo.branch(&branch_name, &commit, false)
            .context("Failed to create branch")?;

        // Add worktree using git command (git2-rs doesn't support this directly)
        Command::new("git")
            .args(&[
                "-C", project.repo_path.to_str().unwrap(),
                "worktree", "add",
                worktree_path.to_str().unwrap(),
                &branch_name,
            ])
            .status()
            .await?;

        // Verify worktree created
        if !worktree_path.exists() {
            bail!("Worktree creation failed");
        }

        // Create database record
        let worktree = Worktree {
            id: Uuid::new_v4().to_string(),
            task_id: task.id.clone(),
            path: worktree_path,
            branch_name,
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
        };

        Database::insert_worktree(&worktree)?;

        Ok(worktree)
    }

    pub async fn remove(&self, project: &Project) -> Result<()> {
        // Remove worktree using git command
        Command::new("git")
            .args(&[
                "-C", project.repo_path.to_str().unwrap(),
                "worktree", "remove",
                self.path.to_str().unwrap(),
                "--force",  // Remove even with uncommitted changes
            ])
            .status()
            .await?;

        // Remove database record
        Database::delete_worktree(&self.id)?;

        Ok(())
    }
}
```

**Success Criteria**:
- [ ] `kayfabe add <name>` creates task + worktree atomically
- [ ] Worktrees created in correct location (project-wt/<task>)
- [ ] Git branches follow naming convention (kayfabe/<task-name>)
- [ ] Worktrees are fully isolated (changes don't affect others)
- [ ] `kayfabe list` shows all tasks with status
- [ ] `kayfabe list` shows last accessed time
- [ ] `kayfabe remove <name>` removes task + worktree + branch
- [ ] Thoughts directories created (thoughts/<task>/)
- [ ] Database records match filesystem state
- [ ] Graceful error handling for git failures
- [ ] Can create multiple tasks concurrently

**Testing**:
- Create 10 tasks, verify all worktrees created
- Test worktree isolation (modify files in one worktree)
- Test task removal, verify complete cleanup
- Test with special characters in task names
- Verify orphan worktree detection works
- Test concurrent task creation
- Verify last_accessed_at updates correctly
- Test git branch naming with various task names

### Phase 4: TUI with State Management (3-4 days)
**Goal**: Beautiful, responsive terminal UI for task management

**Key Tasks**:
- Build TUI application state machine with clean architecture
- Design multi-panel UI layout (header, task list, details, footer)
- Implement keyboard navigation with vim-style bindings
- Add task filtering by status (pending/active/completed/all)
- Handle task selection and inline actions
- Style with colors, borders, and status indicators
- Implement async refresh without blocking UI
- Add thought count display per task
- Show git branch and worktree info

**Files Created**:
- `src/tui/app.rs` - Application state machine
- `src/tui/ui.rs` - UI rendering logic
- `src/tui/events.rs` - Event handling (keyboard, mouse)
- `src/tui/state.rs` - UI state management
- `src/tui/widgets/` - Custom widgets (task list, detail panel)
- Updated `src/cli/list.rs` - Launch TUI mode

**Implementation Details**:

**Application State**:
```rust
// src/tui/state.rs
pub struct AppState {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub sort_by: SortBy,
    pub show_help: bool,
}

pub enum TaskFilter {
    All,
    Pending,
    Active,
    Completed,
    Archived,
}

pub enum SortBy {
    CreatedAt,
    UpdatedAt,
    Name,
    Status,
}
```

**UI Layout**:
```
┌─ Kayfabe - Project: myproject ────────────────────────────┐
│ Filter: [All] [Pending] [Active] [Completed]              │
├────────────────────────────────────────────────────────────┤
│ Tasks (5)                  │ Details                       │
│                            │                               │
│ ✓ feature-x [completed]    │ Name: feature-x               │
│ ▶ feature-y [active]       │ Status: Active                │
│ ○ feature-z [pending]      │ Branch: kayfabe/feature-y     │
│                            │ Created: 2 days ago           │
│                            │ Updated: 5 minutes ago        │
│                            │ Thoughts: 12 files            │
│                            │                               │
│                            │ Description:                  │
│                            │ Build new feature Y           │
│                            │                               │
├────────────────────────────┴───────────────────────────────┤
│ ↑↓:Navigate  Enter:Execute  r:Remove  f:Filter  q:Quit    │
└────────────────────────────────────────────────────────────┘
```

**Keyboard Shortcuts**:
- `↑/k` - Move up
- `↓/j` - Move down
- `Enter` - Execute selected task
- `r` - Remove selected task (with confirmation)
- `f` - Toggle filter menu
- `s` - Change sort order
- `t` - Open thoughts directory
- `h` - Show help overlay
- `q/Esc` - Quit

**Event Loop**:
```rust
// src/tui/app.rs
pub async fn run_tui(project: &Project) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = AppState::new(project).await?;

    loop {
        // Render UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
                    KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                    KeyCode::Enter => app.execute_selected().await?,
                    KeyCode::Char('r') => app.remove_selected().await?,
                    KeyCode::Char('f') => app.toggle_filter(),
                    // ... more bindings
                    _ => {}
                }
            }
        }

        // Async refresh (check for new thoughts, status changes)
        if app.should_refresh() {
            app.refresh().await?;
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}
```

**Styling**:
```rust
// Status indicators with colors
match task.status {
    TaskStatus::Pending => "○".fg(Color::Gray),
    TaskStatus::Active => "▶".fg(Color::Green),
    TaskStatus::Completed => "✓".fg(Color::Blue),
    TaskStatus::Archived => "⊗".fg(Color::DarkGray),
}
```

**Success Criteria**:
- [ ] TUI launches cleanly from `kayfabe list`
- [ ] Can navigate tasks with both arrow keys and vim bindings
- [ ] Task details update in real-time when selection changes
- [ ] Filters work correctly (all/pending/active/completed)
- [ ] Keyboard shortcuts all functional
- [ ] Responsive UI (60fps, <16ms render time)
- [ ] Proper terminal cleanup on exit
- [ ] Shows thought count per task
- [ ] Shows git branch and last update time
- [ ] Confirmation prompt for destructive actions
- [ ] Help overlay accessible

**Testing**:
- Test with 0 tasks (empty state)
- Test with 100+ tasks (performance)
- Test all keyboard shortcuts
- Test filter transitions
- Test terminal resize handling
- Test with long task names/descriptions
- Verify proper cleanup on Ctrl+C
- Test color rendering in different terminals

### Phase 5: Tmux Integration & Process Monitoring (3 days)
**Goal**: Execute tasks in tmux with Claude and capture outputs

**Key Tasks**:
- Build tmux manager (create/attach/kill sessions)
- Implement process spawning with stdout/stderr capture
- Launch Claude in tmux with proper environment
- Update task status to "active" on execution
- Capture and store process outputs in database
- Handle session attachment and detachment
- Implement process health monitoring
- Add process kill functionality
- Create logs viewing command
- Handle graceful shutdown on SIGINT/SIGTERM

**Files Created**:
- `src/utils/tmux.rs` - Tmux session management
- `src/core/executor.rs` - Process execution logic
- `src/core/process_monitor.rs` - Output capture and monitoring
- `src/cli/execute.rs` - Execute command
- `src/cli/attach.rs` - Attach to running session
- `src/cli/kill.rs` - Kill running process
- `src/cli/logs.rs` - View execution logs

**Implementation Details**:

**Execute Command Flow**:
1. Validate task exists and is not already running
2. Create or attach to tmux session `kayfabe-<task-name>`
3. Set working directory to task worktree
4. Spawn Claude process with `tokio::process::Command`
5. Redirect stdout/stderr to pipes for capture
6. Create `execution_processes` record in database
7. Update task status to "active"
8. Launch background task to capture output streams
9. Return control to user (process runs in background)

**Executor Implementation**:
```rust
// src/core/executor.rs
pub struct Executor {
    task: Task,
    project: Project,
    db: Database,
}

impl Executor {
    pub async fn execute(&mut self) -> Result<()> {
        // Check if already running
        if let Some(exec) = self.db.get_active_execution(&self.task.id)? {
            bail!("Task already running in session: {}", exec.tmux_session);
        }

        // Create/attach tmux session
        let tmux = TmuxManager::new();
        let session = tmux.create_session(&self.task).await?;

        // Build Claude command
        let command = self.build_claude_command()?;

        // Spawn process with output capture
        let mut child = Command::new("claude")
            .current_dir(&self.task.worktree_path.as_ref().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn Claude")?;

        // Create execution record
        let exec = ExecutionProcess {
            id: Uuid::new_v4().to_string(),
            task_id: self.task.id.clone(),
            command: command.clone(),
            status: ProcessStatus::Running,
            tmux_session: session.clone(),
            started_at: Utc::now(),
            ..Default::default()
        };

        self.db.insert_execution(&exec)?;

        // Update task status
        self.task.status = TaskStatus::Active;
        self.task.last_accessed_at = Some(Utc::now());
        self.db.update_task(&self.task)?;

        // Spawn output capture in background
        let exec_id = exec.id.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Err(e) = capture_process_output(child, exec_id, db).await {
                eprintln!("Error capturing output: {}", e);
            }
        });

        println!("✓ Task '{}' executing in session: {}", self.task.name, session);
        println!("  Attach with: tmux attach -t {}", session);
        println!("  Or use: kayfabe attach {}", self.task.name);

        Ok(())
    }

    fn build_claude_command(&self) -> Result<String> {
        // Read claude.md for custom instructions
        let claude_md = self.project.path.join("claude.md");
        let instructions = if claude_md.exists() {
            fs::read_to_string(&claude_md)?
        } else {
            String::new()
        };

        Ok(format!("claude --context {}", self.task.worktree_path.display()))
    }
}
```

**Process Monitor**:
```rust
// src/core/process_monitor.rs
async fn capture_process_output(
    mut child: Child,
    exec_id: String,
    db: Database,
) -> Result<()> {
    let stdout = child.stdout.take().context("No stdout")?;
    let stderr = child.stderr.take().context("No stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut stdout_buffer = Vec::new();
    let mut stderr_buffer = Vec::new();

    // Read output streams concurrently
    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line? {
                    Some(l) => stdout_buffer.push(l),
                    None => break,
                }
            }
            line = stderr_reader.next_line() => {
                match line? {
                    Some(l) => stderr_buffer.push(l),
                    None => break,
                }
            }
        }
    }

    // Wait for process completion
    let status = child.wait().await?;

    // Update execution record
    db.update_execution_complete(
        &exec_id,
        status.code(),
        &stdout_buffer.join("\n"),
        &stderr_buffer.join("\n"),
    )?;

    // Update task status based on exit code
    let task_status = if status.success() {
        TaskStatus::Completed
    } else {
        TaskStatus::Archived  // Failed
    };

    db.update_task_status_by_execution(&exec_id, task_status)?;

    Ok(())
}
```

**Additional Commands**:

**Attach**: `kayfabe attach <task>`
```rust
// src/cli/attach.rs
pub async fn attach(task_name: &str) -> Result<()> {
    let task = Database::get_task_by_name(task_name)?;
    let session = format!("kayfabe-{}", task.name);

    TmuxManager::new().attach(&session)?;
    Ok(())
}
```

**Kill**: `kayfabe kill <task>`
```rust
// src/cli/kill.rs
pub async fn kill(task_name: &str) -> Result<()> {
    let task = Database::get_task_by_name(task_name)?;

    if let Some(exec) = Database::get_active_execution(&task.id)? {
        // Kill tmux session (which kills the process)
        TmuxManager::new().kill_session(&exec.tmux_session)?;

        // Update execution record
        Database::update_execution_killed(&exec.id)?;

        // Update task status
        task.status = TaskStatus::Pending;
        Database::update_task(&task)?;

        println!("✓ Killed task '{}'", task.name);
    } else {
        println!("Task '{}' is not running", task.name);
    }

    Ok(())
}
```

**Logs**: `kayfabe logs <task>`
```rust
// src/cli/logs.rs
pub async fn logs(task_name: &str, follow: bool) -> Result<()> {
    let task = Database::get_task_by_name(task_name)?;
    let exec = Database::get_latest_execution(&task.id)?
        .context("No execution found for task")?;

    println!("=== Task: {} ===", task.name);
    println!("Command: {}", exec.command);
    println!("Status: {:?}", exec.status);
    println!("Started: {}", exec.started_at);
    if let Some(completed) = exec.completed_at {
        println!("Completed: {}", completed);
    }
    println!("\n--- STDOUT ---");
    println!("{}", exec.stdout.unwrap_or_default());

    if !exec.stderr.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        println!("\n--- STDERR ---");
        println!("{}", exec.stderr.unwrap_or_default());
    }

    if let Some(code) = exec.exit_code {
        println!("\nExit code: {}", code);
    }

    Ok(())
}
```

**Success Criteria**:
- [ ] `kayfabe execute <task>` creates tmux session
- [ ] Window starts in correct worktree directory
- [ ] Claude launches with proper context
- [ ] Task status updates to "active"
- [ ] Stdout/stderr captured and stored in database
- [ ] `kayfabe attach <task>` attaches to running session
- [ ] `kayfabe kill <task>` terminates process cleanly
- [ ] `kayfabe logs <task>` displays execution history
- [ ] Process exit codes recorded correctly
- [ ] Graceful handling of process crashes
- [ ] Can execute multiple tasks concurrently
- [ ] Proper cleanup on SIGINT/SIGTERM

**Testing**:
- Execute task and verify tmux session created
- Verify working directory is worktree path
- Attach to running session and verify functionality
- Kill running task and verify cleanup
- View logs and verify stdout/stderr captured
- Test with Claude not installed (graceful error)
- Test with tmux not available (graceful error)
- Execute multiple tasks, verify isolation
- Test process crash handling
- Verify exit codes recorded correctly

### Phase 6: Enhanced Cleanup & Maintenance (2 days)
**Goal**: Comprehensive worktree cleanup with safety features

**Key Tasks**:
- Implement configurable stale detection (7d/14d/30d thresholds)
- Build orphan worktree detector (filesystem vs database)
- Create cleanup command with dry-run mode
- Add confirmation prompts with detailed preview
- Implement uncommitted changes detection
- Check for active tmux sessions before cleanup
- Update database atomically on cleanup
- Add database consistency verification
- Implement backup option before deletion
- Track cleanup history in activities

**Files Created**:
- `src/core/cleaner.rs` - Enhanced cleanup logic
- `src/cli/clean.rs` - Cleanup command with multiple modes
- `src/core/consistency.rs` - Database/filesystem consistency checks

**Implementation Details**:

**Cleanup Modes**:

1. **Stale Worktree Cleanup**:
   ```bash
   kayfabe clean --stale 7d          # Delete worktrees older than 7 days
   kayfabe clean --stale 14d --dry-run  # Preview without deleting
   ```

2. **Orphan Worktree Cleanup**:
   ```bash
   kayfabe clean --orphans           # Clean worktrees without DB records
   kayfabe clean --orphans --dry-run # Preview orphans
   ```

3. **Force Mode** (skip confirmation):
   ```bash
   kayfabe clean --stale 7d --force  # No confirmation prompt
   ```

4. **Combined Cleanup**:
   ```bash
   kayfabe clean --all               # Both stale and orphans
   ```

**Cleaner Implementation**:
```rust
// src/core/cleaner.rs
pub struct Cleaner {
    project: Project,
    db: Database,
}

#[derive(Debug)]
pub struct CleanupTarget {
    task_name: String,
    worktree_path: PathBuf,
    last_accessed: DateTime<Utc>,
    age_days: i64,
    has_uncommitted: bool,
    has_active_session: bool,
    is_orphan: bool,
}

impl Cleaner {
    pub async fn find_stale_worktrees(
        &self,
        threshold_days: i64,
    ) -> Result<Vec<CleanupTarget>> {
        let threshold = Utc::now() - Duration::days(threshold_days);
        let mut targets = Vec::new();

        // Query tasks with old last_accessed_at
        let stale_tasks = self.db.get_tasks_older_than(&threshold)?;

        for task in stale_tasks {
            let worktree = self.db.get_worktree_by_task(&task.id)?;

            // Check for uncommitted changes
            let has_uncommitted = self.check_uncommitted(&worktree.path)?;

            // Check for active tmux session
            let session_name = format!("kayfabe-{}", task.name);
            let has_active_session = TmuxManager::new()
                .session_exists(&session_name)?;

            let age = Utc::now() - task.last_accessed_at.unwrap_or(task.created_at);

            targets.push(CleanupTarget {
                task_name: task.name,
                worktree_path: worktree.path,
                last_accessed: task.last_accessed_at.unwrap_or(task.created_at),
                age_days: age.num_days(),
                has_uncommitted,
                has_active_session,
                is_orphan: false,
            });
        }

        Ok(targets)
    }

    pub async fn find_orphan_worktrees(&self) -> Result<Vec<CleanupTarget>> {
        let mut orphans = Vec::new();

        // Scan worktree directory
        let worktree_dir = &self.project.worktree_dir;
        for entry in fs::read_dir(worktree_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let name = path.file_name().unwrap().to_str().unwrap();

            // Check if task exists in database
            if self.db.get_task_by_name(name).is_err() {
                // This is an orphan
                let metadata = fs::metadata(&path)?;
                let modified = metadata.modified()?;
                let last_accessed: DateTime<Utc> = modified.into();

                orphans.push(CleanupTarget {
                    task_name: name.to_string(),
                    worktree_path: path,
                    last_accessed,
                    age_days: (Utc::now() - last_accessed).num_days(),
                    has_uncommitted: self.check_uncommitted(&path)?,
                    has_active_session: false,
                    is_orphan: true,
                });
            }
        }

        Ok(orphans)
    }

    pub async fn cleanup(
        &mut self,
        targets: Vec<CleanupTarget>,
        dry_run: bool,
        force: bool,
    ) -> Result<()> {
        if targets.is_empty() {
            println!("No worktrees to clean up.");
            return Ok(());
        }

        // Display preview
        self.display_cleanup_preview(&targets);

        // Safety checks
        let has_warnings = targets.iter().any(|t| {
            t.has_uncommitted || t.has_active_session
        });

        if has_warnings {
            println!("\n⚠️  WARNING: Some worktrees have uncommitted changes or active sessions!");
        }

        // Dry run mode - just preview
        if dry_run {
            println!("\n[DRY RUN] No changes made. Use without --dry-run to cleanup.");
            return Ok(());
        }

        // Confirmation prompt (unless --force)
        if !force {
            print!("\nProceed with cleanup? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cleanup cancelled.");
                return Ok(());
            }
        }

        // Perform cleanup
        for target in &targets {
            self.cleanup_worktree(target).await?;
        }

        println!("\n✓ Cleaned up {} worktree(s)", targets.len());

        Ok(())
    }

    async fn cleanup_worktree(&mut self, target: &CleanupTarget) -> Result<()> {
        println!("Cleaning up: {}", target.task_name);

        // Get task and worktree from DB
        if let Ok(task) = self.db.get_task_by_name(&target.task_name) {
            // Remove via task removal (handles git and DB)
            task.remove(&self.project).await?;
        } else {
            // Orphan - remove filesystem only
            fs::remove_dir_all(&target.worktree_path)?;
        }

        // Log activity
        Activity::log_cleanup(&target.task_name, target.age_days)?;

        Ok(())
    }

    fn check_uncommitted(&self, worktree_path: &Path) -> Result<bool> {
        let repo = Repository::open(worktree_path)?;
        let statuses = repo.statuses(None)?;

        Ok(!statuses.is_empty())
    }

    fn display_cleanup_preview(&self, targets: &[CleanupTarget]) {
        println!("\n=== Cleanup Preview ===");
        println!("Found {} worktree(s) to clean up:\n", targets.len());

        for target in targets {
            let status_icon = if target.has_uncommitted {
                "⚠️ "
            } else if target.has_active_session {
                "▶️ "
            } else {
                "  "
            };

            let orphan_tag = if target.is_orphan { " [ORPHAN]" } else { "" };

            println!(
                "{}{:<20} ({}d ago){}",
                status_icon,
                target.task_name,
                target.age_days,
                orphan_tag
            );

            if target.has_uncommitted {
                println!("    └─ Has uncommitted changes");
            }
            if target.has_active_session {
                println!("    └─ Has active tmux session");
            }
        }
    }
}
```

**CLI Interface**:
```rust
// src/cli/clean.rs
#[derive(Parser)]
pub struct CleanArgs {
    /// Remove worktrees older than N days
    #[arg(long, value_name = "DAYS")]
    stale: Option<i64>,

    /// Remove orphaned worktrees (no DB record)
    #[arg(long)]
    orphans: bool,

    /// Clean both stale and orphaned worktrees
    #[arg(long)]
    all: bool,

    /// Preview without deleting
    #[arg(long)]
    dry_run: bool,

    /// Skip confirmation prompt
    #[arg(long)]
    force: bool,
}

pub async fn clean(args: CleanArgs) -> Result<()> {
    let project = Project::load_current()?;
    let mut cleaner = Cleaner::new(project);

    let mut targets = Vec::new();

    if let Some(days) = args.stale {
        targets.extend(cleaner.find_stale_worktrees(days).await?);
    }

    if args.orphans || args.all {
        targets.extend(cleaner.find_orphan_worktrees().await?);
    }

    if args.all && args.stale.is_none() {
        // Default to 14 days for --all
        targets.extend(cleaner.find_stale_worktrees(14).await?);
    }

    cleaner.cleanup(targets, args.dry_run, args.force).await?;

    Ok(())
}
```

**Success Criteria**:
- [ ] `kayfabe clean --stale 7d` detects worktrees older than 7 days
- [ ] `kayfabe clean --stale 14d --dry-run` shows preview without deleting
- [ ] `kayfabe clean --orphans` detects worktrees without DB records
- [ ] Shows detailed preview with age and warnings
- [ ] Detects uncommitted changes and warns user
- [ ] Detects active tmux sessions and warns user
- [ ] Confirmation prompt works correctly
- [ ] `--force` flag skips confirmation
- [ ] Database updated atomically after cleanup
- [ ] Filesystem and database stay consistent
- [ ] Activities log cleanup events
- [ ] Can clean up 100+ worktrees efficiently

**Testing**:
- Create tasks, wait, run cleanup with various thresholds
- Test dry-run mode (verify no deletion)
- Test orphan detection (create worktree manually)
- Test with uncommitted changes (verify warning)
- Test with active tmux session (verify warning)
- Test force mode (verify no prompt)
- Test cleanup of 50+ worktrees
- Verify database consistency after cleanup
- Test with invalid worktree paths
- Test cancellation at confirmation prompt

### Phase 7: Testing, Documentation & Polish (3-4 days)
**Goal**: Comprehensive testing, documentation, and final refinements

**Key Tasks**:
- Write comprehensive README with examples
- Implement status command with detailed project info
- Enhance error messages with actionable suggestions
- Add --help text for all commands and flags
- Write unit tests for core modules (80%+ coverage)
- Create integration tests for full workflows
- Add TUI tests with mock backend
- Document Claude integration patterns
- Create architecture documentation
- Add troubleshooting guide
- Write development setup instructions
- Add performance benchmarks
- Create example workflows and tutorials

**Files Created**:
- `README.md` - Quick start and installation
- `docs/architecture.md` - System design and components
- `docs/workflows.md` - Common workflows and patterns
- `docs/troubleshooting.md` - Common issues and solutions
- `docs/development.md` - Contributing and local setup
- `src/cli/status.rs` - Project status command
- `tests/unit/` - Unit tests for all modules
- `tests/integration/` - End-to-end workflow tests
- `benches/` - Performance benchmarks

**Testing Strategy**:

**Unit Tests** (in each module):
```rust
// tests/unit/task_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_creation() {
        let project = setup_test_project().await;
        let task = Task::create(&project, "test-task", None).await.unwrap();

        assert_eq!(task.name, "test-task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.worktree_path.is_some());
    }

    #[tokio::test]
    async fn test_duplicate_task_name() {
        let project = setup_test_project().await;
        Task::create(&project, "duplicate", None).await.unwrap();

        let result = Task::create(&project, "duplicate", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_task_removal_cleanup() {
        let project = setup_test_project().await;
        let task = Task::create(&project, "remove-me", None).await.unwrap();
        let worktree_path = task.worktree_path.clone().unwrap();

        task.remove(&project).await.unwrap();

        assert!(!worktree_path.exists());
        assert!(Database::get_task(&task.id).is_err());
    }
}
```

**Integration Tests**:
```rust
// tests/integration/full_workflow_test.rs
#[tokio::test]
async fn test_complete_task_workflow() {
    // 1. Initialize project
    let project = Project::init(
        "test-project",
        "https://github.com/example/repo.git"
    ).await.unwrap();

    assert!(project.path.join(".git").exists());
    assert!(project.path.join("kayfabe.md").exists());

    // 2. Create task
    let task = Task::create(&project, "feature-x", Some("Test feature")).await.unwrap();
    assert_eq!(task.status, TaskStatus::Pending);

    let worktree = Database::get_worktree_by_task(&task.id).unwrap();
    assert!(worktree.path.exists());

    // 3. Execute task
    let mut executor = Executor::new(task.clone(), project.clone());
    executor.execute().await.unwrap();

    let updated_task = Database::get_task(&task.id).unwrap();
    assert_eq!(updated_task.status, TaskStatus::Active);

    // 4. Verify execution record
    let exec = Database::get_active_execution(&task.id).unwrap().unwrap();
    assert_eq!(exec.status, ProcessStatus::Running);
    assert!(exec.tmux_session.starts_with("kayfabe-"));

    // 5. Kill task
    kill_task(&task.name).await.unwrap();

    let final_task = Database::get_task(&task.id).unwrap();
    assert_eq!(final_task.status, TaskStatus::Pending);

    // 6. Cleanup
    project.remove().await.unwrap();
    assert!(!project.path.exists());
}
```

**TUI Tests**:
```rust
// tests/tui/app_tests.rs
#[tokio::test]
async fn test_tui_navigation() {
    let mut terminal = setup_test_terminal();
    let project = setup_test_project().await;

    // Create multiple tasks
    for i in 0..5 {
        Task::create(&project, &format!("task-{}", i), None).await.unwrap();
    }

    let mut app = AppState::new(&project).await.unwrap();

    // Test navigation
    assert_eq!(app.selected_index, 0);

    app.select_next();
    assert_eq!(app.selected_index, 1);

    app.select_previous();
    assert_eq!(app.selected_index, 0);

    // Test filter
    app.set_filter(TaskFilter::Pending);
    assert_eq!(app.filtered_tasks().len(), 5);

    // Test rendering (no panics)
    terminal.draw(|f| ui::render(f, &app)).unwrap();
}
```

**Status Command Implementation**:
```rust
// src/cli/status.rs
pub async fn status() -> Result<()> {
    let project = Project::load_current()?;

    // Gather statistics
    let all_tasks = Database::get_tasks_by_project(&project.id)?;
    let pending = all_tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
    let active = all_tasks.iter().filter(|t| t.status == TaskStatus::Active).count();
    let completed = all_tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();

    let worktrees = Database::get_worktrees_by_project(&project.id)?;
    let executions = Database::get_recent_executions(&project.id, 10)?;

    // Display status
    println!("╭─────────────────────────────────────────╮");
    println!("│  Kayfabe Project Status                 │");
    println!("╰─────────────────────────────────────────╯");
    println!();
    println!("Project: {}", project.name);
    println!("Repository: {}", project.git_url);
    println!("Path: {}", project.path.display());
    println!();
    println!("Tasks:");
    println!("  Total:     {}", all_tasks.len());
    println!("  Pending:   {} 🔵", pending);
    println!("  Active:    {} 🟢", active);
    println!("  Completed: {} ✅", completed);
    println!();
    println!("Worktrees: {}", worktrees.len());
    println!("Recent executions: {}", executions.len());

    if !executions.is_empty() {
        println!();
        println!("Recent activity:");
        for exec in executions.iter().take(5) {
            let task = Database::get_task(&exec.task_id)?;
            println!("  {} - {} ({:?})",
                exec.started_at.format("%Y-%m-%d %H:%M"),
                task.name,
                exec.status
            );
        }
    }

    Ok(())
}
```

**Error Handling Enhancements**:
```rust
// src/errors.rs
use anyhow::{Context, Result};

pub trait KayfabeContext<T> {
    fn kayfabe_context(self, msg: &str) -> Result<T>;
}

impl<T, E> KayfabeContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn kayfabe_context(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
            .context(get_helpful_suggestion(msg))
    }
}

fn get_helpful_suggestion(error_msg: &str) -> String {
    match error_msg {
        msg if msg.contains("repository") => {
            "Suggestion: Run 'kayfabe init' to initialize the project first".to_string()
        }
        msg if msg.contains("tmux") => {
            "Suggestion: Install tmux with: brew install tmux (macOS) or apt install tmux (Linux)".to_string()
        }
        msg if msg.contains("task") && msg.contains("not found") => {
            "Suggestion: Run 'kayfabe list' to see available tasks".to_string()
        }
        _ => "".to_string(),
    }
}
```

**Documentation Structure**:

**README.md**:
- Quick start (5 commands)
- Installation instructions
- Basic usage examples
- Link to full documentation

**docs/architecture.md**:
- System architecture diagram
- Component relationships
- Data flow diagrams
- Database schema visualization
- Technology choices rationale

**docs/workflows.md**:
- Common workflows with examples
- Claude integration patterns
- Best practices
- Tips and tricks
- Example kayfabe.md and claude.md files

**docs/troubleshooting.md**:
- Common errors and solutions
- Debug mode instructions
- Log file locations
- Recovery procedures
- FAQ

**docs/development.md**:
- Local setup instructions
- Running tests
- Building from source
- Contributing guidelines
- Release process

**Performance Benchmarks**:
```rust
// benches/task_operations.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_task_creation(c: &mut Criterion) {
    c.bench_function("task create", |b| {
        b.iter(|| {
            // Setup
            let project = setup_test_project();

            // Benchmark
            black_box(Task::create(&project, "bench-task", None))
        });
    });
}

fn benchmark_task_list(c: &mut Criterion) {
    let project = setup_project_with_100_tasks();

    c.bench_function("list 100 tasks", |b| {
        b.iter(|| {
            black_box(Database::get_tasks_by_project(&project.id))
        });
    });
}

criterion_group!(benches, benchmark_task_creation, benchmark_task_list);
criterion_main!(benches);
```

**Success Criteria**:
- [ ] README is complete with quick start
- [ ] All docs/ files created and comprehensive
- [ ] All commands have detailed --help text
- [ ] Error messages include actionable suggestions
- [ ] `kayfabe status` shows complete project overview
- [ ] Unit test coverage >80% for core modules
- [ ] Integration tests cover full workflows
- [ ] TUI tests verify UI behavior
- [ ] Performance benchmarks established
- [ ] `kayfabe list` <50ms for 100 tasks
- [ ] `kayfabe add` <200ms
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] Documentation reviewed and accurate

**Testing Checklist**:
- [ ] Test on macOS
- [ ] Test on Linux
- [ ] Test with SSH git URLs
- [ ] Test with HTTPS git URLs
- [ ] Test with private repositories
- [ ] Test with 100+ tasks
- [ ] Test TUI with various terminal sizes
- [ ] Test cleanup with large worktrees
- [ ] Test concurrent task execution
- [ ] Test graceful degradation (no tmux, no git, etc.)
- [ ] Test error recovery scenarios
- [ ] Load test database with 1000+ tasks

## Thoughts & Artifact Management

### Directory Structure

```
thoughts/
├── <task-name>/
│   ├── 2026-01-23_140530_thought.md
│   ├── 2026-01-23_141205_artifact.md
│   ├── 2026-01-23_142015_error.md
│   └── summary.md  (auto-generated)
```

### Activity Types

- **thought** - Claude's reasoning, planning documents, decision logs
- **artifact** - Generated READMEs, documentation, code snippets
- **error** - Error reports, debugging information, stack traces
- **summary** - Auto-generated task summary (created on task completion)

### Naming Convention

- **Format**: `{timestamp}_{type}.md`
- **Timestamp**: ISO 8601 format (`YYYY-MM-DD_HHmmss`)
- **Type**: `thought`, `artifact`, `error`, `summary`
- **Examples**:
  - `2026-01-23_140530_thought.md` - Thought captured at 2:05:30 PM
  - `2026-01-23_141205_artifact.md` - Artifact created at 2:12:05 PM

### File Format

Each thought/artifact file includes frontmatter for metadata:

```markdown
---
type: thought
task: feature-x
timestamp: 2026-01-23T14:05:30Z
tags: [planning, architecture]
---

# Feature X Planning

Claude's reasoning and planning content here...
```

### TUI Integration

**List View:**
- Display thought count next to each task: `feature-x (12 thoughts)`
- Show latest thought timestamp
- Icon indicator for recent thoughts (< 1 hour)

**Detail Panel:**
- Preview latest thought excerpt
- Show thought activity timeline
- Keyboard shortcut to open thoughts directory

**Keyboard Shortcuts:**
- `t` - Open thoughts directory in default editor
- `Shift+T` - Show thoughts timeline overlay

### Implementation

**File Watcher**:
```rust
// src/core/thoughts.rs
use notify::{Watcher, RecursiveMode, Event};

pub struct ThoughtWatcher {
    watcher: RecommendedWatcher,
    thoughts_dir: PathBuf,
}

impl ThoughtWatcher {
    pub fn watch(project: &Project) -> Result<Self> {
        let thoughts_dir = project.path.join("thoughts");

        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(tx)?;

        watcher.watch(&thoughts_dir, RecursiveMode::Recursive)?;

        // Spawn background task to process events
        tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                if let Ok(Event { kind: Create(_), paths, .. }) = event {
                    for path in paths {
                        process_new_thought(&path).await;
                    }
                }
            }
        });

        Ok(Self { watcher, thoughts_dir })
    }
}

async fn process_new_thought(path: &Path) -> Result<()> {
    // Parse frontmatter
    let content = fs::read_to_string(path)?;
    let (metadata, _) = parse_frontmatter(&content)?;

    // Create activity record
    let activity = Activity {
        id: Uuid::new_v4().to_string(),
        task_id: metadata.task,
        activity_type: metadata.type,
        file_path: Some(path.to_path_buf()),
        content: Some(content),
        created_at: metadata.timestamp,
        metadata_json: serde_json::to_string(&metadata.tags)?,
    };

    Database::insert_activity(&activity)?;

    Ok(())
}
```

**Summary Generation**:
```rust
// Auto-generate summary on task completion
pub async fn generate_task_summary(task: &Task) -> Result<()> {
    let thoughts_dir = get_thoughts_dir(task);
    let summary_path = thoughts_dir.join("summary.md");

    // Collect all thoughts
    let activities = Database::get_activities_by_task(&task.id)?;

    // Generate summary using LLM (optional) or template
    let summary = create_summary_from_activities(&activities);

    // Write summary file
    fs::write(&summary_path, summary)?;

    Ok(())
}
```

## Database Schema

### Core Schema

```sql
-- Schema versioning and migrations
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL,
    description TEXT
);

-- Projects table
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    repo_path TEXT NOT NULL,
    worktree_dir TEXT NOT NULL,
    git_url TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    config_json TEXT  -- Flexible config storage for future extensions
);

-- Tasks table with enhanced tracking
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'active', 'completed', 'archived')),
    worktree_path TEXT,
    git_branch TEXT,  -- Track the git branch name
    last_accessed_at TEXT,  -- For stale detection
    cleanup_eligible BOOLEAN DEFAULT 0,  -- Flag for cleanup
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    metadata_json TEXT,  -- Flexible metadata storage
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(project_id, name)
);

-- Worktrees table
CREATE TABLE worktrees (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    branch_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_accessed_at TEXT NOT NULL,
    is_orphan BOOLEAN DEFAULT 0,  -- Track orphaned worktrees
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Activities table for thought/artifact tracking
CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    activity_type TEXT NOT NULL CHECK(activity_type IN ('thought', 'artifact', 'error', 'summary')),
    content TEXT,
    file_path TEXT,
    created_at TEXT NOT NULL,
    metadata_json TEXT,  -- Store additional activity metadata
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Execution processes table for stdout/stderr capture
CREATE TABLE execution_processes (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    command TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'killed')),
    exit_code INTEGER,
    stdout TEXT,  -- Captured stdout
    stderr TEXT,  -- Captured stderr
    started_at TEXT NOT NULL,
    completed_at TEXT,
    tmux_session TEXT,  -- Associated tmux session
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

### Performance Indexes

```sql
-- Composite indexes for common query patterns
CREATE INDEX idx_tasks_project_status ON tasks(project_id, status);
CREATE INDEX idx_tasks_last_accessed ON tasks(last_accessed_at);
CREATE INDEX idx_tasks_cleanup_eligible ON tasks(cleanup_eligible, last_accessed_at);
CREATE INDEX idx_worktrees_task ON worktrees(task_id);
CREATE INDEX idx_worktrees_orphan ON worktrees(is_orphan);
CREATE INDEX idx_activities_task_created ON activities(task_id, created_at);
CREATE INDEX idx_activities_type ON activities(activity_type, created_at);
CREATE INDEX idx_execution_processes_task ON execution_processes(task_id);
CREATE INDEX idx_execution_processes_status ON execution_processes(status, started_at);
```

### Migration Strategy

**Approach:**
- SQL migration files numbered sequentially (001, 002, etc.)
- Track applied migrations in `schema_migrations` table
- Support forward migrations (rollback optional for MVP)
- Validate schema integrity on startup
- Use transactions for atomic migrations

**Migration Files Structure:**
```
src/db/migrations/
├── 001_initial_schema.sql           # Core tables
├── 002_add_execution_processes.sql  # Process monitoring
├── 003_add_composite_indexes.sql    # Performance optimization
└── migration.rs                     # Migration runner
```

**Implementation:**
```rust
// src/db/migrations.rs
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL,
            description TEXT
        )",
        [],
    )?;

    // Apply pending migrations in order
    for migration in MIGRATIONS {
        if !is_applied(conn, migration.version)? {
            apply_migration(conn, migration)?;
        }
    }

    Ok(())
}
```

### Timeline Summary

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1: Foundation | 2-3 days | Database schema with migrations, data models, CLI skeleton |
| Phase 2: Project Init | 2-3 days | Project initialization, git clone, config templates |
| Phase 3: Task CRUD | 4-5 days | Task management, worktree creation/removal, isolation |
| Phase 4: TUI | 3-4 days | Terminal UI with navigation, filtering, task details |
| Phase 5: Tmux & Process Monitoring | 3 days | Process execution, output capture, session management |
| Phase 6: Enhanced Cleanup | 2 days | Stale detection, orphan cleanup, dry-run mode |
| Phase 7: Testing & Documentation | 3-4 days | Tests, docs, error handling, polish |
| **Total** | **19-24 days** | **Fully functional MVP** |

**Timeline Rationale**:
- Original estimate: 14-20 days
- Revised to 19-24 days based on:
  - vibe-kanban complexity analysis (73 migrations shows iterative nature)
  - Process monitoring requirements (stdout/stderr capture)
  - Enhanced cleanup strategy (orphan detection, dry-run)
  - Comprehensive testing needs (unit + integration + TUI tests)
  - More realistic buffer for unexpected issues

**Critical Path**:
1. Foundation → Project Init → Task CRUD (sequential, can't parallelize)
2. TUI and Tmux can be developed in parallel after Task CRUD
3. Cleanup depends on Task CRUD
4. Testing and documentation spans all phases

## Configuration Management

### Global Configuration (~/.kayfabe/)

```
~/.kayfabe/
├── kayfabe.db          # Global project registry
├── config.toml         # User preferences
└── templates/          # Default templates
    ├── kayfabe.md
    └── claude.md
```

### Project Configuration

```
<project>/
├── .kayfabe/
│   ├── config.toml     # Project-specific settings
│   └── state.json      # Runtime state (sessions, locks)
├── kayfabe.md          # Project documentation
└── claude.md           # Claude instructions
```

### Configuration File Format

**Global config.toml** (`~/.kayfabe/config.toml`):
```toml
[defaults]
cleanup_threshold_days = 14
default_shell = "bash"
theme = "dark"

[worktree]
base_dir_suffix = "-wt"  # myproject -> myproject-wt
orphan_cleanup_enabled = true

[execution]
tmux_session_prefix = "kayfabe"
capture_output = true
max_output_size_mb = 50

[ui]
show_git_status = true
show_thought_count = true
task_sort = "updated_at"  # created_at, name, status
refresh_interval_ms = 1000
```

**Project config.toml** (`.kayfabe/config.toml`):
```toml
[project]
name = "myproject"
repo_url = "git@github.com:user/repo.git"

[worktree]
base_dir = "myproject-wt"
cleanup_threshold_days = 7  # Override global default
branch_prefix = "kayfabe"  # kayfabe/<task-name>

[execution]
default_command = "claude"
env_vars = { "EDITOR" = "vim" }

[claude]
model = "sonnet"
context_files = ["kayfabe.md", "claude.md"]
```

### Configuration Precedence

Priority (highest to lowest):
1. **CLI flags** - `--stale 7d`
2. **Project config** - `.kayfabe/config.toml`
3. **Global config** - `~/.kayfabe/config.toml`
4. **Built-in defaults** - Hardcoded in application

Example:
```rust
// src/config.rs
#[derive(Debug, Deserialize)]
pub struct Config {
    pub worktree: WorktreeConfig,
    pub execution: ExecutionConfig,
    pub ui: UiConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Start with defaults
        let mut config = Config::default();

        // Load global config
        if let Some(global) = Self::load_global()? {
            config.merge(global);
        }

        // Load project config
        if let Some(project) = Self::load_project()? {
            config.merge(project);
        }

        Ok(config)
    }

    fn load_global() -> Result<Option<Config>> {
        let path = dirs::home_dir()
            .context("No home directory")?
            .join(".kayfabe")
            .join("config.toml");

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        Ok(Some(toml::from_str(&content)?))
    }

    fn load_project() -> Result<Option<Config>> {
        let path = Path::new(".kayfabe").join("config.toml");

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        Ok(Some(toml::from_str(&content)?))
    }
}
```

### Environment Variables

Support for runtime overrides via environment variables:

- `KAYFABE_DB` - Override database path
- `KAYFABE_CONFIG` - Override global config location
- `KAYFABE_LOG_LEVEL` - Set logging level (debug, info, warn, error)
- `KAYFABE_WORKTREE_DIR` - Override worktree base directory

Example:
```bash
KAYFABE_LOG_LEVEL=debug kayfabe execute task-1
```

## Performance Considerations

### Database Optimization

**Indexing Strategy**:
- Composite indexes on frequently-joined tables
- Covering indexes for common queries
- Index on last_accessed_at for cleanup queries

**Query Optimization**:
```rust
// Use prepared statements for common queries
pub struct PreparedQueries {
    get_task_by_name: Statement<'static>,
    get_tasks_by_status: Statement<'static>,
    insert_activity: Statement<'static>,
}

impl PreparedQueries {
    pub fn new(conn: &Connection) -> Result<Self> {
        Ok(Self {
            get_task_by_name: conn.prepare(
                "SELECT * FROM tasks WHERE name = ? AND project_id = ?"
            )?,
            get_tasks_by_status: conn.prepare(
                "SELECT * FROM tasks WHERE status = ? ORDER BY updated_at DESC"
            )?,
            insert_activity: conn.prepare(
                "INSERT INTO activities (id, task_id, activity_type, content, created_at)
                 VALUES (?, ?, ?, ?, ?)"
            )?,
        })
    }
}
```

**Connection Pooling**:
```rust
// Use r2d2 for connection pooling in async contexts
type DbPool = r2d2::Pool<SqliteConnectionManager>;

pub fn create_pool(db_path: &Path) -> Result<DbPool> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = r2d2::Pool::builder()
        .max_size(10)
        .build(manager)?;
    Ok(pool)
}
```

### Performance Benchmarks

**Target Performance** (MVP):
- `kayfabe list` < 50ms for 100 tasks
- `kayfabe add` < 200ms (includes git operations)
- `kayfabe execute` < 500ms (includes tmux spawn)
- TUI rendering < 16ms (60fps)
- Database queries < 10ms average

**Scaling Limits** (tested):
- 1,000 tasks per project
- 100 active worktrees simultaneously
- 10,000 activity records
- 100MB database size (typical usage)

### Optimization Strategies

**TUI Rendering**:
- Lazy-load worktree details only when selected
- Cache git repository status (refresh every 1s)
- Virtualized scrolling for large task lists
- Debounce keyboard events

**Git Operations**:
- Cache repository object
- Batch worktree operations where possible
- Use git worktree list once, parse results

**Filesystem Operations**:
- Stream large outputs instead of buffering
- Use async I/O for all file operations
- Implement output rotation for large stdout/stderr

**Memory Management**:
```rust
// Limit stdout/stderr capture size
const MAX_OUTPUT_SIZE: usize = 50 * 1024 * 1024; // 50MB

async fn capture_stream_with_limit(
    stream: impl AsyncRead,
    limit: usize,
) -> Result<String> {
    let mut buffer = Vec::with_capacity(limit);
    let mut reader = BufReader::new(stream);

    while buffer.len() < limit {
        let chunk = reader.fill_buf().await?;
        if chunk.is_empty() {
            break;
        }

        let to_read = std::cmp::min(chunk.len(), limit - buffer.len());
        buffer.extend_from_slice(&chunk[..to_read]);
        reader.consume(to_read);
    }

    Ok(String::from_utf8_lossy(&buffer).to_string())
}
```

### Performance Monitoring

Add `--profile` flag to commands for performance analysis:
```rust
// src/utils/profiler.rs
pub struct Profiler {
    start: Instant,
    checkpoints: Vec<(String, Duration)>,
}

impl Profiler {
    pub fn checkpoint(&mut self, label: &str) {
        self.checkpoints.push((
            label.to_string(),
            self.start.elapsed(),
        ));
    }

    pub fn report(&self) {
        println!("\n=== Performance Profile ===");
        for (label, duration) in &self.checkpoints {
            println!("{:<30} {:>8.2}ms", label, duration.as_secs_f64() * 1000.0);
        }
    }
}

// Usage:
// kayfabe add task-1 --profile
// Output:
// === Performance Profile ===
// Database connection         2.34ms
// Git worktree creation      145.67ms
// Database update             3.21ms
// Thoughts directory          1.05ms
// Total                     152.27ms
```

## Error Handling Patterns

### Error Categories

```rust
// src/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum KayfabeError {
    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Task '{0}' not found")]
    TaskNotFound(String),

    #[error("Task '{0}' already exists")]
    TaskExists(String),

    #[error("Worktree creation failed: {0}")]
    WorktreeCreation(String),

    #[error("Tmux error: {0}")]
    Tmux(String),

    #[error("Process execution failed: {0}")]
    Execution(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Error Context and Recovery

```rust
// Provide actionable error messages
impl KayfabeError {
    pub fn suggestion(&self) -> Option<String> {
        match self {
            KayfabeError::TaskNotFound(name) => {
                Some(format!("Run 'kayfabe list' to see available tasks, or 'kayfabe add {}' to create it", name))
            }
            KayfabeError::Git(e) if e.message().contains("not a git repository") => {
                Some("Run 'kayfabe init' to initialize the project first".to_string())
            }
            KayfabeError::Tmux(_) => {
                Some("Install tmux: brew install tmux (macOS) or apt install tmux (Linux)".to_string())
            }
            _ => None,
        }
    }
}

// Display errors with suggestions
pub fn display_error(error: &KayfabeError) {
    eprintln!("Error: {}", error);

    if let Some(suggestion) = error.suggestion() {
        eprintln!("\nSuggestion:");
        eprintln!("  {}", suggestion);
    }

    // Log full error to debug file
    log_error_to_file(error);
}
```

### Recovery Strategies

**Database Corruption**:
```rust
// Attempt recovery or create backup
pub fn recover_database(db_path: &Path) -> Result<()> {
    // 1. Try to open and validate
    match Connection::open(db_path) {
        Ok(conn) => {
            if validate_schema(&conn).is_ok() {
                return Ok(());
            }
        }
        Err(_) => {}
    }

    // 2. Backup corrupted database
    let backup_path = db_path.with_extension("db.backup");
    fs::copy(db_path, &backup_path)?;

    // 3. Recreate database
    fs::remove_file(db_path)?;
    let conn = Connection::open(db_path)?;
    run_migrations(&conn)?;

    eprintln!("Database corrupted and recreated. Backup saved to: {:?}", backup_path);

    Ok(())
}
```

**Orphaned Worktrees**:
```rust
// Detect and repair filesystem/database mismatches
pub async fn repair_worktrees(project: &Project) -> Result<()> {
    let db_worktrees = Database::get_worktrees_by_project(&project.id)?;
    let fs_worktrees = scan_worktree_directory(&project.worktree_dir)?;

    // Find orphans (in filesystem but not database)
    for fs_wt in &fs_worktrees {
        if !db_worktrees.iter().any(|db| db.path == fs_wt.path) {
            println!("Found orphaned worktree: {:?}", fs_wt.path);
            println!("  1. Remove: kayfabe clean --orphans");
            println!("  2. Import: kayfabe import {:?}", fs_wt.path);
        }
    }

    // Find missing (in database but not filesystem)
    for db_wt in &db_worktrees {
        if !fs_worktrees.iter().any(|fs| fs.path == db_wt.path) {
            println!("Worktree missing from filesystem: {:?}", db_wt.path);
            println!("  Removing from database...");
            Database::delete_worktree(&db_wt.id)?;
        }
    }

    Ok(())
}
```

## Out of Scope (MVP)

- Web UI or dashboard
- Multi-user/authentication
- Cloud sync
- Jira/Linear integration
- Advanced diff viewers
- MCP server integration
- PR automation
- Task dependencies
- Time tracking

## Documentation Plan

### Documentation Structure

```
docs/
├── README.md              # Main documentation (quick start)
├── architecture.md        # System design and components
├── workflows.md           # Common workflows and examples
├── troubleshooting.md     # Common issues and solutions
├── development.md         # Contributing and local setup
├── api.md                 # Internal API documentation (future)
└── implementation-plan.md # This file (development roadmap)
```

### README.md

**Contents**:
- Project overview and value proposition
- Quick start (5 commands to get started)
- Installation instructions (cargo, homebrew)
- Basic usage examples
- Link to full documentation
- Contributing guidelines

**Example Quick Start**:
```bash
# 1. Install
cargo install kayfabe

# 2. Initialize project
kayfabe init myproject git@github.com:user/repo.git

# 3. Add a task
cd myproject
kayfabe add "feature-x" --description "Build new feature"

# 4. Execute task (opens Claude in tmux)
kayfabe execute feature-x

# 5. View all tasks
kayfabe list
```

### architecture.md

**Contents**:
- System architecture diagram
- Component relationships (CLI → Core → Database)
- Data flow diagrams (task creation, execution, cleanup)
- Database schema visualization
- Technology choices and rationale
- Worktree architecture explanation
- Process monitoring architecture

**Diagrams to Include**:
```
┌──────────────────────────────────────────┐
│              CLI Layer                    │
│  (init, add, list, execute, clean, etc.) │
└───────────────┬──────────────────────────┘
                │
┌───────────────▼──────────────────────────┐
│            Core Business Logic            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │  Task    │ │ Worktree │ │ Executor │ │
│  │ Manager  │ │ Manager  │ │          │ │
│  └──────────┘ └──────────┘ └──────────┘ │
└───────────────┬──────────────────────────┘
                │
┌───────────────▼──────────────────────────┐
│          Database Layer (SQLite)          │
│  (projects, tasks, worktrees, activities) │
└───────────────────────────────────────────┘
```

### workflows.md

**Contents**:
- Common workflows with step-by-step examples
- Claude integration patterns
- Best practices for task management
- Example kayfabe.md and claude.md files
- Multi-task workflows
- Team collaboration patterns (future)
- Tips and tricks

**Example Workflows**:

**1. Feature Development Workflow**:
```bash
# Create task
kayfabe add "api-endpoint" --description "Build /users endpoint"

# Execute task
kayfabe execute api-endpoint

# [Claude helps build the feature]

# Check thoughts/artifacts
ls thoughts/api-endpoint/

# Complete and cleanup
kayfabe remove api-endpoint  # After merging to main
```

**2. Bug Fix Workflow**:
```bash
# Create bug fix task
kayfabe add "fix-auth-bug" --description "Fix JWT validation"

# Execute and debug
kayfabe execute fix-auth-bug

# View execution logs
kayfabe logs fix-auth-bug

# Clean up after fix
kayfabe remove fix-auth-bug
```

**3. Parallel Development**:
```bash
# Create multiple tasks
kayfabe add "feature-a"
kayfabe add "feature-b"
kayfabe add "refactor-c"

# Execute all in separate tmux sessions
kayfabe execute feature-a
kayfabe execute feature-b
kayfabe execute refactor-c

# View all tasks
kayfabe list  # Shows all active tasks
```

### troubleshooting.md

**Contents**:
- Common errors and solutions
- Debug mode instructions
- Log file locations
- Recovery procedures
- FAQ
- Performance troubleshooting

**Common Issues**:

**Issue**: "Task already exists"
```
Error: Task 'feature-x' already exists

Solution:
  - Use a different name: kayfabe add "feature-x-v2"
  - Remove existing task: kayfabe remove feature-x
  - View all tasks: kayfabe list
```

**Issue**: "Tmux not found"
```
Error: Tmux error: command not found

Solution:
  Install tmux:
    macOS:  brew install tmux
    Ubuntu: sudo apt install tmux
    Arch:   sudo pacman -S tmux
```

**Issue**: "Database corrupted"
```
Error: Database error: disk I/O error

Solution:
  1. Backup database: cp ~/.kayfabe/kayfabe.db ~/.kayfabe/kayfabe.db.backup
  2. Run repair: kayfabe repair
  3. If repair fails, recreate: rm ~/.kayfabe/kayfabe.db && kayfabe init
```

**Debug Mode**:
```bash
# Enable debug logging
export KAYFABE_LOG_LEVEL=debug
kayfabe add task-1

# View logs
tail -f ~/.kayfabe/logs/kayfabe.log
```

### development.md

**Contents**:
- Local development setup
- Building from source
- Running tests (unit, integration, TUI)
- Code structure explanation
- Contributing guidelines
- Release process
- Debugging tips

**Setup Instructions**:
```bash
# Clone repository
git clone https://github.com/user/kayfabe.git
cd kayfabe

# Build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- add test-task

# Install locally
cargo install --path .
```

**Testing**:
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# TUI tests
cargo test --test tui_tests

# Benchmarks
cargo bench

# Coverage
cargo tarpaulin --out Html
```

**Code Structure**:
```
src/
├── main.rs           # Entry point, CLI initialization
├── cli/              # Command implementations
│   ├── init.rs
│   ├── add.rs
│   ├── list.rs
│   └── ...
├── core/             # Business logic
│   ├── task.rs       # Task lifecycle management
│   ├── worktree.rs   # Git worktree operations
│   ├── executor.rs   # Process execution
│   └── cleaner.rs    # Cleanup logic
├── db/               # Database layer
│   ├── models.rs     # Data structures
│   ├── migrations.rs # Schema migrations
│   └── queries.rs    # Prepared queries
├── tui/              # Terminal UI
│   ├── app.rs        # Application state
│   ├── ui.rs         # Rendering
│   └── events.rs     # Event handling
└── utils/            # Utilities
    ├── git.rs        # Git helpers
    ├── tmux.rs       # Tmux integration
    └── paths.rs      # Path management
```

## Future Enhancements

1. **Agent System** - Symlinked custom scripts
2. **Enhanced TUI** - Diff viewer, thought visualization
3. **Git Automation** - Auto-commit, PR creation
4. **Cloud Sync** - Remote server, team features
5. **Issue Integration** - Jira/Linear/GitHub sync
6. **Analytics** - Time tracking, metrics
7. **Multi-Project** - Dashboard, cross-project features

## Key Design Decisions

### Why Bare Repository?
Allows multiple worktrees without cluttering a main working directory. Clean separation of concerns.

### Why SQLite?
Simple, reliable, serverless. Perfect for CLI tool. Easy backup/migration.

### Why Rust?
Fast, reliable, excellent CLI ecosystem. Same proven stack as vibe-kanban.

### Why Tmux?
Widely used, persistent sessions, scriptable. Natural fit for dev workflows.

### Directory Structure

**Project Structure**:
```
myproject/
├── .git/                              # Bare repository (no working directory)
│   ├── refs/heads/kayfabe/*          # Task branches
│   ├── worktrees/                     # Worktree metadata
│   └── ...
├── myproject-wt/                      # Worktree base directory
│   ├── task-1/                        # Isolated worktree for task-1
│   │   ├── .git                       # Git link file
│   │   └── [source code]
│   ├── task-2/                        # Isolated worktree for task-2
│   │   ├── .git
│   │   └── [source code]
│   └── feature-x/
│       ├── .git
│       └── [source code]
├── thoughts/                          # Claude artifacts and thoughts
│   ├── task-1/
│   │   ├── 2026-01-23_140530_thought.md
│   │   ├── 2026-01-23_141205_artifact.md
│   │   └── summary.md
│   ├── task-2/
│   │   └── ...
│   └── .gitkeep
├── .kayfabe/                          # Local project state
│   ├── config.toml                    # Project-specific config
│   └── state.json                     # Runtime state (locks, sessions)
├── kayfabe.md                         # Project documentation
├── claude.md                          # Claude-specific instructions
└── README.md                          # Project README (optional)
```

**Global Structure** (`~/.kayfabe/`):
```
~/.kayfabe/
├── kayfabe.db                         # Global project registry (SQLite)
├── config.toml                        # Global user preferences
├── logs/                              # Application logs
│   └── kayfabe.log
└── templates/                         # Default templates
    ├── kayfabe.md.template
    └── claude.md.template
```

**Benefits of This Structure**:
- **Bare Repository**: Enables multiple worktrees without a main working directory
- **Isolated Worktrees**: Each task has completely separate working directory
- **Centralized Thoughts**: All AI artifacts in one place, easy to backup
- **Local Config**: Project-specific settings without affecting global config
- **Global Registry**: One database tracks all projects across the system

## Design Decisions (Resolved)

Based on requirements clarification and research:

1. **Claude Invocation**: ✅ Use shell command `claude` for MVP
   - Simple, works with existing Claude CLI
   - Can be configured via project config

2. **Thoughts Parsing**: ✅ Simple file-based approach
   - Store in `thoughts/<task>/*.md` with naming convention
   - Watch directory for new files
   - Parse frontmatter for metadata
   - Auto-generate summaries on task completion

3. **Worktree Naming**: ✅ Use task slugs with sanitization
   - Branch naming: `kayfabe/<task-name-sanitized>`
   - Worktree directory: `<project>-wt/<task-name>`
   - Ensures readability and avoids collisions

4. **Status Transitions**: ✅ Auto-transition pending→active on execute
   - Tasks start as "pending"
   - Become "active" when executed
   - Become "completed" on successful exit
   - Become "archived" on failure

5. **Default Branch**: ✅ Detect from repository
   - Use `git2` to detect HEAD
   - Fall back to main/master if ambiguous
   - Store in project config for future reference

6. **UI Approach**: ✅ Terminal TUI with ratatui (not web-based)
   - Fast, lightweight
   - No server/browser overhead
   - Familiar terminal workflow

7. **Agent Support**: ✅ Claude-only for MVP
   - Multi-agent support planned for future
   - Architecture supports adding more agents later

8. **Process Logging**: ✅ Yes - capture stdout/stderr
   - Store in `execution_processes` table
   - Essential for debugging
   - Viewable via `kayfabe logs <task>`

## Success Metrics

### Technical
- All phases complete and tested
- <100ms command response time
- Handles 100+ tasks per project
- No data corruption
- Clean error handling

### User Experience  
- Intuitive command structure
- Helpful error messages
- Fast TUI navigation
- Smooth tmux integration
- Minimal configuration needed

## Getting Started

### Quick Start (5 Minutes)

```bash
# 1. Initialize project from git repository
kayfabe init myproject git@github.com:user/repo.git
cd myproject

# Directory structure created:
# myproject/
# ├── .git/              (bare repository)
# ├── myproject-wt/      (worktrees will go here)
# ├── thoughts/          (AI artifacts)
# ├── kayfabe.md         (project docs)
# └── claude.md          (Claude instructions)

# 2. Add your first task
kayfabe add "feature-auth" --description "Build user authentication"

# Creates:
# - Isolated git worktree at myproject-wt/feature-auth
# - Git branch: kayfabe/feature-auth
# - Database entry
# - Thoughts directory: thoughts/feature-auth/

# 3. Execute task with Claude
kayfabe execute feature-auth

# Opens:
# - New tmux session: kayfabe-feature-auth
# - Working directory: myproject-wt/feature-auth
# - Claude CLI with project context
# - Captures stdout/stderr to database

# 4. Attach to running session (from another terminal)
kayfabe attach feature-auth

# Or use tmux directly:
# tmux attach -t kayfabe-feature-auth

# 5. View all tasks with beautiful TUI
kayfabe list

# TUI shows:
# - All tasks with status indicators
# - Last updated time
# - Thought count
# - Git branch info
# - Keyboard navigation (↑↓, Enter to execute, r to remove)

# 6. View task execution logs
kayfabe logs feature-auth

# Shows:
# - Command executed
# - Exit code
# - Full stdout/stderr
# - Timestamps

# 7. Kill running task
kayfabe kill feature-auth

# Terminates:
# - Claude process
# - Tmux session
# - Updates task status to "pending"

# 8. Check project status
kayfabe status

# Displays:
# - Project info (name, git URL, path)
# - Task counts by status
# - Recent executions
# - Worktree count

# 9. Clean up stale worktrees (preview mode)
kayfabe clean --stale 7d --dry-run

# Shows preview of what would be deleted:
# - Tasks older than 7 days
# - Warnings for uncommitted changes
# - Warnings for active sessions

# 10. Actually clean up (with confirmation)
kayfabe clean --stale 7d

# Prompts for confirmation before deleting
# Use --force to skip confirmation

# 11. Remove completed task
kayfabe remove feature-auth

# Removes:
# - Git worktree
# - Git branch (optional)
# - Database records
# - Thoughts directory (optional, prompts)
```

### Complete Workflow Example

```bash
# Scenario: Build a new API endpoint

# 1. Setup
kayfabe init api-project git@github.com:company/api.git
cd api-project

# 2. Create task
kayfabe add "users-endpoint" --description "Build GET /users endpoint with pagination"

# 3. Start working with Claude
kayfabe execute users-endpoint

# [Claude session opens in tmux]
# Claude helps you:
# - Design the endpoint
# - Write the code
# - Generate tests
# - Create documentation
# All thoughts saved to thoughts/users-endpoint/

# 4. Check progress (from another terminal)
kayfabe list
# Shows users-endpoint as "active" with thought count

# 5. View what Claude generated
ls thoughts/users-endpoint/
# Output:
# 2026-01-23_140530_thought.md    # Planning
# 2026-01-23_141205_artifact.md   # Code implementation
# 2026-01-23_142015_thought.md    # Test strategy

# 6. After completing the work, detach from tmux
# Ctrl+b, then d

# 7. View execution summary
kayfabe logs users-endpoint

# 8. Remove task after merging to main
kayfabe remove users-endpoint
# Keeps thoughts/ directory for reference
```

### Advanced Usage

```bash
# Work on multiple tasks in parallel
kayfabe add "feature-a"
kayfabe add "feature-b"
kayfabe add "bug-fix"

kayfabe execute feature-a  # Session 1
kayfabe execute feature-b  # Session 2
kayfabe execute bug-fix    # Session 3

# View all active sessions
kayfabe status

# Switch between sessions
kayfabe attach feature-a
kayfabe attach feature-b

# Clean up all completed and old tasks
kayfabe clean --stale 14d

# Find and remove orphaned worktrees
kayfabe clean --orphans

# Both stale and orphans
kayfabe clean --all
```

### Configuration

```bash
# Global config
mkdir -p ~/.kayfabe
cat > ~/.kayfabe/config.toml << EOF
[defaults]
cleanup_threshold_days = 14

[execution]
default_command = "claude"

[ui]
theme = "dark"
task_sort = "updated_at"
EOF

# Project-specific config
cat > .kayfabe/config.toml << EOF
[worktree]
cleanup_threshold_days = 7  # More aggressive for this project

[claude]
model = "sonnet"
context_files = ["kayfabe.md", "claude.md", "README.md"]
EOF
```

## Risk Mitigation

| Risk | Impact | Probability | Mitigation Strategy |
|------|--------|-------------|---------------------|
| **Git worktree corruption** | High | Low | • Atomic operations with rollback<br>• Validation checks before/after creation<br>• Orphan detection and repair tools<br>• Transaction-based database updates |
| **Database corruption** | High | Low | • SQLite transactions for all writes<br>• Automatic backups before major operations<br>• Schema migrations with validation<br>• Recovery tools (`kayfabe repair`)<br>• Write-ahead logging (WAL mode) |
| **Tmux unavailable** | Medium | Medium | • Check for tmux on startup<br>• Graceful error with installation instructions<br>• Future: fallback to direct execution (no tmux) |
| **Claude CLI unavailable** | Medium | Medium | • Check for claude command before execute<br>• Clear error message with setup instructions<br>• Configurable command via config.toml |
| **Large task counts (100+)** | Medium | Medium | • Pagination in TUI (virtualized scrolling)<br>• Composite indexes on queries<br>• Lazy-loading of worktree details<br>• Tested up to 1000 tasks |
| **Concurrent access** | Low | Medium | • SQLite file locking (automatic)<br>• Git locks for worktree operations<br>• Advisory locks for cleanup operations<br>• Detect active processes before removal |
| **Disk space exhaustion** | Medium | Low | • Monitor worktree directory size<br>• Warn on low disk space<br>• Automatic cleanup suggestions<br>• Configurable output size limits |
| **Memory leaks (stdout/stderr)** | Medium | Low | • Limit output capture size (50MB default)<br>• Stream processing instead of buffering<br>• Rotation for long-running processes<br>• Memory profiling in tests |
| **Network failures (git clone)** | Low | Medium | • Retry logic with exponential backoff<br>• Clear error messages<br>• Support for local repositories<br>• Timeout configuration |
| **Filesystem inconsistency** | Medium | Low | • Consistency checks on startup<br>• `kayfabe repair` command<br>• Orphan detection<br>• Comprehensive logging |
| **Migration failures** | High | Low | • Test migrations thoroughly<br>• Validate schema after each migration<br>• Backup before migrations<br>• Rollback support (future) |
| **Process crashes** | Medium | Medium | • Capture stderr for debugging<br>• Record exit codes<br>• Automatic status updates<br>• Recovery suggestions in logs |
| **Terminal compatibility** | Low | Medium | • Test on multiple terminals (iTerm2, Terminal.app, Alacritty)<br>• Fallback colors for limited terminals<br>• Standard ANSI codes only |
| **Performance degradation** | Low | Low | • Benchmarks in CI/CD<br>• Performance profiling tools<br>• Monitoring and alerts<br>• Query optimization |

### Recovery Procedures

**Database Recovery**:
```bash
# Automatic repair
kayfabe repair

# Manual recovery
cp ~/.kayfabe/kayfabe.db ~/.kayfabe/kayfabe.db.backup
sqlite3 ~/.kayfabe/kayfabe.db "PRAGMA integrity_check"
```

**Worktree Recovery**:
```bash
# Detect and repair inconsistencies
kayfabe clean --orphans --dry-run
kayfabe clean --orphans

# Manual git worktree repair
git worktree list
git worktree prune
```

**Process Recovery**:
```bash
# Kill stuck processes
kayfabe kill <task>

# View execution history
kayfabe logs <task>

# Check tmux sessions
tmux list-sessions
```

## Next Steps

1. Review and approve this plan
2. Clarify open questions above
3. Create Cargo project structure
4. Begin Phase 1 implementation
5. Iterate based on testing and feedback
