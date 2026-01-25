use anyhow::{bail, Context, Result};

use crate::core::{Executor, TaskManager};
use crate::db::models::TaskStatus;
use crate::db::Database;
use crate::utils::interactive;

/// Kill a running task
pub async fn kill_task(name: Option<String>, interactive_mode: bool, force: bool) -> Result<()> {
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
                "Select task to kill:",
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
        if !force {
            bail!(
                "Task '{}' is not active (status: {:?}). Nothing to kill.",
                task_name,
                task.status
            );
        } else {
            println!("⚠️  Task '{}' is not active, but --force was specified", task_name);
        }
    }

    // Confirm kill operation if not forced
    if !force {
        println!("⚠️  This will terminate the running process and tmux session for task '{}'", task_name);
        print!("   Continue? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    println!("💀 Killing task '{}'...", task_name);

    // Create executor and kill task (open new DB connection)
    let executor_db = Database::open().context("Failed to open database for executor")?;
    let executor = Executor::new(executor_db);
    executor.kill_task(&task).await?;

    println!("\n✓ Task '{}' has been stopped", task_name);
    println!("  Use 'kayfabe execute {}' to start it again", task_name);

    Ok(())
}
