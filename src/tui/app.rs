use anyhow::{Context, Result};
use crossterm::{
    event::Event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

use crate::cli;
use crate::core::{Executor, TaskManager};
use crate::db::models::Project;
use crate::db::operations::execution_processes;
use crate::db::Database;
use crate::tui::{command, events, state::AppState, ui};
use crate::utils;

/// Run the TUI application standalone (no project required initially)
pub async fn run_tui_standalone() -> Result<()> {
    // Try to get current project, but don't fail if not in one
    let db = Database::open().context("Failed to open database")?;
    let task_manager = TaskManager::new(db);

    let project = match utils::find_project_root() {
        Ok(_root) => {
            task_manager.get_current_project().ok()
        }
        Err(_) => None,
    };

    match project {
        Some(p) => run_tui(&p).await,
        None => {
            // No project - show welcome screen with command mode only
            run_tui_no_project().await
        }
    }
}

/// Run TUI without a project (for initialization)
async fn run_tui_no_project() -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
    terminal.clear()?;

    // Show welcome screen
    let mut quit = false;
    let mut command_input = String::new();
    let mut in_command_mode = false;
    let mut message = Some("No project found. Use /init <name> <git-url> to get started".to_string());
    let mut error_message: Option<String> = None;

    loop {
        terminal.draw(|f| {
            ui::render_welcome_screen(f, &command_input, in_command_mode, &message, &error_message)
        })?;

        if let Some(event) = events::poll_event(Duration::from_millis(100))? {
            if let Event::Key(key) = event {
                if in_command_mode {
                    let action = events::handle_command_key_event(key);
                    match action {
                        events::CommandAction::Char(c) => {
                            command_input.push(c);
                        }
                        events::CommandAction::Backspace => {
                            command_input.pop();
                        }
                        events::CommandAction::Submit => {
                            let cmd_str = command_input.clone();
                            command_input.clear();
                            in_command_mode = false;

                            // Handle init command
                            if let Ok(cmd) = command::parse_command(&cmd_str) {
                                if let command::Command::Init { name, git_url } = cmd {
                                    // Exit TUI for init
                                    disable_raw_mode()?;
                                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                                    terminal.show_cursor()?;

                                    // Interactive mode if no args provided
                                    let interactive = name.is_none() || git_url.is_none();

                                    match cli::init::init(name, git_url, interactive).await {
                                        Ok(_) => {
                                            println!("\n✓ Project initialized successfully!");
                                            println!("You can now run 'kayfabe' to enter the TUI.");
                                            return Ok(());
                                        }
                                        Err(e) => {
                                            eprintln!("\n✗ Initialization failed: {}", e);
                                            println!("\nPress Enter to return...");
                                            let mut input = String::new();
                                            io::stdin().read_line(&mut input)?;
                                        }
                                    }

                                    enable_raw_mode()?;
                                    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                    terminal.clear()?;
                                } else if let command::Command::Quit = cmd {
                                    quit = true;
                                } else {
                                    error_message = Some("Only /init and /quit commands are available without a project".to_string());
                                    message = None;
                                }
                            } else {
                                error_message = Some("Invalid command. Try /init or /init <name> <git-url>".to_string());
                                message = None;
                            }
                        }
                        events::CommandAction::Cancel => {
                            in_command_mode = false;
                            command_input.clear();
                        }
                        events::CommandAction::None => {}
                    }
                } else {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                            quit = true;
                        }
                        crossterm::event::KeyCode::Char(':') | crossterm::event::KeyCode::Char('/') => {
                            in_command_mode = true;
                            command_input.clear();
                            error_message = None;
                        }
                        crossterm::event::KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Run the TUI application with task interface
pub async fn run_tui(project: &Project) -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
    terminal.clear()?;

    // Run the task app
    let result = run_task_app(&mut terminal, project).await;

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    terminal.show_cursor()?;

    result
}

/// Main task application loop
async fn run_task_app<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    project: &Project,
) -> Result<()> {
    // Initialize database and task manager
    let db = Database::open().context("Failed to open database")?;
    let task_manager = TaskManager::new(db);

    // Load initial tasks
    let tasks = task_manager
        .list_tasks(project, None)
        .context("Failed to load tasks")?;

    let mut app_state = AppState::new(tasks);

    loop {
        // Check if terminal needs clearing (after returning from tmux attach)
        if app_state.needs_clear() {
            terminal.clear()?;
            app_state.clear_terminal_flag();
        }

        // Render UI
        terminal.draw(|f| ui::render(f, &app_state))?;

        // Poll for events with timeout
        if let Some(event) = events::poll_event(Duration::from_millis(100))? {
            if let Event::Key(key) = event {
                // If help is shown, any key closes it
                if app_state.show_help {
                    app_state.toggle_help();
                    continue;
                }

                // Handle input based on mode
                if app_state.command_mode {
                    handle_command_mode(key, &mut app_state, &task_manager, project).await?;
                } else {
                    handle_normal_mode(key, &mut app_state, &task_manager, project).await?;
                }
            }
        }

        // Auto-clear messages after 5s
        if app_state.should_clear_messages() {
            app_state.clear_messages();
        }

        // Auto-refresh if needed
        if app_state.should_refresh() {
            if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                let _ = app_state.update_tasks(updated_tasks);
            }
        }

        // Refresh execution details if needed
        if app_state.should_refresh_execution() {
            if let Some(task) = app_state.selected_task() {
                let db = Database::open().context("Failed to open database")?;
                if let Ok(Some(exec)) = execution_processes::get_latest_by_task_id(db.conn(), &task.id) {
                    app_state.set_execution(Some(exec));
                }
            }
            app_state.clear_execution_refresh_needed();
        }

        // Check if we should quit
        if app_state.should_quit {
            break;
        }
    }

    Ok(())
}

/// Handle keyboard input in command mode
async fn handle_command_mode(
    key: crossterm::event::KeyEvent,
    state: &mut AppState,
    task_manager: &TaskManager,
    project: &Project,
) -> Result<()> {
    let action = events::handle_command_key_event(key);

    match action {
        events::CommandAction::Char(c) => {
            state.push_char(c);
        }
        events::CommandAction::Backspace => {
            state.pop_char();
        }
        events::CommandAction::Submit => {
            let cmd_str = state.get_command_input().to_string();
            state.exit_command_mode();

            if !cmd_str.trim().is_empty() {
                execute_command(&cmd_str, state, task_manager, project).await?;
            }
        }
        events::CommandAction::Cancel => {
            state.exit_command_mode();
        }
        events::CommandAction::None => {}
    }

    Ok(())
}

/// Handle keyboard input in normal mode
async fn handle_normal_mode(
    key: crossterm::event::KeyEvent,
    state: &mut AppState,
    task_manager: &TaskManager,
    project: &Project,
) -> Result<()> {
    let action = events::handle_key_event(key);

    match action {
        events::KeyAction::Quit => {
            state.quit();
        }
        events::KeyAction::MoveUp => {
            state.select_previous();
        }
        events::KeyAction::MoveDown => {
            state.select_next();
        }
        events::KeyAction::MoveFirst => {
            state.select_first();
        }
        events::KeyAction::MoveLast => {
            state.select_last();
        }
        events::KeyAction::ToggleFilter => {
            state.toggle_filter();
        }
        events::KeyAction::ToggleSort => {
            state.toggle_sort();
        }
        events::KeyAction::ToggleHelp => {
            state.toggle_help();
        }
        events::KeyAction::Execute => {
            if let Some(task) = state.selected_task() {
                let task = task.clone(); // Clone to avoid borrow issues

                // Execute task WITHOUT exiting TUI
                let db = Database::open().context("Failed to open database")?;
                let mut executor = Executor::new(db);

                match executor.execute_task(project, &task, "claude").await {
                    Ok(exec) => {
                        state.set_status(format!(
                            "✓ Started execution for '{}' in session {}",
                            task.name,
                            exec.tmux_session.as_ref().unwrap_or(&"unknown".to_string())
                        ));
                        state.set_execution(Some(exec));
                        state.mark_execution_refresh_needed();

                        // Refresh task list to show updated status
                        if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                            let _ = state.update_tasks(updated_tasks);
                        }
                    }
                    Err(e) => {
                        state.set_error(format!("Failed to execute task: {}", e));
                    }
                }
            } else {
                state.set_error("No task selected".to_string());
            }
        }
        events::KeyAction::Attach => {
            if let Some(task) = state.selected_task() {
                let task = task.clone();

                // Check if task is active
                if task.status != crate::db::models::TaskStatus::Active {
                    state.set_error(format!("Task '{}' is not active", task.name));
                } else {
                    // Exit TUI temporarily to attach to tmux
                    disable_raw_mode()?;
                    execute!(std::io::stdout(), LeaveAlternateScreen)?;

                    let db = Database::open().context("Failed to open database")?;
                    let executor = Executor::new(db);

                    match executor.attach_task(&task) {
                        Ok(_) => {
                            // User has detached from tmux, return to TUI
                        }
                        Err(e) => {
                            eprintln!("\n✗ Failed to attach: {}", e);
                            println!("\nPress Enter to continue...");
                            let mut input = String::new();
                            io::stdin().read_line(&mut input)?;
                        }
                    }

                    // Restore TUI with full terminal reset
                    enable_raw_mode()?;
                    execute!(std::io::stdout(), EnterAlternateScreen)?;

                    // Request terminal clear for next render cycle
                    state.request_terminal_clear();

                    // Refresh task list after returning
                    if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                        let _ = state.update_tasks(updated_tasks);
                    }
                    state.mark_execution_refresh_needed();
                }
            } else {
                state.set_error("No task selected".to_string());
            }
        }
        events::KeyAction::Kill => {
            if let Some(task) = state.selected_task() {
                let task = task.clone();

                // Check if task is active
                if task.status != crate::db::models::TaskStatus::Active {
                    state.set_error(format!("Task '{}' is not active", task.name));
                } else {
                    let db = Database::open().context("Failed to open database")?;
                    let executor = Executor::new(db);

                    match executor.kill_task(&task).await {
                        Ok(_) => {
                            state.set_status(format!("✓ Killed task '{}'", task.name));
                            state.set_execution(None);

                            // Request terminal clear to fix UI corruption
                            state.request_terminal_clear();

                            // Refresh task list
                            if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                                let _ = state.update_tasks(updated_tasks);
                            }

                            // Mark execution refresh needed
                            state.mark_execution_refresh_needed();
                        }
                        Err(e) => {
                            state.set_error(format!("Failed to kill task: {}", e));
                        }
                    }
                }
            } else {
                state.set_error("No task selected".to_string());
            }
        }
        events::KeyAction::ViewLogs => {
            if let Some(task) = state.selected_task() {
                let task_id = task.id.clone();
                let task_name = task.name.clone();

                // Load and display execution for selected task
                let db = Database::open().context("Failed to open database")?;

                match execution_processes::get_latest_by_task_id(db.conn(), &task_id) {
                    Ok(Some(exec)) => {
                        state.set_execution(Some(exec));
                        state.set_status(format!("Showing logs for '{}'", task_name));
                        // Request terminal clear to prevent UI corruption
                        state.request_terminal_clear();
                    }
                    Ok(None) => {
                        state.set_execution(None);
                        state.set_error(format!("No execution history for '{}'", task_name));
                    }
                    Err(e) => {
                        state.set_error(format!("Failed to load logs: {}", e));
                    }
                }
            } else {
                state.set_error("No task selected".to_string());
            }
        }
        events::KeyAction::Refresh => {
            if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                state.update_tasks(updated_tasks)?;
                state.set_status("Task list refreshed".to_string());
            }
        }
        events::KeyAction::EnterCommandMode => {
            state.enter_command_mode();
        }
        events::KeyAction::PageUp | events::KeyAction::PageDown |
        events::KeyAction::Remove | events::KeyAction::OpenThoughts => {
            // TODO: Not implemented yet
        }
        events::KeyAction::None => {}
    }

    Ok(())
}

/// Execute a command
async fn execute_command(
    cmd_str: &str,
    state: &mut AppState,
    task_manager: &TaskManager,
    project: &Project,
) -> Result<()> {
    match command::parse_command(cmd_str) {
        Ok(cmd) => {
            match cmd {
                command::Command::Add { name, description, interactive } => {
                    if interactive {
                        // Interactive wizard mode requested
                        // Show helpful message since we can't run wizards inside TUI
                        state.set_error(
                            "💡 For interactive wizard: Exit TUI (press 'q') and run 'kayfabe add'. Or use '/add <name> [desc]' here.".to_string()
                        );
                    } else if let Some(task_name) = name {
                        // Direct mode - create task immediately
                        match task_manager.create_task(project, &task_name, description.as_deref()) {
                            Ok(_) => {
                                state.set_status(format!("✓ Created task '{}'", task_name));
                                // Refresh task list
                                if let Ok(tasks) = task_manager.list_tasks(project, None) {
                                    state.update_tasks(tasks)?;
                                }
                            }
                            Err(e) => {
                                state.set_error(format!("Failed to create task: {}", e));
                            }
                        }
                    } else {
                        state.set_error("Task name is required. Use '/add <name> [desc]'".to_string());
                    }
                }
                command::Command::Remove { name, force } => {
                    match task_manager.remove_task(project, &name, force) {
                        Ok(_) => {
                            state.set_status(format!("✓ Removed task '{}'", name));
                            // Refresh task list
                            if let Ok(tasks) = task_manager.list_tasks(project, None) {
                                state.update_tasks(tasks)?;
                            }
                        }
                        Err(e) => {
                            state.set_error(format!("Failed to remove task: {}", e));
                        }
                    }
                }
                command::Command::Execute { name } => {
                    // Get the task first
                    match task_manager.get_task(project, &name)? {
                        Some(task) => {
                            // Execute WITHOUT exiting TUI
                            let db = Database::open().context("Failed to open database")?;
                            let mut executor = Executor::new(db);

                            match executor.execute_task(project, &task, "claude").await {
                                Ok(exec) => {
                                    state.set_status(format!(
                                        "✓ Started execution for '{}' in session {}",
                                        task.name,
                                        exec.tmux_session.as_ref().unwrap_or(&"unknown".to_string())
                                    ));
                                    state.set_execution(Some(exec));
                                    state.mark_execution_refresh_needed();

                                    // Refresh task list
                                    if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                                        let _ = state.update_tasks(updated_tasks);
                                    }
                                }
                                Err(e) => {
                                    state.set_error(format!("Failed to execute task: {}", e));
                                }
                            }
                        }
                        None => {
                            state.set_error(format!("Task '{}' not found", name));
                        }
                    }
                }
                command::Command::Status => {
                    let task_count = state.tasks.len();
                    let pending = state.tasks.iter().filter(|t| t.status == crate::db::models::TaskStatus::Pending).count();
                    let active = state.tasks.iter().filter(|t| t.status == crate::db::models::TaskStatus::Active).count();
                    let completed = state.tasks.iter().filter(|t| t.status == crate::db::models::TaskStatus::Completed).count();

                    state.set_status(format!(
                        "Tasks: {} total ({} pending, {} active, {} completed)",
                        task_count, pending, active, completed
                    ));
                }
                command::Command::Refresh => {
                    if let Ok(tasks) = task_manager.list_tasks(project, None) {
                        state.update_tasks(tasks)?;
                        state.set_status("Task list refreshed".to_string());
                    }
                }
                command::Command::Help => {
                    state.toggle_help();
                }
                command::Command::Quit => {
                    state.quit();
                }
                command::Command::Init { .. } => {
                    state.set_error("Cannot initialize project from within TUI".to_string());
                }
                command::Command::Attach { name } => {
                    match task_manager.get_task(project, &name)? {
                        Some(task) => {
                            if task.status != crate::db::models::TaskStatus::Active {
                                state.set_error(format!("Task '{}' is not active", task.name));
                            } else {
                                // Exit TUI temporarily to attach
                                disable_raw_mode()?;
                                execute!(std::io::stdout(), LeaveAlternateScreen)?;

                                let db = Database::open().context("Failed to open database")?;
                                let executor = Executor::new(db);

                                match executor.attach_task(&task) {
                                    Ok(_) => {
                                        // User has detached, return to TUI
                                    }
                                    Err(e) => {
                                        eprintln!("\n✗ Failed to attach: {}", e);
                                        println!("\nPress Enter to continue...");
                                        let mut input = String::new();
                                        io::stdin().read_line(&mut input)?;
                                    }
                                }

                                // Restore TUI with full terminal reset
                                enable_raw_mode()?;
                                execute!(std::io::stdout(), EnterAlternateScreen)?;

                                // Request terminal clear in the next render cycle
                                state.request_terminal_clear();

                                // Refresh task list
                                if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                                    let _ = state.update_tasks(updated_tasks);
                                }
                                state.mark_execution_refresh_needed();
                            }
                        }
                        None => {
                            state.set_error(format!("Task '{}' not found", name));
                        }
                    }
                }
                command::Command::Kill { name } => {
                    match task_manager.get_task(project, &name)? {
                        Some(task) => {
                            if task.status != crate::db::models::TaskStatus::Active {
                                state.set_error(format!("Task '{}' is not active", task.name));
                            } else {
                                let db = Database::open().context("Failed to open database")?;
                                let executor = Executor::new(db);

                                match executor.kill_task(&task).await {
                                    Ok(_) => {
                                        state.set_status(format!("✓ Killed task '{}'", task.name));
                                        state.set_execution(None);

                                        // Request terminal clear to fix UI corruption
                                        state.request_terminal_clear();

                                        // Refresh task list
                                        if let Ok(updated_tasks) = task_manager.list_tasks(project, None) {
                                            let _ = state.update_tasks(updated_tasks);
                                        }

                                        // Mark execution refresh needed
                                        state.mark_execution_refresh_needed();
                                    }
                                    Err(e) => {
                                        state.set_error(format!("Failed to kill task: {}", e));
                                    }
                                }
                            }
                        }
                        None => {
                            state.set_error(format!("Task '{}' not found", name));
                        }
                    }
                }
            }
        }
        Err(e) => {
            state.set_error(format!("Invalid command: {}", e));
        }
    }

    Ok(())
}

