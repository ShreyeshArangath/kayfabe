use crate::db::models::{ExecutionProcess, Task, TaskStatus};
use anyhow::Result;
use std::time::Instant;

/// Task filter options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFilter {
    All,
    Pending,
    Active,
    Completed,
    Archived,
}

impl TaskFilter {
    pub fn as_str(&self) -> &str {
        match self {
            TaskFilter::All => "All",
            TaskFilter::Pending => "Pending",
            TaskFilter::Active => "Active",
            TaskFilter::Completed => "Completed",
            TaskFilter::Archived => "Archived",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TaskFilter::All => TaskFilter::Pending,
            TaskFilter::Pending => TaskFilter::Active,
            TaskFilter::Active => TaskFilter::Completed,
            TaskFilter::Completed => TaskFilter::Archived,
            TaskFilter::Archived => TaskFilter::All,
        }
    }

    pub fn matches(&self, task: &Task) -> bool {
        match self {
            TaskFilter::All => true,
            TaskFilter::Pending => task.status == TaskStatus::Pending,
            TaskFilter::Active => task.status == TaskStatus::Active,
            TaskFilter::Completed => task.status == TaskStatus::Completed,
            TaskFilter::Archived => task.status == TaskStatus::Archived,
        }
    }
}

/// Sort options for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    CreatedAt,
    UpdatedAt,
    Name,
    Status,
}

impl SortBy {
    pub fn as_str(&self) -> &str {
        match self {
            SortBy::CreatedAt => "Created",
            SortBy::UpdatedAt => "Updated",
            SortBy::Name => "Name",
            SortBy::Status => "Status",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SortBy::CreatedAt => SortBy::UpdatedAt,
            SortBy::UpdatedAt => SortBy::Name,
            SortBy::Name => SortBy::Status,
            SortBy::Status => SortBy::CreatedAt,
        }
    }
}

/// Application state for the TUI
pub struct AppState {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub sort_by: SortBy,
    pub show_help: bool,
    pub should_quit: bool,
    pub last_refresh: Instant,
    pub command_mode: bool,
    pub command_input: String,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub message_timestamp: Option<Instant>,
    pub selected_task_execution: Option<ExecutionProcess>,
    pub execution_refresh_needed: bool,
    pub needs_terminal_clear: bool,
}

impl AppState {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            selected_index: 0,
            filter: TaskFilter::All,
            sort_by: SortBy::UpdatedAt,
            show_help: false,
            should_quit: false,
            last_refresh: Instant::now(),
            command_mode: false,
            command_input: String::new(),
            status_message: None,
            error_message: None,
            message_timestamp: None,
            selected_task_execution: None,
            execution_refresh_needed: false,
            needs_terminal_clear: false,
        }
    }

    /// Get filtered and sorted tasks
    pub fn filtered_tasks(&self) -> Vec<&Task> {
        let mut filtered: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| self.filter.matches(task))
            .collect();

        // Sort tasks
        match self.sort_by {
            SortBy::CreatedAt => {
                filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            SortBy::UpdatedAt => {
                filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            SortBy::Name => {
                filtered.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortBy::Status => {
                filtered.sort_by_key(|task| task.status as u8);
            }
        }

        filtered
    }

    /// Get currently selected task
    pub fn selected_task(&self) -> Option<&Task> {
        let filtered = self.filtered_tasks();
        filtered.get(self.selected_index).copied()
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        let filtered_count = self.filtered_tasks().len();
        if filtered_count > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let filtered_count = self.filtered_tasks().len();
        if filtered_count > 0 && self.selected_index < filtered_count.saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Move selection to first item
    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    /// Move selection to last item
    pub fn select_last(&mut self) {
        let filtered_count = self.filtered_tasks().len();
        if filtered_count > 0 {
            self.selected_index = filtered_count - 1;
        }
    }

    /// Toggle filter to next option
    pub fn toggle_filter(&mut self) {
        self.filter = self.filter.next();
        self.selected_index = 0; // Reset selection when filter changes
    }

    /// Change sort order
    pub fn toggle_sort(&mut self) {
        self.sort_by = self.sort_by.next();
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Check if we should refresh data
    pub fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed().as_secs() >= 5
    }

    /// Update tasks list
    pub fn update_tasks(&mut self, tasks: Vec<Task>) -> Result<()> {
        self.tasks = tasks;
        self.last_refresh = Instant::now();

        // Ensure selected index is still valid
        let filtered_count = self.filtered_tasks().len();
        if self.selected_index >= filtered_count && filtered_count > 0 {
            self.selected_index = filtered_count - 1;
        }

        Ok(())
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Enter command mode
    pub fn enter_command_mode(&mut self) {
        self.command_mode = true;
        self.command_input.clear();
        self.clear_messages();
    }

    /// Exit command mode
    pub fn exit_command_mode(&mut self) {
        self.command_mode = false;
        self.command_input.clear();
    }

    /// Add character to command input
    pub fn push_char(&mut self, c: char) {
        self.command_input.push(c);
    }

    /// Remove last character from command input
    pub fn pop_char(&mut self) {
        self.command_input.pop();
    }

    /// Get the current command input
    pub fn get_command_input(&self) -> &str {
        &self.command_input
    }

    /// Clear the command input
    pub fn clear_command_input(&mut self) {
        self.command_input.clear();
    }

    /// Set status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.error_message = None;
        self.message_timestamp = Some(Instant::now());
    }

    /// Set error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.status_message = None;
        self.message_timestamp = Some(Instant::now());
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.status_message = None;
        self.error_message = None;
        self.message_timestamp = None;
    }

    /// Check if messages should be auto-cleared (after 5 seconds)
    pub fn should_clear_messages(&self) -> bool {
        if let Some(timestamp) = self.message_timestamp {
            timestamp.elapsed().as_secs() >= 5
        } else {
            false
        }
    }

    /// Set execution details for selected task
    pub fn set_execution(&mut self, execution: Option<ExecutionProcess>) {
        self.selected_task_execution = execution;
    }

    /// Mark that execution details need refresh
    pub fn mark_execution_refresh_needed(&mut self) {
        self.execution_refresh_needed = true;
    }

    /// Clear execution refresh flag
    pub fn clear_execution_refresh_needed(&mut self) {
        self.execution_refresh_needed = false;
    }

    /// Check if we need to refresh execution details
    pub fn should_refresh_execution(&self) -> bool {
        self.execution_refresh_needed
    }

    /// Mark terminal for clearing (after returning from tmux attach)
    pub fn request_terminal_clear(&mut self) {
        self.needs_terminal_clear = true;
    }

    /// Check if terminal needs clearing
    pub fn needs_clear(&self) -> bool {
        self.needs_terminal_clear
    }

    /// Clear the terminal clear flag
    pub fn clear_terminal_flag(&mut self) {
        self.needs_terminal_clear = false;
    }
}
