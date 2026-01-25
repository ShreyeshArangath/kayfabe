use anyhow::{Context, Result};
use inquire::Select;

use crate::core::TaskManager;
use crate::db::models::{Project, Task, TaskStatus};

/// Interactive task selection with search and full display
pub fn select_task(
    task_manager: &TaskManager,
    project: &Project,
    prompt: &str,
    status_filter: Option<TaskStatus>,
) -> Result<Task> {
    // Get tasks based on filter
    let tasks = task_manager.list_tasks(project, status_filter)?;

    if tasks.is_empty() {
        let filter_msg = match status_filter {
            Some(status) => format!(" with status {:?}", status),
            None => String::new(),
        };
        anyhow::bail!("No tasks found{}", filter_msg);
    }

    // Create display strings for tasks
    let task_displays: Vec<String> = tasks
        .iter()
        .map(|task| {
            let status_icon = match task.status {
                TaskStatus::Pending => "○",
                TaskStatus::Active => "▶",
                TaskStatus::Completed => "✓",
                TaskStatus::Archived => "⊗",
            };

            let description = task
                .description
                .as_ref()
                .map(|d| format!(" - {}", truncate(d, 50)))
                .unwrap_or_default();

            format!("{} {} {}{}", status_icon, task.name, format_status(&task.status), description)
        })
        .collect();

    // Use inquire to create interactive selection
    let selection = Select::new(prompt, task_displays.clone())
        .with_page_size(15)
        .with_help_message("↑↓ to move, enter to select, type to filter")
        .prompt()
        .context("Task selection cancelled")?;

    // Find the selected task by matching the display string
    let selected_index = task_displays
        .iter()
        .position(|d| d == &selection)
        .context("Failed to find selected task")?;

    Ok(tasks[selected_index].clone())
}

/// Select multiple tasks interactively
pub fn select_multiple_tasks(
    task_manager: &TaskManager,
    project: &Project,
    prompt: &str,
    status_filter: Option<TaskStatus>,
) -> Result<Vec<Task>> {
    use inquire::MultiSelect;

    // Get tasks based on filter
    let tasks = task_manager.list_tasks(project, status_filter)?;

    if tasks.is_empty() {
        let filter_msg = match status_filter {
            Some(status) => format!(" with status {:?}", status),
            None => String::new(),
        };
        anyhow::bail!("No tasks found{}", filter_msg);
    }

    // Create display strings for tasks
    let task_displays: Vec<String> = tasks
        .iter()
        .map(|task| {
            let status_icon = match task.status {
                TaskStatus::Pending => "○",
                TaskStatus::Active => "▶",
                TaskStatus::Completed => "✓",
                TaskStatus::Archived => "⊗",
            };

            let description = task
                .description
                .as_ref()
                .map(|d| format!(" - {}", truncate(d, 50)))
                .unwrap_or_default();

            format!("{} {} {}{}", status_icon, task.name, format_status(&task.status), description)
        })
        .collect();

    // Clone task_displays for use in inquire (which takes ownership)
    let task_displays_clone = task_displays.clone();

    // Use inquire to create interactive multi-selection
    let selections = MultiSelect::new(prompt, task_displays_clone)
        .with_page_size(15)
        .with_help_message("↑↓ to move, space to select, enter to confirm, type to filter")
        .prompt()
        .context("Task selection cancelled")?;

    // Find the selected tasks by matching the display strings
    let selected_tasks: Vec<Task> = selections
        .iter()
        .filter_map(|selection| {
            task_displays
                .iter()
                .position(|d| d == selection)
                .map(|idx| tasks[idx].clone())
        })
        .collect();

    Ok(selected_tasks)
}

/// Confirm an action interactively
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    use inquire::Confirm;

    let result = Confirm::new(prompt)
        .with_default(default)
        .with_help_message("y/n")
        .prompt()
        .context("Confirmation cancelled")?;

    Ok(result)
}

/// Helper to truncate text
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format task status with color
fn format_status(status: &TaskStatus) -> String {
    format!("[{:?}]", status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_format_status() {
        assert_eq!(format_status(&TaskStatus::Pending), "[Pending]");
        assert_eq!(format_status(&TaskStatus::Active), "[Active]");
    }
}
