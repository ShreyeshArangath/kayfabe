use anyhow::{Context, Result};
use serde_json::json;

use crate::core::TaskManager;
use crate::db::{Database, operations::tasks};
use crate::utils::interactive::{wizard_add_task, SUCCESS_COLOR, RESET_COLOR};

/// Add a new task with worktree
pub async fn add_task(
    name: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    auto_assign: bool,
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

    // Determine mode: use wizard if interactive flag is set OR if name is not provided
    let use_wizard = interactive || name.is_none();

    let (task_name, task_description, task_priority, task_auto_assign) = if use_wizard {
        // Run wizard to collect inputs
        wizard_add_task(&task_manager, &project)?
    } else {
        // Use provided CLI arguments (direct mode)
        let name = name.expect("Name required if not interactive");

        // Validate name doesn't exist (wizard does this automatically)
        if tasks::name_exists(task_manager.db.conn(), &project.id, &name)? {
            anyhow::bail!("Task '{}' already exists", name);
        }

        (
            name,
            description,
            priority.unwrap_or_else(|| "medium".to_string()),
            auto_assign,
        )
    };

    // Create task
    let mut task = task_manager
        .create_task(&project, &task_name, task_description.as_deref())
        .context("Failed to create task")?;

    // Store priority and auto_assign in metadata_json if not default
    if task_priority != "medium" || task_auto_assign {
        let metadata = json!({
            "priority": task_priority,
            "auto_assign": task_auto_assign,
        });
        task.metadata_json = Some(metadata.to_string());
        tasks::update(task_manager.db.conn(), &task)
            .context("Failed to update task metadata")?;
    }

    // Print success message with priority indicator
    let priority_icon = match task_priority.as_str() {
        "high" => "🔴",
        "low" => "🟢",
        _ => "🟡",
    };

    println!("\n{}✓ Task '{}' created successfully{}", SUCCESS_COLOR, task.name, RESET_COLOR);
    println!("  ID: {}", task.id);
    println!("  Status: {:?}", task.status);
    println!("  Priority: {} {}", priority_icon, task_priority);

    if let Some(worktree_path) = &task.worktree_path {
        println!("  Worktree: {}", worktree_path.display());
    }

    if let Some(branch) = &task.git_branch {
        println!("  Branch: {}", branch);
    }

    if task_auto_assign {
        println!("  Auto-assign: enabled");
    }

    println!("\nNext steps:");
    println!("  • View tasks: kayfabe list");
    println!("  • Execute task: kayfabe execute {}", task_name);
    if let Some(worktree_path) = &task.worktree_path {
        println!("  • Navigate to worktree: cd {}", worktree_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_task_requires_project() {
        // This should fail when not in a project directory
        let result = add_task(
            Some("test-task".to_string()),
            Some("Test".to_string()),
            None,
            false,
            false, // not interactive
        )
        .await;

        assert!(result.is_err());
    }
}
