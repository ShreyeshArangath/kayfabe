# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kayfabe is an AI-enhanced CLI task manager built in Rust that accelerates engineer productivity by combining task management with git worktree orchestration. It enables engineers to work on multiple tasks in parallel with isolated git worktrees, integrated tmux sessions, and AI assistant tracking.

### Core Mission
Enable engineers to manage a swarm of AI agents (like Claude) working in parallel across different tasks, with each task in its own isolated git worktree.

## Architecture

### High-Level Components

```
kayfabe/
├── src/
│   ├── cli/              # Command handlers (init, add, list, execute, etc.)
│   ├── core/             # Business logic
│   │   ├── task.rs       # Task lifecycle management
│   │   ├── worktree.rs   # Git worktree operations
│   │   ├── project.rs    # Project initialization & config
│   │   ├── cleaner.rs    # Stale worktree cleanup
│   │   ├── executor.rs   # Process execution & monitoring
│   │   ├── agent.rs      # Agent lifecycle (multi-agent extension)
│   │   ├── scheduler.rs  # Task scheduling & assignment
│   │   └── coordinator.rs # Multi-agent orchestration
│   ├── db/               # SQLite persistence layer
│   │   ├── models.rs     # Database models
│   │   └── migrations.rs # Schema migrations
│   ├── tui/              # Terminal UI (ratatui)
│   │   ├── dashboard.rs  # Agent status dashboard
│   │   └── agent_view.rs # Per-agent detailed view
│   └── utils/            # Helpers (tmux, paths)
```

### Tech Stack

- **CLI**: `clap` for argument parsing
- **Database**: `rusqlite` with migration system (expect 50+ migrations like vibe-kanban)
- **TUI**: `ratatui` + `crossterm` for interactive terminal UI
- **Git**: `git2` (git2-rs) + git CLI hybrid approach for worktree management
- **Async**: `tokio` for process management and concurrent agent execution
- **Serialization**: `serde` + `chrono` for data handling
- **Errors**: `anyhow` for error handling

### Multi-Agent Extension Stack
- `futures` - Agent coordination primitives
- `notify` - File system watching for agent outputs
- `sysinfo` - Resource monitoring (CPU, memory)

## Database Architecture

### Core Principles
1. **Schema Evolution**: Plan for 50-100+ migrations (learned from vibe-kanban's 73 migrations)
2. **Migration Tracking**: Use schema versioning table from day 1
3. **Composite Indexes**: Critical for query performance at scale
4. **Connection Pooling**: For async operations

### Key Tables

**tasks**: Core task tracking with status, worktree path, priority, assigned agent
**worktrees**: Git worktree metadata, branch names, last accessed times
**execution_processes**: Process tracking with stdout/stderr, exit codes, PIDs
**agents**: Agent pool tracking (status, current task, tmux session, resource usage)
**task_dependencies**: Sequential execution dependencies
**agent_events**: Agent activity audit log
**resource_snapshots**: CPU/memory usage tracking per agent

### Cleanup Strategy
- Configurable staleness thresholds (7d/14d/30d)
- Orphan detection to prevent filesystem/DB inconsistency
- `last_accessed_at` tracking essential for stale detection
- Dry-run mode to prevent accidental deletions

## Git Worktree Management

### Design Philosophy
- **One worktree per task** for isolation
- **Repository-level separation** improves multi-worktree coordination (learned from vibe-kanban)
- **Hybrid approach**: Use git2-rs for queries, git CLI for complex operations
- **Cleanup automation**: Detect and remove stale worktrees

### Collaborative Mode
- **Shared worktrees**: Multiple agents can work in same worktree with git locks
- **Parallel worktrees**: Each agent gets own worktree for independent subtasks

## Process Execution & Monitoring

### Execution Flow

1. **Tmux Session**: Create/attach to `kayfabe-<task-name>` session in task worktree
2. **Process Spawning**: Launch Claude via `tokio::process::Command` with piped stdout/stderr
3. **Output Capture**: Stream output to both terminal (tmux) and database in real-time
4. **Status Tracking**: Update task status, monitor exit codes, handle graceful shutdown

### Critical Implementation Details
- Store PIDs and tmux session info in database
- Buffer output intelligently to avoid memory issues (learned from vibe-kanban)
- Line-buffered streaming for live updates
- Capture exit codes to understand failures
- Handle SIGINT/SIGTERM for graceful shutdown

## Multi-Agent Orchestration

### Agent Lifecycle States
- **Idle**: Waiting for task assignment
- **Active**: Currently executing a task
- **Paused**: Suspended by user
- **Terminated**: Shut down

### Scheduling Strategies
- **RoundRobin**: Distribute tasks evenly across agents
- **PriorityBased**: High priority tasks first
- **LoadBalanced**: Consider agent resource usage
- **Manual**: User assigns tasks explicitly

### Coordination Modes
- **Parallel**: Independent tasks executed simultaneously (3-5 agents)
- **Sequential**: Task chains with dependencies
- **Collaborative**: Multiple agents on same task with roles (implementer, reviewer, tester)

## Development Workflow

### When Implementation Begins

Build commands (once Cargo.toml exists):
```bash
cargo build                    # Build debug binary
cargo build --release         # Build optimized binary
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test
cargo clippy                  # Linting
cargo fmt                     # Format code
```

### CLI Commands (Planned)

**Basic Task Management**:
```bash
kayfabe init                              # Initialize project
kayfabe add <task> [--priority high]     # Create task
kayfabe list                             # List tasks
kayfabe execute <task>                   # Execute task in tmux
kayfabe status                           # Show repository status
```

**Worktree Management**:
```bash
kayfabe worktree list                    # List all worktrees
kayfabe worktree list --stale 14         # List stale worktrees (>14 days)
kayfabe worktree remove <name>           # Remove worktree
kayfabe worktree cleanup                 # Clean up stale worktrees
```

**Multi-Agent Commands** (Extension):
```bash
kayfabe agents start --count 3           # Start agent pool
kayfabe agents list                      # List all agents with status
kayfabe agents show <agent-id>           # View agent details
kayfabe agents pause/resume <agent-id>   # Control agents
kayfabe agents stop [--all]              # Stop agents
kayfabe agents logs <agent-id> [--follow] # View agent logs

kayfabe add <task> --auto-assign         # Auto-assign to available agent
kayfabe add <task> --assign <agent-id>   # Assign to specific agent
kayfabe add <task> --depends-on <task>   # Create dependency

kayfabe execute --auto --mode parallel   # Execute with scheduler
kayfabe execute <task> --agents a1,a2,a3 --roles implement,review,test

kayfabe dashboard                        # Launch real-time TUI dashboard
```

## Configuration

### kayfabe.toml (Planned)

```toml
[project]
name = "myproject"
repository_path = "/path/to/repo"

[worktrees]
base_path = ".kayfabe/worktrees"
cleanup_threshold_days = 14
enable_auto_cleanup = true

[execution]
default_shell = "/bin/zsh"
enable_tmux = true
default_mode = "parallel"
enable_collaborative = true

[agents]
max_concurrent = 5
default_count = 2
auto_assign = true
scheduling_strategy = "priority"

[resource_limits]
max_cpu_percent = 80.0
max_memory_mb = 2048
max_duration_minutes = 240

[monitoring]
snapshot_interval_secs = 30
enable_heartbeat = true
heartbeat_interval_secs = 60
```

## Key Design Decisions

### From Implementation Research

1. **Iterative Schema Evolution**: Schema will evolve significantly - migrations are first-class citizens
2. **Process Output Capture**: Essential for debugging - store stdout/stderr with smart buffering
3. **Composite Indexes**: Add early for performance at scale
4. **Tmux Integration**: Provides persistent sessions ideal for dev workflows
5. **Resource Monitoring**: Track CPU/memory per agent to prevent system overload
6. **File System Watching**: Monitor agent outputs for thought/artifact tracking

### Performance Targets (Multi-Agent)

- Support 5 concurrent agents without degradation
- <100ms scheduler assignment time
- <50ms dashboard refresh rate
- <500MB overhead per agent

## Important Implementation Notes

### Git Worktree Isolation
Each task gets its own worktree on a dedicated branch. This prevents conflicts when multiple agents work in parallel and makes cleanup straightforward.

### Process Lifecycle Tracking
Store PIDs, exit codes, and session info. This is critical for debugging failed executions and understanding what happened during AI agent runs.

### Agent Communication
Agents communicate via database state and event system. No direct inter-process communication needed - database acts as coordination layer.

### Resource Management
Enforce CPU/memory limits per agent. Monitor resource usage every 30s. Automatically throttle or terminate agents exceeding limits.

### Thought/Artifact Tracking
Watch task directories for Claude markdown outputs. Parse and store artifacts (code, configs, docs) in database for provenance tracking.

## Documentation References

- `docs/implementation-plan.md`: Detailed 7-phase implementation plan (19-24 days MVP)
- `docs/multi-agent-architecture.md`: Multi-agent orchestration design (Phases 8-12)
