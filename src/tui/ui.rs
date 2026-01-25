use crate::db::models::{Task, TaskStatus};
use crate::tui::state::AppState;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Main UI rendering function
pub fn render(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Command/status bar
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    render_header(f, chunks[0], state);
    render_content(f, chunks[1], state);
    render_command_bar(f, chunks[2], state);
    render_footer(f, chunks[3], state);

    if state.show_help {
        render_help_overlay(f);
    }
}

/// Render the header section
fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let title = format!(
        " Kayfabe - Task Manager │ Filter: {} │ Sort: {} ",
        state.filter.as_str(),
        state.sort_by.as_str()
    );

    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

/// Render the main content area (split between task list and details)
fn render_content(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45), // Task list
            Constraint::Percentage(55), // Details panel
        ])
        .split(area);

    render_task_list(f, chunks[0], state);
    render_task_details(f, chunks[1], state);
}

/// Render the task list
fn render_task_list(f: &mut Frame, area: Rect, state: &AppState) {
    let filtered_tasks = state.filtered_tasks();

    let items: Vec<ListItem> = filtered_tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let is_selected = idx == state.selected_index;
            create_task_list_item(task, is_selected)
        })
        .collect();

    let title = format!(" Tasks ({}) ", filtered_tasks.len());

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 60))
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

/// Create a single task list item
fn create_task_list_item(task: &Task, is_selected: bool) -> ListItem<'_> {
    let (status_icon, status_color) = match task.status {
        TaskStatus::Pending => ("○", Color::Gray),
        TaskStatus::Active => ("▶", Color::Green),
        TaskStatus::Completed => ("✓", Color::Blue),
        TaskStatus::Archived => ("⊗", Color::DarkGray),
    };

    let name = truncate(&task.name, 30);
    let status_text = format!("{:?}", task.status);

    let line = if is_selected {
        Line::from(vec![
            Span::styled(
                format!(" {} ", status_icon),
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<32}", name),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}]", status_text),
                Style::default().fg(status_color),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" {} ", status_icon), Style::default().fg(status_color)),
            Span::styled(format!("{:<32}", name), Style::default().fg(Color::White)),
            Span::styled(
                format!("[{}]", status_text),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    };

    ListItem::new(line)
}

/// Render the task details panel
fn render_task_details(f: &mut Frame, area: Rect, state: &AppState) {
    if let Some(task) = state.selected_task() {
        let details_text = create_task_details(task);

        let paragraph = Paragraph::new(details_text)
            .block(
                Block::default()
                    .title(" Details ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    } else {
        let empty = Paragraph::new("No task selected")
            .block(
                Block::default()
                    .title(" Details ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(empty, area);
    }
}

/// Create detailed task information
fn create_task_details(task: &Task) -> Text<'_> {
    let mut lines = Vec::new();

    // Task name
    lines.push(Line::from(vec![
        Span::styled("Name: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(&task.name, Style::default().fg(Color::White)),
    ]));

    lines.push(Line::from(""));

    // Status
    let (status_icon, status_color) = match task.status {
        TaskStatus::Pending => ("○ Pending", Color::Gray),
        TaskStatus::Active => ("▶ Active", Color::Green),
        TaskStatus::Completed => ("✓ Completed", Color::Blue),
        TaskStatus::Archived => ("⊗ Archived", Color::DarkGray),
    };
    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(status_icon, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
    ]));

    lines.push(Line::from(""));

    // Branch
    if let Some(branch) = &task.git_branch {
        lines.push(Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(branch, Style::default().fg(Color::Yellow)),
        ]));
        lines.push(Line::from(""));
    }

    // Timestamps
    lines.push(Line::from(vec![
        Span::styled("Created: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format_relative_time(&task.created_at), Style::default().fg(Color::White)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Updated: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format_relative_time(&task.updated_at), Style::default().fg(Color::White)),
    ]));

    if let Some(last_accessed) = &task.last_accessed_at {
        lines.push(Line::from(vec![
            Span::styled("Accessed: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format_relative_time(last_accessed), Style::default().fg(Color::White)),
        ]));
    }

    if let Some(completed) = &task.completed_at {
        lines.push(Line::from(vec![
            Span::styled("Completed: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format_relative_time(completed), Style::default().fg(Color::White)),
        ]));
    }

    lines.push(Line::from(""));

    // Description
    if let Some(desc) = &task.description {
        lines.push(Line::from(Span::styled(
            "Description:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Wrap description text
        for line in desc.lines() {
            lines.push(Line::from(Span::styled(line, Style::default().fg(Color::White))));
        }

        lines.push(Line::from(""));
    }

    // Worktree path
    if let Some(path) = &task.worktree_path {
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                path.display().to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    Text::from(lines)
}

/// Render the command/status bar
fn render_command_bar(f: &mut Frame, area: Rect, state: &AppState) {
    if state.command_mode {
        // Command input mode
        let input = Paragraph::new(state.get_command_input())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Command ")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            );
        f.render_widget(input, area);
    } else if let Some(error) = &state.error_message {
        // Error message
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Error ")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            );
        f.render_widget(error_widget, area);
    } else if let Some(status) = &state.status_message {
        // Status message
        let status_widget = Paragraph::new(status.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Info ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            );
        f.render_widget(status_widget, area);
    } else {
        // Hint text
        let hint = Paragraph::new("Press : or / to enter command mode")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        f.render_widget(hint, area);
    }
}

/// Render the footer with keyboard shortcuts
fn render_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let shortcuts = if state.command_mode {
        vec![("Enter", "Submit"), ("Esc", "Cancel")]
    } else {
        vec![
            ("j/k", "Nav"),
            (":", "Cmd"),
            ("Enter", "Exec"),
            ("f", "Filter"),
            ("s", "Sort"),
            ("?", "Help"),
            ("q", "Quit"),
        ]
    };

    let footer_text: Vec<Span> = shortcuts
        .iter()
        .enumerate()
        .flat_map(|(idx, (key, action))| {
            let mut spans = vec![
                Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(" {} ", action),
                    Style::default().fg(Color::White),
                ),
            ];

            // Add separator except for last item
            if idx < shortcuts.len() - 1 {
                spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
            }

            spans
        })
        .collect();

    let footer = Paragraph::new(Line::from(footer_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

/// Render help overlay
fn render_help_overlay(f: &mut Frame) {
    let area = centered_rect(70, 80, f.size());

    // Clear the area
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Kayfabe Help",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Command Mode", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  :/          ", Style::default().fg(Color::Green)),
            Span::styled("Enter command mode", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Available Commands:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  :add <name> [desc]  ", Style::default().fg(Color::Green)),
            Span::styled("Add a new task", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :remove <name>      ", Style::default().fg(Color::Green)),
            Span::styled("Remove a task", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :execute <name>     ", Style::default().fg(Color::Green)),
            Span::styled("Execute a task", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :status             ", Style::default().fg(Color::Green)),
            Span::styled("Show task statistics", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :refresh            ", Style::default().fg(Color::Green)),
            Span::styled("Refresh task list", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :help               ", Style::default().fg(Color::Green)),
            Span::styled("Show this help", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  :quit or :q         ", Style::default().fg(Color::Green)),
            Span::styled("Exit Kayfabe", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ↑/k         ", Style::default().fg(Color::Green)),
            Span::styled("Move up", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ↓/j         ", Style::default().fg(Color::Green)),
            Span::styled("Move down", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  g/Home      ", Style::default().fg(Color::Green)),
            Span::styled("Jump to first", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  G/End       ", Style::default().fg(Color::Green)),
            Span::styled("Jump to last", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Quick Actions", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  f           ", Style::default().fg(Color::Green)),
            Span::styled("Toggle filter", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  s           ", Style::default().fg(Color::Green)),
            Span::styled("Change sort order", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  q           ", Style::default().fg(Color::Green)),
            Span::styled("Quit", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .alignment(Alignment::Left);

    f.render_widget(help, area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format a datetime as relative time (e.g., "2 hours ago")
fn format_relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if duration.num_days() < 30 {
        let days = duration.num_days();
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if duration.num_days() < 365 {
        let months = duration.num_days() / 30;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = duration.num_days() / 365;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

/// Render welcome screen when no project is found
pub fn render_welcome_screen(
    f: &mut Frame,
    command_input: &str,
    in_command_mode: bool,
    message: &Option<String>,
    error_message: &Option<String>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Command/message
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    // Header
    let header = Paragraph::new(" Kayfabe - Task Manager ")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Welcome content
    let welcome_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Welcome to Kayfabe!",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "No project found in the current directory.",
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "To get started, initialize a new project:",
            Style::default().fg(Color::Green),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  1. Press ", Style::default().fg(Color::White)),
            Span::styled(":", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" or ", Style::default().fg(Color::White)),
            Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to enter command mode", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  2. Type: ", Style::default().fg(Color::White)),
            Span::styled("/init <project-name> <git-url>", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  3. Press ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Example:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  /init my-app https://github.com/user/repo.git", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let content = Paragraph::new(welcome_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .alignment(Alignment::Center);
    f.render_widget(content, chunks[1]);

    // Command input or message
    if in_command_mode {
        let input = Paragraph::new(command_input)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Command ")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            );
        f.render_widget(input, chunks[2]);
    } else if let Some(err) = error_message {
        let error = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Error ")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            );
        f.render_widget(error, chunks[2]);
    } else if let Some(msg) = message {
        let status = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Info ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            );
        f.render_widget(status, chunks[2]);
    }

    // Footer
    let shortcuts = if in_command_mode {
        vec![("Enter", "Execute"), ("Esc", "Cancel")]
    } else {
        vec![(":", "Command"), ("q", "Quit")]
    };

    let footer_text: Vec<Span> = shortcuts
        .iter()
        .enumerate()
        .flat_map(|(idx, (key, action))| {
            let mut spans = vec![
                Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", action), Style::default().fg(Color::White)),
            ];
            if idx < shortcuts.len() - 1 {
                spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
            }
            spans
        })
        .collect();

    let footer = Paragraph::new(Line::from(footer_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}
