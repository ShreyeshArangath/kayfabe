use anyhow::{Context, Result};

use crate::core::TaskManager;
use crate::db::Database;

/// Add a new task with worktree
pub async fn add_task(
    name: String,
    description: Option<String>,
    _priority: Option<String>,
    _auto_assign: bool,
) -> Result<()> {
    // Open database connection
    let db = Database::open().context("Failed to open database")?;

    // Create task manager
    let task_manager = TaskManager::new(db);

    // Get current project
    let project = task_manager
        .get_current_project()
        .context("Failed to get current project. Run this command from within a kayfabe project directory.")?;

    // Create task
    let task = task_manager
        .create_task(&project, &name, description.as_deref())
        .context("Failed to create task")?;

    println!("\n✓ Task '{}' created successfully", task.name);
    println!("  ID: {}", task.id);
    println!("  Status: {:?}", task.status);

    if let Some(worktree_path) = &task.worktree_path {
        println!("  Worktree: {}", worktree_path.display());
    }

    if let Some(branch) = &task.git_branch {
        println!("  Branch: {}", branch);
    }

    println!("\nNext steps:");
    println!("  • View tasks: kayfabe list");
    println!("  • Execute task: kayfabe execute {}", name);
    println!("  • Navigate to worktree: cd {}", task.worktree_path.unwrap().display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_task_requires_project() {
        // This should fail when not in a project directory
        let result = add_task(
            "test-task".to_string(),
            Some("Test".to_string()),
            None,
            false,
        )
        .await;

        assert!(result.is_err());
    }
}
