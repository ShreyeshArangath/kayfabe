# Multi-Agent Architecture for Kayfabe

**Date**: 2026-01-23
**Extension**: Multi-Agent Orchestration Layer
**Base**: Built on top of core kayfabe implementation plan

## Vision

Extend kayfabe to orchestrate multiple AI agents working in parallel across different tasks, similar to vibe-kanban's agent ecosystem. Engineers manage a swarm of agents rather than executing tasks one-at-a-time.

## Key Capabilities

1. **Multi-Agent Execution**: Run 3-5 Claude instances simultaneously on different tasks
2. **Agent Pool Management**: Create, pause, resume, and terminate agent sessions
3. **Task Assignment**: Assign tasks to specific agents or let the scheduler decide
4. **Status Monitoring**: Real-time view of what each agent is doing
5. **Agent Communication**: Agents can signal completion, blockers, or request review
6. **Resource Limits**: CPU/memory guards, max concurrent agents
7. **Coordination Modes**:
   - **Parallel**: Independent tasks executed simultaneously
   - **Sequential**: Task chains with dependencies
   - **Collaborative**: Multiple agents on same task (review, test, implement)

## Enhanced Architecture

### New Components

```
kayfabe/
├── src/
│   ├── core/
│   │   ├── task.rs              # (Enhanced with agent assignment)
│   │   ├── agent.rs             # NEW - Agent lifecycle management
│   │   ├── scheduler.rs         # NEW - Task scheduling & agent assignment
│   │   ├── coordinator.rs       # NEW - Multi-agent coordination
│   │   ├── executor.rs          # NEW - Agent execution engine
│   │   └── monitor.rs           # NEW - Real-time agent monitoring
│   ├── tui/
│   │   ├── dashboard.rs         # NEW - Agent status dashboard
│   │   └── agent_view.rs        # NEW - Per-agent detailed view
│   └── mcp/
│       └── server.rs            # NEW - MCP integration for agent config
```

### Tech Stack Additions

- `tokio` - Async runtime for concurrent agent management
- `futures` - Agent coordination primitives
- `notify` - File system watching for agent outputs
- `sysinfo` - Resource monitoring (CPU, memory)

## Enhanced Database Schema

### New Tables

```sql
-- Agent pool tracking
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,              -- e.g., "claude-1", "claude-2"
    status TEXT NOT NULL              -- idle, active, paused, terminated
        CHECK(status IN ('idle', 'active', 'paused', 'terminated')),
    current_task_id TEXT,             -- NULL if idle
    tmux_session TEXT,                -- tmux session identifier
    pid INTEGER,                      -- Process ID
    started_at TEXT,
    last_heartbeat TEXT,              -- Last activity timestamp
    cpu_usage REAL,                   -- Percentage
    memory_usage INTEGER,             -- MB
    FOREIGN KEY (current_task_id) REFERENCES tasks(id)
);

-- Task dependencies for sequential execution
CREATE TABLE task_dependencies (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    depends_on_task_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    FOREIGN KEY (depends_on_task_id) REFERENCES tasks(id),
    UNIQUE(task_id, depends_on_task_id)
);

-- Agent activity log
CREATE TABLE agent_events (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    event_type TEXT NOT NULL         -- started, task_assigned, completed, error, paused
        CHECK(event_type IN ('started', 'task_assigned', 'completed', 'error', 'paused', 'resumed', 'terminated')),
    task_id TEXT,
    message TEXT,
    metadata TEXT,                    -- JSON for additional context
    created_at TEXT NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Resource usage snapshots
CREATE TABLE resource_snapshots (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    cpu_percent REAL,
    memory_mb INTEGER,
    disk_io_read INTEGER,
    disk_io_write INTEGER,
    recorded_at TEXT NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);
```

### Enhanced Existing Tables

```sql
-- Add to tasks table
ALTER TABLE tasks ADD COLUMN assigned_agent_id TEXT REFERENCES agents(id);
ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 0;
ALTER TABLE tasks ADD COLUMN execution_mode TEXT DEFAULT 'standard'
    CHECK(execution_mode IN ('standard', 'review', 'test', 'deploy'));
ALTER TABLE tasks ADD COLUMN estimated_duration INTEGER; -- minutes
ALTER TABLE tasks ADD COLUMN actual_duration INTEGER;    -- minutes
```

## Agent Lifecycle

### 1. Agent Creation

```rust
pub struct Agent {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub current_task: Option<Task>,
    pub tmux_session: String,
    pub pid: u32,
    pub resource_limits: ResourceLimits,
}

pub enum AgentStatus {
    Idle,           // Waiting for task assignment
    Active,         // Currently executing a task
    Paused,         // Suspended by user
    Terminated,     // Shut down
}

impl Agent {
    pub async fn spawn(config: AgentConfig) -> Result<Agent>;
    pub async fn assign_task(&mut self, task: Task) -> Result<()>;
    pub async fn pause(&mut self) -> Result<()>;
    pub async fn resume(&mut self) -> Result<()>;
    pub async fn terminate(&mut self) -> Result<()>;
    pub fn check_health(&self) -> HealthStatus;
}
```

### 2. Scheduler

```rust
pub struct Scheduler {
    agents: Vec<Agent>,
    task_queue: VecDeque<Task>,
    strategy: SchedulingStrategy,
}

pub enum SchedulingStrategy {
    RoundRobin,           // Distribute evenly
    PriorityBased,        // High priority tasks first
    LoadBalanced,         // Consider agent resource usage
    Manual,               // User assigns tasks explicitly
}

impl Scheduler {
    pub fn assign_task(&mut self, task: Task) -> Result<String>; // Returns agent_id
    pub fn get_next_task(&mut self) -> Option<Task>;
    pub fn rebalance(&mut self) -> Result<()>;
    pub fn handle_completion(&mut self, agent_id: String, task_id: String) -> Result<()>;
}
```

### 3. Coordinator

```rust
pub struct Coordinator {
    scheduler: Scheduler,
    executors: HashMap<String, AgentExecutor>,
}

pub enum CoordinationMode {
    Parallel,              // All tasks run independently
    Sequential,            // Wait for dependencies
    Collaborative(Vec<AgentRole>), // Multiple agents on one task
}

pub enum AgentRole {
    Implementer,
    Reviewer,
    Tester,
}

impl Coordinator {
    pub async fn orchestrate(&mut self, tasks: Vec<Task>, mode: CoordinationMode) -> Result<()>;
    pub async fn start_agents(&mut self, count: usize) -> Result<()>;
    pub async fn stop_agents(&mut self, agent_ids: Vec<String>) -> Result<()>;
    pub async fn monitor_progress(&self) -> ProgressReport;
}
```

## CLI Commands - Multi-Agent Extension

### Agent Management

```bash
# Start agent pool
kayfabe agents start --count 3

# List all agents
kayfabe agents list
# Output:
# AGENT      STATUS    TASK         UPTIME   CPU    MEM
# claude-1   active    feat-auth    2h 15m   45%    800MB
# claude-2   idle      -            2h 10m   5%     200MB
# claude-3   active    fix-bug-123  1h 45m   60%    950MB

# View agent details
kayfabe agents show claude-1

# Pause/resume agent
kayfabe agents pause claude-1
kayfabe agents resume claude-1

# Stop agents
kayfabe agents stop claude-1
kayfabe agents stop --all

# Agent logs
kayfabe agents logs claude-1 --follow
```

### Task Assignment

```bash
# Auto-assign to next available agent
kayfabe add "feature-x" --auto-assign

# Assign to specific agent
kayfabe add "feature-y" --assign claude-2

# Reassign task
kayfabe task reassign feature-x claude-3

# Execute with multiple agents (collaborative)
kayfabe execute feature-x --agents claude-1,claude-2 --roles implement,review

# Sequential execution with dependencies
kayfabe add "tests" --depends-on feature-x
kayfabe add "deploy" --depends-on tests

# Parallel batch execution
kayfabe execute --batch task-1,task-2,task-3 --mode parallel
```

### Monitoring Dashboard

```bash
# Launch real-time dashboard
kayfabe dashboard

# ASCII Dashboard Layout:
┌─────────────────── Kayfabe Dashboard ────────────────────┐
│ Project: myproject              Agents: 3/5              │
│ Active Tasks: 4/12              Completed Today: 7       │
├──────────────────────────────────────────────────────────┤
│ AGENT      STATUS    TASK            PROGRESS   RESOURCE │
│ claude-1   🟢 Active feat-auth       65%       ▓▓▓▒░ 45% │
│ claude-2   🔵 Idle   -               -         ▓░░░░  5% │
│ claude-3   🟢 Active fix-bug-123     80%       ▓▓▓▓░ 60% │
├──────────────────────────────────────────────────────────┤
│ QUEUE                                                     │
│ 1. refactor-db (Priority: High)                          │
│ 2. update-docs (Priority: Low)                           │
│ 3. add-tests (Blocked by: feat-auth)                     │
└──────────────────────────────────────────────────────────┘
```

## Agent Execution Patterns

### Pattern 1: Parallel Independent Tasks

```bash
# Scenario: Multiple features in parallel
kayfabe agents start --count 3

kayfabe add "feature-auth" --priority high
kayfabe add "feature-payments" --priority high
kayfabe add "refactor-db" --priority medium

# Scheduler auto-assigns based on priority
kayfabe execute --auto --mode parallel

# All three start immediately on separate agents
```

### Pattern 2: Sequential Pipeline

```bash
# Scenario: Implementation → Testing → Deployment
kayfabe add "implement-feature"
kayfabe add "write-tests" --depends-on implement-feature
kayfabe add "deploy-staging" --depends-on write-tests

kayfabe agents start --count 1
kayfabe execute --auto --mode sequential

# Agents work through the chain automatically
```

### Pattern 3: Collaborative Multi-Agent

```bash
# Scenario: One task, multiple specialized agents
kayfabe add "complex-refactor"

kayfabe execute complex-refactor \
    --agents claude-1,claude-2,claude-3 \
    --roles implementer,reviewer,tester \
    --mode collaborative

# claude-1: Makes code changes
# claude-2: Reviews changes in real-time
# claude-3: Writes tests
# All work in same worktree with file watching
```

## TUI Enhancements

### Enhanced Dashboard View

```rust
pub struct DashboardView {
    agent_panel: AgentPanel,      // Top - Agent status grid
    task_panel: TaskPanel,        // Middle - Task queue
    detail_panel: DetailPanel,    // Bottom - Selected agent details
    log_panel: LogPanel,          // Right - Live event log
}

pub struct AgentPanel {
    agents: Vec<AgentWidget>,
    // Shows: name, status indicator, current task, resource bars
}

pub struct TaskPanel {
    tasks: Vec<TaskWidget>,
    // Shows: name, status, assigned agent, progress, dependencies
}
```

### Keyboard Shortcuts

```
Tab     - Switch between panels
↑/↓     - Navigate items
Enter   - View details
a       - Create new agent
k       - Kill selected agent
p       - Pause/resume agent
t       - Assign task
Space   - Start/stop task
d       - View dependencies
l       - View logs
q       - Quit
```

## Agent Communication Protocol

### Event System

```rust
pub enum AgentEvent {
    TaskStarted { task_id: String },
    ProgressUpdate { task_id: String, percent: u8, message: String },
    TaskCompleted { task_id: String, duration: Duration },
    ErrorEncountered { task_id: String, error: String },
    BlockerFound { task_id: String, blocker: String },
    ReviewRequested { task_id: String, files: Vec<String> },
    ResourceWarning { agent_id: String, resource_type: ResourceType },
}

pub trait EventEmitter {
    fn emit(&self, event: AgentEvent) -> Result<()>;
}
```

### Inter-Agent Messaging

```rust
pub struct Message {
    from_agent: String,
    to_agent: Option<String>,  // None = broadcast
    message_type: MessageType,
    content: String,
    timestamp: DateTime<Utc>,
}

pub enum MessageType {
    Completion,      // "Task X is done"
    Blocker,         // "Need review on file Y"
    Request,         // "Can someone handle task Z?"
    Status,          // "I'm at 70% on feature A"
}
```

## Resource Management

### Limits & Monitoring

```rust
pub struct ResourceLimits {
    max_cpu_percent: f32,      // e.g., 80%
    max_memory_mb: usize,      // e.g., 2048
    max_duration_minutes: u64, // e.g., 240 (4 hours)
    max_concurrent_agents: usize, // e.g., 5
}

pub struct ResourceMonitor {
    limits: ResourceLimits,
    snapshots: Vec<ResourceSnapshot>,
}

impl ResourceMonitor {
    pub fn check_agent(&self, agent: &Agent) -> ResourceStatus;
    pub fn enforce_limits(&mut self) -> Result<Vec<AgentAction>>;
    // Returns actions like: [ThrottleAgent(id), TerminateAgent(id)]
}
```

## Integration Points

### 1. MCP Server Integration

```rust
// Serve agent status via MCP
pub struct KayfabeMCPServer {
    coordinator: Arc<Mutex<Coordinator>>,
}

impl MCPServer for KayfabeMCPServer {
    fn get_resource(&self, uri: &str) -> MCPResource {
        match uri {
            "agents" => self.list_agents(),
            "tasks" => self.list_tasks(),
            "status" => self.get_status(),
            _ => MCPResource::NotFound,
        }
    }
}
```

### 2. File System Watching

```rust
// Watch for agent output files
pub struct OutputWatcher {
    watcher: RecommendedWatcher,
    tx: mpsc::Sender<FileEvent>,
}

impl OutputWatcher {
    pub fn watch_task_dir(&mut self, task_id: &str) -> Result<()>;
    // Watches thoughts/<task_id>/ for new artifacts
    // Parses Claude's markdown outputs
    // Updates database with artifacts
}
```

### 3. Git Integration

```rust
// Enhanced worktree manager for multi-agent
impl WorktreeManager {
    pub fn create_shared_worktree(&self, task_id: &str, agents: Vec<String>) -> Result<()>;
    // For collaborative mode - multiple agents, same worktree
    // Uses git locks to prevent conflicts

    pub fn create_agent_worktrees(&self, task_id: &str, agents: Vec<String>) -> Result<HashMap<String, PathBuf>>;
    // For parallel subtasks - each agent gets their own worktree
}
```

## Implementation Phases - Multi-Agent Extension

### Phase 8: Agent Foundation (3-4 days)

**Goal**: Basic agent lifecycle and database

**Tasks**:
- Create agent database schema
- Implement Agent struct and lifecycle methods
- Build agent spawning with tmux
- Add agent CRUD operations
- Create basic agent CLI commands

**Success Criteria**:
- [ ] Can spawn multiple agent processes
- [ ] Agents tracked in database
- [ ] `kayfabe agents list` works
- [ ] Can pause/resume/terminate agents

### Phase 9: Scheduler & Coordinator (4-5 days)

**Goal**: Task assignment and coordination logic

**Tasks**:
- Implement Scheduler with multiple strategies
- Build Coordinator for multi-agent orchestration
- Add task dependency tracking
- Implement auto-assignment logic
- Create priority queue for tasks

**Success Criteria**:
- [ ] Scheduler assigns tasks to idle agents
- [ ] Dependencies respected in sequential mode
- [ ] Parallel mode works with independent tasks
- [ ] Priority-based assignment functional

### Phase 10: Enhanced TUI Dashboard (3-4 days)

**Goal**: Real-time multi-agent dashboard

**Tasks**:
- Build agent status panel
- Add task queue visualization
- Implement progress bars and resource meters
- Create detailed agent view
- Add live event log stream

**Success Criteria**:
- [ ] Dashboard shows all agents in real-time
- [ ] Can navigate between agents
- [ ] Resource usage visible
- [ ] Event log updates live

### Phase 11: Monitoring & Health (2-3 days)

**Goal**: Resource monitoring and limits

**Tasks**:
- Implement ResourceMonitor
- Add CPU/memory tracking per agent
- Build health check system
- Create resource limit enforcement
- Add agent heartbeat mechanism

**Success Criteria**:
- [ ] Resource usage tracked every 30s
- [ ] Limits enforced automatically
- [ ] Unhealthy agents detected
- [ ] Alerts on resource warnings

### Phase 12: Advanced Features (3-4 days)

**Goal**: Collaborative mode, messaging, MCP

**Tasks**:
- Implement inter-agent messaging
- Build collaborative execution mode
- Add MCP server for agent status
- Create file watching for agent outputs
- Implement shared worktree management

**Success Criteria**:
- [ ] Agents can communicate
- [ ] Collaborative mode assigns roles correctly
- [ ] MCP server exposes agent data
- [ ] File watcher detects Claude outputs

## Configuration

### kayfabe.toml

```toml
[agents]
max_concurrent = 5
default_count = 2
auto_assign = true
scheduling_strategy = "priority"  # roundrobin, priority, loadbalanced, manual

[resource_limits]
max_cpu_percent = 80.0
max_memory_mb = 2048
max_duration_minutes = 240

[execution]
default_mode = "parallel"  # parallel, sequential
enable_collaborative = true

[monitoring]
snapshot_interval_secs = 30
enable_heartbeat = true
heartbeat_interval_secs = 60

[tmux]
session_prefix = "kayfabe"
default_shell = "/bin/zsh"
enable_logging = true
```

## Benefits Over Single-Agent Execution

1. **Parallelism**: Work on 3-5 features simultaneously
2. **Efficiency**: Idle agents auto-pick next tasks
3. **Specialization**: Assign agents specific roles (implement/review/test)
4. **Visibility**: See all work in progress at a glance
5. **Resource Awareness**: Prevent system overload
6. **Coordination**: Dependencies handled automatically
7. **Scalability**: Add more agents as needed

## Example Workflow

```bash
# Morning: Start the day
kayfabe agents start --count 3
kayfabe dashboard

# Add today's tasks
kayfabe add "feature-notifications" --priority high
kayfabe add "fix-login-bug" --priority critical
kayfabe add "refactor-api" --priority medium
kayfabe add "update-docs" --priority low --depends-on refactor-api

# Auto-execute with priority scheduling
kayfabe execute --auto --mode parallel

# Dashboard shows:
# - claude-1: Working on fix-login-bug (critical)
# - claude-2: Working on feature-notifications (high)
# - claude-3: Working on refactor-api (medium)
# - Queue: update-docs (waiting for refactor-api)

# Check on specific agent
kayfabe agents show claude-1
kayfabe agents logs claude-1 --tail 50

# When refactor-api completes, claude-3 auto-picks update-docs
# End of day: Review completions
kayfabe status
kayfabe agents stop --all
```

## Questions for Consideration

1. **Agent Naming**: Auto-generate (claude-1, claude-2) or allow custom names?
2. **Claude Invocation**: How to spawn multiple Claude instances? Different config files?
3. **Communication**: Should agents actively communicate or just via database state?
4. **Failure Handling**: Auto-restart failed agents? Reassign their tasks?
5. **Cost Awareness**: Track API usage per agent? Set spending limits?
6. **Agent Personas**: Allow configuring different agent "personalities" or specializations?
7. **Workspace Isolation**: Always separate worktrees or allow shared for collaborative?

## Success Metrics

### Performance
- Support 5 concurrent agents without degradation
- <100ms scheduler assignment time
- <50ms dashboard refresh rate
- <500MB overhead per agent

### Functionality
- All coordination modes work correctly
- Dependencies respected 100% of time
- No race conditions in task assignment
- Resource limits enforced accurately

### User Experience
- Clear visibility into agent status
- Intuitive dashboard navigation
- Helpful event notifications
- Smooth multi-agent workflows

## Next Steps

1. Review and refine this multi-agent architecture
2. Decide on phasing - build core first or integrate during initial phases?
3. Answer open questions above
4. Begin Phase 8 implementation after core MVP is complete
5. Test with real multi-task workflows
