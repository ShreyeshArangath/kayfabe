use anyhow::{bail, Context, Result};

use crate::core::{Executor, TaskManager};
use crate::db::models::TaskStatus;
use crate::db::Database;
use crate::utils::interactive;

/// Execute a task in tmux with Claude
pub async fn execute_task(
    name: Option<String>,
    command: Option<String>,
    interactive_mode: bool,
) -> Result<()> {
    // Open database connection
    let db = Database::open().context("Failed to open database")?;

    // Create task manager
    let task_manager = TaskManager::new(db);

    // Get current project
    let project = task_manager
        .get_current_project()
        .context("Failed to get current project. Run this command from within a kayfabe project directory.")?;

    // Determine task name (from argument or interactive selection)
    let task_name = match (name, interactive_mode) {
        (Some(n), _) => n,
        (None, true) => {
            // Interactive mode - select task (only pending tasks)
            let task = interactive::select_task(
                &task_manager,
                &project,
                "Select task to execute:",
                Some(TaskStatus::Pending),
            )?;
            task.name
        }
        (None, false) => {
            bail!("Task name required. Use --interactive or provide task name as argument.");
        }
    };

    // Get task
    let task = task_manager
        .get_task(&project, &task_name)?
        .ok_or_else(|| anyhow::anyhow!("Task '{}' not found", task_name))?;

    // Check task status
    if task.status == TaskStatus::Active {
        println!("⚠️  Task '{}' is already active", task_name);
        println!("   Use 'kayfabe attach {}' to attach to the running session", task_name);
        println!("   Or use 'kayfabe kill {}' to stop it first", task_name);
        return Ok(());
    }

    // Get command to execute
    let cmd = command.unwrap_or_else(|| "claude".to_string());

    println!("\n╭─────────────────────────────────────────╮");
    println!("│  Executing Task                         │");
    println!("╰─────────────────────────────────────────╯");
    println!("Task:    {}", task.name);
    println!("Command: {}", cmd);
    if let Some(desc) = &task.description {
        println!("Description: {}", desc);
    }
    println!();

    // Create executor and execute (open new DB connection)
    let executor_db = Database::open().context("Failed to open database for executor")?;
    let mut executor = Executor::new(executor_db);
    let exec = executor.execute_task(&project, &task, &cmd).await?;

    println!("\n╭─────────────────────────────────────────╮");
    println!("│  Execution Started                      │");
    println!("╰─────────────────────────────────────────╯");
    if let Some(session) = &exec.tmux_session {
        println!("Session: {}", session);
        println!("\nTo attach to the session:");
        println!("  kayfabe attach {}", task_name);
        println!("  OR");
        println!("  tmux attach -t {}", session);
        println!("\nTo stop execution:");
        println!("  kayfabe kill {}", task_name);
        println!("\nTo view logs:");
        println!("  kayfabe logs {}", task_name);
    }

    Ok(())
}
