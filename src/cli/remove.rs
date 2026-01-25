use anyhow::{Context, Result};
use std::io::{self, Write};

use crate::core::TaskManager;
use crate::db::Database;

/// Remove a task with comprehensive cleanup
pub async fn remove_task(name: String, force: bool) -> Result<()> {
    // Open database connection
    let db = Database::open().context("Failed to open database")?;

    // Create task manager
    let task_manager = TaskManager::new(db);

    // Get current project
    let project = task_manager
        .get_current_project()
        .context("Failed to get current project. Run this command from within a kayfabe project directory.")?;

    // Get task to check if it exists
    let task = task_manager
        .get_task(&project, &name)?
        .ok_or_else(|| anyhow::anyhow!("Task '{}' not found", name))?;

    // Confirmation prompt if not forced
    if !force {
        print!("⚠️  Are you sure you want to remove task '{}'? ", name);

        if let Some(worktree_path) = &task.worktree_path {
            print!("\n   Worktree: {}", worktree_path.display());
        }

        print!("\n   This action cannot be undone. Continue? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("❌ Task removal cancelled.");
            return Ok(());
        }
    }

    // Remove task
    task_manager
        .remove_task(&project, &name, force)
        .context("Failed to remove task")?;

    println!("\n✓ Task '{}' removed successfully", name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remove_task_requires_project() {
        // This should fail when not in a project directory
        let result = remove_task("test-task".to_string(), true).await;
        assert!(result.is_err());
    }
}
