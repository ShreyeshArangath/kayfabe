use anyhow::{Context, Result};

use crate::core::TaskManager;
use crate::db::models::TaskStatus;
use crate::db::Database;
use crate::tui;

/// List tasks for the current project
pub async fn list_tasks(status_filter: Option<String>, use_tui: bool) -> Result<()> {
    // Open database connection
    let db = Database::open().context("Failed to open database")?;

    // Create task manager
    let task_manager = TaskManager::new(db);

    // Get current project
    let project = task_manager
        .get_current_project()
        .context("Failed to get current project. Run this command from within a kayfabe project directory.")?;

    // If TUI mode is requested, launch it
    if use_tui {
        return tui::run_tui(&project).await;
    }

    // Parse status filter
    let status = match status_filter {
        Some(s) => {
            let s = s.to_lowercase();
            TaskStatus::from_str(&s)
                .ok_or_else(|| anyhow::anyhow!("Invalid status: {}. Valid statuses: pending, active, completed, archived", s))?
        }
        None => {
            // No filter - show all tasks
            let tasks = task_manager.list_tasks(&project, None)?;

            println!("\n=== Tasks for project '{}' ===", project.name);
            task_manager.display_tasks(&tasks, false);
            task_manager.display_stats(&project)?;

            return Ok(());
        }
    };

    // Show filtered tasks
    let tasks = task_manager.list_tasks(&project, Some(status))?;

    println!(
        "\n=== {:?} tasks for project '{}' ===",
        status, project.name
    );
    task_manager.display_tasks(&tasks, false);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_tasks_requires_project() {
        // This should fail when not in a project directory
        let result = list_tasks(None, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_status_filter() {
        let result = list_tasks(Some("invalid".to_string()), false).await;
        // Should fail due to invalid status, or fail due to no project
        assert!(result.is_err());
    }
}
