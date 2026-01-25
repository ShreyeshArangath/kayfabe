use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Keyboard event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Quit,
    MoveUp,
    MoveDown,
    MoveFirst,
    MoveLast,
    PageUp,
    PageDown,
    ToggleFilter,
    ToggleSort,
    ToggleHelp,
    Execute,
    Remove,
    OpenThoughts,
    Refresh,
    EnterCommandMode,
    None,
}

/// Command mode keyboard action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAction {
    Char(char),
    Backspace,
    Submit,
    Cancel,
    None,
}

/// Handle keyboard input in normal mode
pub fn handle_key_event(key: KeyEvent) -> KeyAction {
    match (key.code, key.modifiers) {
        // Command mode
        (KeyCode::Char(':'), KeyModifiers::NONE) => KeyAction::EnterCommandMode,
        (KeyCode::Char('/'), KeyModifiers::NONE) => KeyAction::EnterCommandMode,

        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) => KeyAction::Quit,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => KeyAction::Quit,
        (KeyCode::Esc, _) => KeyAction::Quit,

        // Navigation - Vim style
        (KeyCode::Char('k'), KeyModifiers::NONE) => KeyAction::MoveUp,
        (KeyCode::Up, _) => KeyAction::MoveUp,
        (KeyCode::Char('j'), KeyModifiers::NONE) => KeyAction::MoveDown,
        (KeyCode::Down, _) => KeyAction::MoveDown,

        // Jump to first/last
        (KeyCode::Char('g'), KeyModifiers::NONE) => KeyAction::MoveFirst,
        (KeyCode::Char('G'), KeyModifiers::SHIFT) => KeyAction::MoveLast,
        (KeyCode::Home, _) => KeyAction::MoveFirst,
        (KeyCode::End, _) => KeyAction::MoveLast,

        // Page navigation
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => KeyAction::PageUp,
        (KeyCode::PageUp, _) => KeyAction::PageUp,
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => KeyAction::PageDown,
        (KeyCode::PageDown, _) => KeyAction::PageDown,

        // Actions
        (KeyCode::Enter, _) => KeyAction::Execute,
        (KeyCode::Char('r'), KeyModifiers::NONE) => KeyAction::Remove,
        (KeyCode::Char('t'), KeyModifiers::NONE) => KeyAction::OpenThoughts,
        (KeyCode::Char('f'), KeyModifiers::NONE) => KeyAction::ToggleFilter,
        (KeyCode::Char('s'), KeyModifiers::NONE) => KeyAction::ToggleSort,
        (KeyCode::Char('h'), KeyModifiers::NONE) => KeyAction::ToggleHelp,
        (KeyCode::Char('?'), KeyModifiers::NONE) => KeyAction::ToggleHelp,
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => KeyAction::Refresh,
        (KeyCode::F(5), _) => KeyAction::Refresh,

        _ => KeyAction::None,
    }
}

/// Handle keyboard input in command mode
pub fn handle_command_key_event(key: KeyEvent) -> CommandAction {
    match key.code {
        KeyCode::Enter => CommandAction::Submit,
        KeyCode::Esc => CommandAction::Cancel,
        KeyCode::Backspace => CommandAction::Backspace,
        KeyCode::Char(c) => CommandAction::Char(c),
        _ => CommandAction::None,
    }
}

/// Poll for events with timeout
pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
