use anyhow::{bail, Context, Result};

use crate::core::TaskManager;
use crate::db::Database;
use crate::utils::interactive;

/// Remove a task with comprehensive cleanup
pub async fn remove_task(name: Option<String>, force: bool, interactive: bool) -> Result<()> {
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
            // Interactive mode - select task
            let task = interactive::select_task(
                &task_manager,
                &project,
                "Select task to remove:",
                None,
            )?;
            task.name
        }
        (None, false) => {
            bail!("Task name required. Use --interactive or provide task name as argument.");
        }
    };

    // Get task to check if it exists
    let task = task_manager
        .get_task(&project, &task_name)?
        .ok_or_else(|| anyhow::anyhow!("Task '{}' not found", task_name))?;

    // Confirmation prompt if not forced
    if !force {
        let mut confirm_msg = format!("Are you sure you want to remove task '{}'?", task_name);

        if let Some(worktree_path) = &task.worktree_path {
            confirm_msg.push_str(&format!("\nWorktree: {}", worktree_path.display()));
        }

        confirm_msg.push_str("\nThis action cannot be undone.");

        if !interactive::confirm(&confirm_msg, false)? {
            println!("❌ Task removal cancelled.");
            return Ok(());
        }
    }

    // Remove task
    task_manager
        .remove_task(&project, &task_name, force)
        .context("Failed to remove task")?;

    println!("\n✓ Task '{}' removed successfully", task_name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remove_task_requires_project() {
        // This should fail when not in a project directory
        let result = remove_task(Some("test-task".to_string()), true, false).await;
        assert!(result.is_err());
    }
}
