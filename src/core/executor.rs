use anyhow::{anyhow, bail, Context, Result};
use tokio::process::Command;

use crate::db::models::{ExecutionProcess, ProcessStatus, Project, Task, TaskStatus};
use crate::db::operations::{execution_processes, tasks};
use crate::db::Database;
use crate::utils::TmuxManager;

use super::process_monitor;

/// Process executor for running commands in tmux sessions
pub struct Executor {
    db: Database,
    tmux: TmuxManager,
}

impl Executor {
    /// Create a new executor
    pub fn new(db: Database) -> Self {
        Self {
            db,
            tmux: TmuxManager::new(),
        }
    }

    /// Execute a task with a command in a tmux session
    ///
    /// This performs:
    /// 1. Validates task is not already running
    /// 2. Creates/attaches tmux session
    /// 3. Spawns command process
    /// 4. Sets up output capture
    /// 5. Updates task status to Active
    /// 6. Returns execution details
    pub async fn execute_task(
        &mut self,
        _project: &Project,
        task: &Task,
        command: &str,
    ) -> Result<ExecutionProcess> {
        // Validate task is not already running
        if let Some(exec) = execution_processes::get_active_by_task_id(self.db.conn(), &task.id)? {
            bail!(
                "Task '{}' is already running in session: {}",
                task.name,
                exec.tmux_session.unwrap_or_else(|| "unknown".to_string())
            );
        }

        // Validate task has a worktree
        let worktree_path = task
            .worktree_path
            .as_ref()
            .ok_or_else(|| anyhow!("Task '{}' has no worktree", task.name))?;

        if !worktree_path.exists() {
            bail!(
                "Task worktree does not exist: {}",
                worktree_path.display()
            );
        }

        // Create tmux session
        let session_name = self.format_session_name(&task.name);
        self.tmux
            .create_or_attach_session(&session_name, worktree_path)
            .context("Failed to create tmux session")?;

        println!(
            "✓ Created tmux session: {} in {}",
            session_name,
            worktree_path.display()
        );

        // Parse and prepare command
        let (program, args) = self.parse_command(command)?;

        // Spawn process with output capture
        let child = Command::new(&program)
            .args(&args)
            .current_dir(worktree_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn command")?;

        println!("✓ Spawned process: {}", command);

        // Create execution record
        let exec = ExecutionProcess::new(
            task.id.clone(),
            command.to_string(),
            Some(session_name.clone()),
        );

        execution_processes::insert(self.db.conn(), &exec)
            .context("Failed to insert execution record")?;

        // Update task status to Active
        let mut updated_task = task.clone();
        updated_task.status = TaskStatus::Active;
        updated_task.last_accessed_at = Some(chrono::Utc::now());

        tasks::update(self.db.conn(), &updated_task)
            .context("Failed to update task status")?;

        println!("✓ Task '{}' is now Active", task.name);

        // Spawn background monitor for output capture
        // Note: We open a new database connection for the background task
        // to avoid shared mutable access issues
        let exec_id = exec.id.clone();
        let task_id = task.id.clone();
        let monitor_db = Database::open().context("Failed to open database for monitor")?;

        process_monitor::spawn_monitor(exec_id, task_id, child, monitor_db);

        println!(
            "✓ Background monitor started for output capture"
        );

        Ok(exec)
    }

    /// Kill a running task execution
    pub async fn kill_task(&self, task: &Task) -> Result<()> {
        // Get active execution
        let exec = execution_processes::get_active_by_task_id(self.db.conn(), &task.id)?
            .ok_or_else(|| anyhow!("Task '{}' is not running", task.name))?;

        // Kill tmux session (which kills all processes in it)
        if let Some(session_name) = &exec.tmux_session {
            self.tmux
                .kill_session(session_name)
                .context("Failed to kill tmux session")?;

            println!("✓ Killed tmux session: {}", session_name);
        }

        // Update execution record
        let mut updated_exec = exec;
        updated_exec.status = ProcessStatus::Killed;
        updated_exec.completed_at = Some(chrono::Utc::now());

        execution_processes::update(self.db.conn(), &updated_exec)
            .context("Failed to update execution record")?;

        // Update task status back to Pending
        let mut updated_task = task.clone();
        updated_task.status = TaskStatus::Pending;
        updated_task.updated_at = chrono::Utc::now();

        tasks::update(self.db.conn(), &updated_task)
            .context("Failed to update task status")?;

        println!("✓ Task '{}' status reset to Pending", task.name);

        Ok(())
    }

    /// Attach to a running task's tmux session
    pub fn attach_task(&self, task: &Task) -> Result<()> {
        // Get active execution
        let exec = execution_processes::get_active_by_task_id(self.db.conn(), &task.id)?
            .ok_or_else(|| anyhow!("Task '{}' is not running", task.name))?;

        let session_name = exec
            .tmux_session
            .ok_or_else(|| anyhow!("No tmux session found for task"))?;

        println!("Attaching to tmux session: {}", session_name);
        println!("Press Ctrl+B then D to detach from session");

        // Update task last accessed time
        tasks::touch(self.db.conn(), &task.id)?;

        // This is a blocking call that takes over the terminal
        self.tmux.attach_session(&session_name)?;

        Ok(())
    }

    /// Get execution logs for a task
    pub fn get_logs(&self, task: &Task) -> Result<ExecutionProcess> {
        // Get latest execution (regardless of status)
        execution_processes::get_latest_by_task_id(self.db.conn(), &task.id)?
            .ok_or_else(|| anyhow!("No execution history found for task '{}'", task.name))
    }

    /// List all executions for a task
    pub fn list_executions(&self, task: &Task) -> Result<Vec<ExecutionProcess>> {
        execution_processes::list_by_task_id(self.db.conn(), &task.id)
    }

    /// Format a session name for a task
    fn format_session_name(&self, task_name: &str) -> String {
        format!("kayfabe-{}", sanitize_session_name(task_name))
    }

    /// Parse a command string into program and arguments
    ///
    /// Handles simple shell-style quoting
    fn parse_command(&self, command: &str) -> Result<(String, Vec<String>)> {
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            bail!("Empty command");
        }

        let program = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        Ok((program, args))
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new(Database::open().expect("Failed to open database"))
    }
}

/// Sanitize a task name for use in tmux session names
///
/// Removes special characters and replaces spaces with dashes
fn sanitize_session_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_session_name() {
        assert_eq!(sanitize_session_name("simple"), "simple");
        assert_eq!(sanitize_session_name("with spaces"), "with-spaces");
        assert_eq!(sanitize_session_name("with-dashes"), "with-dashes");
        assert_eq!(sanitize_session_name("Feature/Branch"), "feature_branch");
        assert_eq!(sanitize_session_name("test@123"), "test_123");
    }

    #[test]
    fn test_parse_command() {
        let db = Database::open_in_memory().unwrap();
        let executor = Executor::new(db);

        let (prog, args) = executor.parse_command("ls -la /tmp").unwrap();
        assert_eq!(prog, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);

        let (prog, args) = executor.parse_command("echo hello").unwrap();
        assert_eq!(prog, "echo");
        assert_eq!(args, vec!["hello"]);

        let (prog, args) = executor.parse_command("claude").unwrap();
        assert_eq!(prog, "claude");
        assert!(args.is_empty());
    }

    #[test]
    fn test_format_session_name() {
        let db = Database::open_in_memory().unwrap();
        let executor = Executor::new(db);

        assert_eq!(
            executor.format_session_name("test-task"),
            "kayfabe-test-task"
        );
        assert_eq!(
            executor.format_session_name("My Task"),
            "kayfabe-my-task"
        );
    }
}
