use crate::error::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub is_main: bool,
    pub staleness_days: Option<f64>,
    pub safety: SafetyCheck,
}

#[derive(Debug, Clone)]
pub struct SafetyCheck {
    pub has_uncommitted_changes: bool,
    pub has_unmerged_commits: bool,
    pub is_safe_to_remove: bool,
}

impl SafetyCheck {
    pub fn blocked_reasons(&self) -> Vec<&'static str> {
        let mut reasons = Vec::new();
        if self.has_uncommitted_changes {
            reasons.push("has uncommitted changes");
        }
        if self.has_unmerged_commits {
            reasons.push("has unmerged commits");
        }
        reasons
    }
}

pub struct Worktree;

impl Worktree {
    pub fn get_info(path: &Path, base_branch: &str) -> Result<WorktreeInfo> {
        let branch = Self::get_branch(path)?;
        let is_main = path.ends_with("main") || branch.as_deref() == Some(base_branch);
        let staleness_days = Self::calculate_staleness(path)?;
        let safety = Self::check_safety(path, base_branch)?;

        Ok(WorktreeInfo {
            path: path.to_path_buf(),
            branch,
            is_main,
            staleness_days,
            safety,
        })
    }

    fn get_branch(path: &Path) -> Result<Option<String>> {
        let output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(path)
            .output()?;

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if branch.is_empty() {
            None
        } else {
            Some(branch)
        })
    }

    fn calculate_staleness(path: &Path) -> Result<Option<f64>> {
        let output = std::process::Command::new("git")
            .args(["log", "-1", "--format=%ct"])
            .current_dir(path)
            .output()?;

        let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if timestamp_str.is_empty() {
            return Ok(None);
        }

        let timestamp: u64 = timestamp_str.parse().unwrap_or(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let days = (now - timestamp) as f64 / 86400.0;
        Ok(Some(days))
    }

    fn check_safety(path: &Path, base_branch: &str) -> Result<SafetyCheck> {
        let status_output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()?;

        let has_uncommitted_changes = !status_output.stdout.is_empty();

        let log_output = std::process::Command::new("git")
            .args(["log", &format!("{}..HEAD", base_branch), "--oneline"])
            .current_dir(path)
            .output()?;

        let has_unmerged_commits = !log_output.stdout.is_empty();

        let is_safe_to_remove = !has_uncommitted_changes && !has_unmerged_commits;

        Ok(SafetyCheck {
            has_uncommitted_changes,
            has_unmerged_commits,
            is_safe_to_remove,
        })
    }
}
