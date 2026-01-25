use anyhow::{bail, Context, Result};

use crate::core::TaskManager;
use crate::db::models::TaskStatus;
use crate::db::Database;
use crate::utils::interactive;

/// Execute a task in tmux with Claude
pub async fn execute_task(
    name: Option<String>,
    command: Option<String>,
    interactive: bool,
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
    let task_name = match (name, interactive) {
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

    let cmd = command.unwrap_or_else(|| "claude".to_string());
    println!("▶️  Executing task '{}' with command: {}", task_name, cmd);
    println!("⚠️  Implementation coming in Phase 5");

    Ok(())
}
