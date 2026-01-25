use anyhow::{bail, Context, Result};

use crate::core::TaskManager;
use crate::db::Database;
use crate::utils::interactive;

/// Show logs for a task
pub async fn logs_task(
    name: Option<String>,
    follow: bool,
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
            // Interactive mode - select task (all tasks)
            let task = interactive::select_task(
                &task_manager,
                &project,
                "Select task to view logs:",
                None,
            )?;
            task.name
        }
        (None, false) => {
            bail!("Task name required. Use --interactive or provide task name as argument.");
        }
    };

    println!("📜 Showing logs for task '{}'", task_name);
    if follow {
        println!("   Following: enabled");
    }
    println!("⚠️  Implementation coming in Phase 5");

    Ok(())
}
