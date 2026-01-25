use anyhow::{bail, Context, Result};

use crate::core::{Executor, TaskManager};
use crate::db::models::TaskStatus;
use crate::db::Database;
use crate::utils::interactive;

/// Attach to a running task's tmux session
pub async fn attach_task(name: Option<String>, interactive_mode: bool) -> Result<()> {
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
            // Interactive mode - select task (only active tasks)
            let task = interactive::select_task(
                &task_manager,
                &project,
                "Select task to attach to:",
                Some(TaskStatus::Active),
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

    // Verify task is active
    if task.status != TaskStatus::Active {
        bail!(
            "Task '{}' is not active (status: {:?}). Use 'kayfabe execute {}' to start it.",
            task_name,
            task.status,
            task_name
        );
    }

    println!("🔗 Attaching to task '{}'", task_name);
    println!("   Press Ctrl+B then D to detach from session\n");

    // Create executor and attach (open new DB connection)
    let executor_db = Database::open().context("Failed to open database for executor")?;
    let executor = Executor::new(executor_db);
    executor.attach_task(&task)?;

    Ok(())
}
