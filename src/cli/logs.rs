use anyhow::{bail, Context, Result};

use crate::core::{Executor, TaskManager};
use crate::db::models::ProcessStatus;
use crate::db::Database;
use crate::utils::interactive;

/// Show logs for a task
pub async fn logs_task(
    name: Option<String>,
    follow: bool,
    interactive_mode: bool,
    all: bool,
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
    let task_name = match (name, interactive_mode) {
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

    // Get task
    let task = task_manager
        .get_task(&project, &task_name)?
        .ok_or_else(|| anyhow::anyhow!("Task '{}' not found", task_name))?;

    // Create executor (open new DB connection)
    let executor_db = Database::open().context("Failed to open database for executor")?;
    let executor = Executor::new(executor_db);

    if all {
        // Show all executions
        let executions = executor.list_executions(&task)?;

        if executions.is_empty() {
            println!("No execution history for task '{}'", task_name);
            return Ok(());
        }

        println!("\n╭─────────────────────────────────────────╮");
        println!("│  Execution History: {}              ", task_name);
        println!("╰─────────────────────────────────────────╯\n");

        for (i, exec) in executions.iter().enumerate() {
            println!("Execution #{}", executions.len() - i);
            println!("  Started:   {}", exec.started_at.format("%Y-%m-%d %H:%M:%S"));
            if let Some(completed) = exec.completed_at {
                println!("  Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
            }
            println!("  Status:    {:?}", exec.status);
            if let Some(code) = exec.exit_code {
                println!("  Exit Code: {}", code);
            }
            println!("  Command:   {}", exec.command);
            println!();
        }

        return Ok(());
    }

    // Get latest execution
    let exec = executor.get_logs(&task)?;

    println!("\n╭─────────────────────────────────────────╮");
    println!("│  Task Execution Logs                    │");
    println!("╰─────────────────────────────────────────╯");
    println!("Task:     {}", task.name);
    println!("Command:  {}", exec.command);
    println!("Status:   {:?}", exec.status);
    println!("Started:  {}", exec.started_at.format("%Y-%m-%d %H:%M:%S"));

    if let Some(completed) = exec.completed_at {
        println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
    }

    if let Some(code) = exec.exit_code {
        println!("Exit Code: {}", code);
    }

    if let Some(session) = &exec.tmux_session {
        println!("Session:  {}", session);
    }

    // Show stdout
    if let Some(stdout) = &exec.stdout {
        if !stdout.is_empty() {
            println!("\n─── STDOUT ───────────────────────────────");
            println!("{}", stdout);
        }
    } else {
        println!("\n─── STDOUT ───────────────────────────────");
        println!("(no output captured)");
    }

    // Show stderr
    if let Some(stderr) = &exec.stderr {
        if !stderr.is_empty() {
            println!("\n─── STDERR ───────────────────────────────");
            println!("{}", stderr);
        }
    }

    // Show follow suggestion if task is still running
    if exec.status == ProcessStatus::Running {
        println!("\n───────────────────────────────────────────");
        println!("⚠️  Task is still running. Output shown above is buffered.");
        println!("   To see live output, attach to the session:");
        println!("   kayfabe attach {}", task_name);

        if follow {
            println!("\n   Note: --follow flag is not yet implemented.");
            println!("   Use 'kayfabe attach' for live output.");
        }
    }

    println!();

    Ok(())
}
