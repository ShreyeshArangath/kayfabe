use anyhow::{bail, Context, Result};

use crate::core::TaskManager;
use crate::db::models::TaskStatus;
use crate::db::Database;
use crate::utils::interactive;

/// Kill a running task
pub async fn kill_task(name: Option<String>, interactive: bool) -> Result<()> {
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
            // Interactive mode - select task (only active tasks)
            let task = interactive::select_task(
                &task_manager,
                &project,
                "Select task to kill:",
                Some(TaskStatus::Active),
            )?;
            task.name
        }
        (None, false) => {
            bail!("Task name required. Use --interactive or provide task name as argument.");
        }
    };

    println!("💀 Killing task '{}'", task_name);
    println!("⚠️  Implementation coming in Phase 5");

    Ok(())
}
