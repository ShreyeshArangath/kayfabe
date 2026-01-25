use anyhow::{Context, Result};
use inquire::{validator::Validation, Confirm, Select, Text};
use rusqlite::Connection;
use std::error::Error;

use crate::core::TaskManager;
use crate::db::{models::{Project, Task, TaskStatus}, operations::tasks};

// ANSI color constants for pretty output
pub const HEADER_COLOR: &str = "\x1b[1;36m";   // Bold cyan
pub const SUCCESS_COLOR: &str = "\x1b[32m";    // Green
pub const ERROR_COLOR: &str = "\x1b[31m";      // Red
pub const INFO_COLOR: &str = "\x1b[33m";       // Yellow
pub const RESET_COLOR: &str = "\x1b[0m";

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

// ======================================================================================
// WIZARD HELPERS
// ======================================================================================

/// Show progress indicator for wizard steps
pub fn show_progress(step: usize, total: usize, label: &str) {
    println!("\n{}Step {}/{}: {}{}", HEADER_COLOR, step, total, label, RESET_COLOR);
}

/// Display a summary box with formatted key-value pairs
pub fn display_summary(items: Vec<(&str, &str)>) {
    println!("\n{}═══ Summary ════════════════════════════{}", INFO_COLOR, RESET_COLOR);
    for (key, value) in items {
        println!("{:<12} {}", format!("{}:", key), value);
    }
    println!();
}

// ======================================================================================
// VALIDATION FUNCTIONS
// ======================================================================================

/// Validate task name format and uniqueness
pub fn validate_task_name(
    name: &str,
    db_conn: &Connection,
    project_id: &str,
) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    // Check length
    if name.is_empty() {
        return Ok(Validation::Invalid("Task name is required".into()));
    }

    if name.len() < 3 {
        return Ok(Validation::Invalid("Task name must be at least 3 characters".into()));
    }

    if name.len() > 50 {
        return Ok(Validation::Invalid("Task name must be 50 characters or less".into()));
    }

    // Check format: lowercase alphanumeric with hyphens/underscores
    let valid_chars = name.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
    });

    if !valid_chars {
        return Ok(Validation::Invalid(
            "Use lowercase letters, numbers, hyphens, and underscores only".into()
        ));
    }

    // Cannot start or end with hyphen
    if name.starts_with('-') || name.ends_with('-') {
        return Ok(Validation::Invalid("Cannot start or end with a hyphen".into()));
    }

    // Check uniqueness
    match tasks::name_exists(db_conn, project_id, name) {
        Ok(true) => Ok(Validation::Invalid(format!("Task '{}' already exists", name).into())),
        Ok(false) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid("Failed to check task name uniqueness".into())),
    }
}

/// Validate project name format
pub fn validate_project_name(name: &str) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    if name.is_empty() {
        return Ok(Validation::Invalid("Project name is required".into()));
    }

    if name.len() < 3 || name.len() > 50 {
        return Ok(Validation::Invalid("Project name must be 3-50 characters".into()));
    }

    let valid = name.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
    });

    if !valid {
        return Ok(Validation::Invalid(
            "Use lowercase alphanumeric with hyphens/underscores".into()
        ));
    }

    Ok(Validation::Valid)
}

/// Validate git URL format
pub fn validate_git_url(url: &str) -> Result<Validation, Box<dyn Error + Send + Sync>> {
    if url.is_empty() {
        return Ok(Validation::Invalid("Git URL is required".into()));
    }

    // Basic validation - check for common git hosting patterns
    let is_valid = url.contains("github.com")
        || url.contains("gitlab.com")
        || url.contains("bitbucket.org")
        || url.starts_with("git@")
        || url.starts_with("https://")
        || url.starts_with("ssh://");

    if !is_valid {
        return Ok(Validation::Invalid("Must be a valid git URL (SSH or HTTPS)".into()));
    }

    Ok(Validation::Valid)
}

// ======================================================================================
// INDIVIDUAL PROMPTS FOR ADD COMMAND
// ======================================================================================

/// Prompt for task name with inline validation
pub fn prompt_task_name(
    db_conn: &Connection,
    project_id: &str,
) -> Result<String> {
    // Clone project_id to move into closure
    let project_id = project_id.to_string();

    // We can't easily pass db_conn into the validator closure due to lifetime issues,
    // so we'll do a two-step validation: format check in validator, uniqueness after prompt
    let name = Text::new("Enter task name:")
        .with_help_message("Use lowercase, hyphens, no spaces")
        .with_validator(move |input: &str| {
            // Check length
            if input.is_empty() {
                return Ok(Validation::Invalid("Task name is required".into()));
            }

            if input.len() < 3 {
                return Ok(Validation::Invalid("Task name must be at least 3 characters".into()));
            }

            if input.len() > 50 {
                return Ok(Validation::Invalid("Task name must be 50 characters or less".into()));
            }

            // Check format: lowercase alphanumeric with hyphens/underscores
            let valid_chars = input.chars().all(|c| {
                c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
            });

            if !valid_chars {
                return Ok(Validation::Invalid(
                    "Use lowercase letters, numbers, hyphens, and underscores only".into()
                ));
            }

            // Cannot start or end with hyphen
            if input.starts_with('-') || input.ends_with('-') {
                return Ok(Validation::Invalid("Cannot start or end with a hyphen".into()));
            }

            Ok(Validation::Valid)
        })
        .prompt()
        .context("Task name input cancelled")?;

    // Check uniqueness after prompting (since we can't access db_conn in the validator)
    if tasks::name_exists(db_conn, &project_id, &name)? {
        anyhow::bail!("Task '{}' already exists. Please try again with a different name.", name);
    }

    Ok(name)
}

/// Prompt for optional task description
pub fn prompt_task_description() -> Result<Option<String>> {
    let description = Text::new("Add description (press Enter to skip):")
        .with_help_message("Optional - describe what this task involves")
        .prompt_skippable()
        .context("Description input cancelled")?;

    Ok(description.filter(|s| !s.trim().is_empty()))
}

/// Prompt for task priority
pub fn prompt_priority() -> Result<String> {
    let options = vec![
        "Medium - Normal priority (default)",
        "High - Urgent work",
        "Low - Nice to have",
    ];

    let selection = Select::new("Select priority:", options)
        .with_help_message("↑↓ to move, enter to select")
        .prompt()
        .context("Priority selection cancelled")?;

    // Extract priority from selection
    let priority = if selection.starts_with("High") {
        "high"
    } else if selection.starts_with("Low") {
        "low"
    } else {
        "medium"
    };

    Ok(priority.to_string())
}

/// Prompt for auto-assign confirmation
pub fn prompt_auto_assign() -> Result<bool> {
    Confirm::new("Auto-assign to available agent?")
        .with_default(false)
        .with_help_message("y/n")
        .prompt()
        .context("Auto-assign confirmation cancelled")
}

// ======================================================================================
// INDIVIDUAL PROMPTS FOR INIT COMMAND
// ======================================================================================

/// Prompt for project name with validation
pub fn prompt_project_name() -> Result<String> {
    Text::new("Enter project name:")
        .with_help_message("Alphanumeric with hyphens. Will be used as directory name")
        .with_validator(|input: &str| validate_project_name(input))
        .prompt()
        .context("Project name input cancelled")
}

/// Prompt for git repository URL with validation
pub fn prompt_git_url() -> Result<String> {
    Text::new("Enter git repository URL:")
        .with_help_message("SSH or HTTPS URL")
        .with_validator(|input: &str| validate_git_url(input))
        .prompt()
        .context("Git URL input cancelled")
}

// ======================================================================================
// WIZARD ORCHESTRATORS
// ======================================================================================

/// Run the full wizard for adding a task
pub fn wizard_add_task(
    task_manager: &TaskManager,
    project: &Project,
) -> Result<(String, Option<String>, String, bool)> {
    println!("\n{}🎯 Create a New Task{}\n", HEADER_COLOR, RESET_COLOR);

    // Step 1: Task name
    show_progress(1, 4, "Task Name");
    let task_name = prompt_task_name(task_manager.db.conn(), &project.id)?;

    // Step 2: Description
    show_progress(2, 4, "Description (Optional)");
    let description = prompt_task_description()?;

    // Step 3: Priority
    show_progress(3, 4, "Priority");
    let priority = prompt_priority()?;

    // Step 4: Auto-assign
    show_progress(4, 4, "Auto-assign");
    let auto_assign = prompt_auto_assign()?;

    // Display summary
    let priority_display = match priority.as_str() {
        "high" => "High",
        "low" => "Low",
        _ => "Medium",
    };

    let desc_display = description.as_deref().unwrap_or("(none)");
    let auto_assign_display = if auto_assign { "Yes" } else { "No" };

    display_summary(vec![
        ("Task", &task_name),
        ("Description", desc_display),
        ("Priority", priority_display),
        ("Auto-assign", auto_assign_display),
    ]);

    // Final confirmation
    let confirmed = Confirm::new("Create this task?")
        .with_default(true)
        .prompt()
        .context("Task creation cancelled")?;

    if !confirmed {
        anyhow::bail!("Task creation cancelled by user");
    }

    Ok((task_name, description, priority, auto_assign))
}

/// Run the full wizard for initializing a project
pub fn wizard_init() -> Result<(String, String)> {
    println!("\n{}🚀 Initialize Kayfabe Project{}\n", HEADER_COLOR, RESET_COLOR);

    // Step 1: Project name
    show_progress(1, 2, "Project Name");
    let project_name = prompt_project_name()?;

    // Step 2: Git URL
    show_progress(2, 2, "Git Repository");
    let git_url = prompt_git_url()?;

    // Display summary
    display_summary(vec![
        ("Project", &project_name),
        ("Repository", &git_url),
    ]);

    // Final confirmation
    let confirmed = Confirm::new("Initialize project?")
        .with_default(true)
        .prompt()
        .context("Project initialization cancelled")?;

    if !confirmed {
        anyhow::bail!("Project initialization cancelled by user");
    }

    Ok((project_name, git_url))
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
