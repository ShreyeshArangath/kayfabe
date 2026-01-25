use anyhow::{anyhow, bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Manager for tmux session operations
pub struct TmuxManager;

impl TmuxManager {
    /// Create a new TmuxManager
    pub fn new() -> Self {
        Self
    }

    /// Check if tmux is installed and available
    pub fn is_available() -> Result<()> {
        Command::new("tmux")
            .arg("-V")
            .output()
            .context("Tmux is not installed or not in PATH. Install with: brew install tmux (macOS) or apt install tmux (Linux)")?;
        Ok(())
    }

    /// Create a new tmux session or attach if it already exists
    ///
    /// Returns the session name
    pub fn create_or_attach_session(
        &self,
        session_name: &str,
        working_dir: &Path,
    ) -> Result<String> {
        // Check if tmux is available
        Self::is_available()?;

        // Check if session already exists
        if self.session_exists(session_name)? {
            return Ok(session_name.to_string());
        }

        // Create new session (detached)
        let status = Command::new("tmux")
            .args([
                "new-session",
                "-d",                        // Detached
                "-s", session_name,          // Session name
                "-c", working_dir.to_str()
                    .ok_or_else(|| anyhow!("Invalid working directory path"))?,
            ])
            .status()
            .context("Failed to create tmux session")?;

        if !status.success() {
            bail!("Failed to create tmux session: {}", session_name);
        }

        Ok(session_name.to_string())
    }

    /// Check if a tmux session exists
    pub fn session_exists(&self, session_name: &str) -> Result<bool> {
        let output = Command::new("tmux")
            .args(["has-session", "-t", session_name])
            .output()
            .context("Failed to check tmux session existence")?;

        Ok(output.status.success())
    }

    /// Attach to a tmux session
    ///
    /// This is a blocking operation that takes over the terminal
    pub fn attach_session(&self, session_name: &str) -> Result<()> {
        if !self.session_exists(session_name)? {
            bail!("Tmux session '{}' does not exist", session_name);
        }

        // Execute tmux attach - this replaces the current process
        let error = Command::new("tmux")
            .args(["attach-session", "-t", session_name])
            .status()
            .context("Failed to attach to tmux session")?;

        if !error.success() {
            bail!("Failed to attach to tmux session: {}", session_name);
        }

        Ok(())
    }

    /// Kill a tmux session
    pub fn kill_session(&self, session_name: &str) -> Result<()> {
        if !self.session_exists(session_name)? {
            // Session doesn't exist, nothing to kill
            return Ok(());
        }

        let status = Command::new("tmux")
            .args(["kill-session", "-t", session_name])
            .status()
            .context("Failed to kill tmux session")?;

        if !status.success() {
            bail!("Failed to kill tmux session: {}", session_name);
        }

        Ok(())
    }

    /// Send keys to a tmux session
    ///
    /// Useful for sending commands to a running session
    pub fn send_keys(&self, session_name: &str, keys: &str) -> Result<()> {
        if !self.session_exists(session_name)? {
            bail!("Tmux session '{}' does not exist", session_name);
        }

        let status = Command::new("tmux")
            .args(["send-keys", "-t", session_name, keys, "C-m"])
            .status()
            .context("Failed to send keys to tmux session")?;

        if !status.success() {
            bail!("Failed to send keys to tmux session: {}", session_name);
        }

        Ok(())
    }

    /// List all tmux sessions
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let sessions = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|s| s.to_string())
                        .collect();
                    Ok(sessions)
                } else {
                    // No sessions exist
                    Ok(vec![])
                }
            }
            Err(_) => {
                // Tmux might not be running or no server exists
                Ok(vec![])
            }
        }
    }

    /// Get the window pane ID for a session
    pub fn get_pane_id(&self, session_name: &str) -> Result<String> {
        if !self.session_exists(session_name)? {
            bail!("Tmux session '{}' does not exist", session_name);
        }

        let output = Command::new("tmux")
            .args(["display-message", "-t", session_name, "-p", "#{pane_id}"])
            .output()
            .context("Failed to get pane ID")?;

        if !output.status.success() {
            bail!("Failed to get pane ID for session: {}", session_name);
        }

        let pane_id = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        Ok(pane_id)
    }
}

impl Default for TmuxManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_tmux_availability() {
        // This test will be skipped in CI if tmux is not installed
        if TmuxManager::is_available().is_err() {
            eprintln!("Skipping test: tmux not available");
            return;
        }

        assert!(TmuxManager::is_available().is_ok());
    }

    #[test]
    fn test_session_lifecycle() {
        if TmuxManager::is_available().is_err() {
            eprintln!("Skipping test: tmux not available");
            return;
        }

        let manager = TmuxManager::new();
        let session_name = "kayfabe-test-session";
        let working_dir = env::current_dir().unwrap();

        // Clean up any existing session
        let _ = manager.kill_session(session_name);

        // Session should not exist
        assert!(!manager.session_exists(session_name).unwrap());

        // Create session
        manager.create_or_attach_session(session_name, &working_dir).unwrap();

        // Session should exist
        assert!(manager.session_exists(session_name).unwrap());

        // Kill session
        manager.kill_session(session_name).unwrap();

        // Session should not exist
        assert!(!manager.session_exists(session_name).unwrap());
    }
}
