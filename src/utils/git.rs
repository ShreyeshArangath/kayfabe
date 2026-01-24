use anyhow::{bail, Context, Result};
use git2::{BranchType, Repository};
use std::path::Path;
use std::process::Command;

/// Validate a git URL format (SSH or HTTPS)
pub fn validate_git_url(url: &str) -> Result<()> {
    let is_ssh = url.starts_with("git@") || url.starts_with("ssh://");
    let is_https = url.starts_with("https://") || url.starts_with("http://");

    if !is_ssh && !is_https {
        bail!(
            "Invalid git URL format. Must start with 'git@', 'ssh://', 'https://', or 'http://'"
        );
    }

    Ok(())
}

/// Detect the default branch of a repository (main, master, or other)
pub fn detect_default_branch(repo: &Repository) -> Result<String> {
    // Try to get HEAD reference
    let head = repo
        .find_reference("HEAD")
        .context("Failed to find HEAD reference")?;

    // If HEAD is symbolic (points to a branch), resolve it
    if let Ok(resolved) = head.resolve() {
        if let Some(name) = resolved.name() {
            // Extract branch name from refs/heads/...
            if let Some(branch_name) = name.strip_prefix("refs/heads/") {
                return Ok(branch_name.to_string());
            }
        }
    }

    // Fallback: Try common default branches
    for branch in &["main", "master", "develop"] {
        if repo.find_branch(branch, BranchType::Local).is_ok() {
            return Ok(branch.to_string());
        }
    }

    // Last resort: Get the first branch
    let branches = repo.branches(Some(BranchType::Local))?;
    for branch_result in branches {
        let (branch, _) = branch_result?;
        if let Some(name) = branch.name()? {
            return Ok(name.to_string());
        }
    }

    bail!("Could not determine default branch - repository may be empty")
}

/// Check if a git repository exists at the given path
pub fn is_git_repository(path: &Path) -> bool {
    Repository::open(path).is_ok()
}

/// Get the current branch name in a repository
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head().context("Failed to get HEAD")?;

    if let Some(branch_name) = head.shorthand() {
        Ok(branch_name.to_string())
    } else {
        bail!("HEAD is not pointing to a branch")
    }
}

/// Check if a worktree directory has uncommitted changes
pub fn has_uncommitted_changes(worktree_path: &Path) -> Result<bool> {
    let repo = Repository::open(worktree_path)
        .with_context(|| format!("Failed to open repository at {}", worktree_path.display()))?;

    let statuses = repo
        .statuses(None)
        .context("Failed to get repository status")?;

    Ok(!statuses.is_empty())
}

/// List all worktrees in a repository using git CLI
/// Returns Vec<(worktree_path, branch_name)>
pub fn list_worktrees(repo_path: &Path) -> Result<Vec<(String, String)>> {
    let output = Command::new("git")
        .args(&["-C", repo_path.to_str().unwrap(), "worktree", "list", "--porcelain"])
        .output()
        .context("Failed to execute git worktree list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git worktree list failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_worktree_list(&stdout)
}

/// Parse git worktree list --porcelain output
fn parse_worktree_list(output: &str) -> Result<Vec<(String, String)>> {
    let mut worktrees = Vec::new();
    let mut current_path = None;
    let mut current_branch = None;

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(path.to_string());
        } else if let Some(branch) = line.strip_prefix("branch ") {
            // Strip refs/heads/ prefix
            current_branch = Some(
                branch
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch)
                    .to_string(),
            );
        } else if line.is_empty() {
            // End of worktree entry
            if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take()) {
                worktrees.push((path, branch));
            }
        }
    }

    // Handle last entry if no trailing newline
    if let (Some(path), Some(branch)) = (current_path, current_branch) {
        worktrees.push((path, branch));
    }

    Ok(worktrees)
}

/// Add a git worktree using git CLI (git2-rs doesn't support this directly)
pub fn add_worktree(repo_path: &Path, worktree_path: &Path, branch_name: &str) -> Result<()> {
    let status = Command::new("git")
        .args(&[
            "-C",
            repo_path.to_str().unwrap(),
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            branch_name,
        ])
        .status()
        .context("Failed to execute git worktree add")?;

    if !status.success() {
        bail!(
            "git worktree add failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    // Verify worktree was created
    if !worktree_path.exists() {
        bail!("Worktree creation succeeded but directory doesn't exist");
    }

    Ok(())
}

/// Remove a git worktree using git CLI
pub fn remove_worktree(repo_path: &Path, worktree_path: &Path, force: bool) -> Result<()> {
    let mut args = vec![
        "-C",
        repo_path.to_str().unwrap(),
        "worktree",
        "remove",
        worktree_path.to_str().unwrap(),
    ];

    if force {
        args.push("--force");
    }

    let status = Command::new("git")
        .args(&args)
        .status()
        .context("Failed to execute git worktree remove")?;

    if !status.success() {
        bail!(
            "git worktree remove failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

/// Prune stale worktree administrative files
pub fn prune_worktrees(repo_path: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(&["-C", repo_path.to_str().unwrap(), "worktree", "prune"])
        .status()
        .context("Failed to execute git worktree prune")?;

    if !status.success() {
        bail!(
            "git worktree prune failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

/// Check if git is available on the system
pub fn check_git_available() -> Result<()> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .context("Failed to execute git. Is git installed?")?;

    if !output.status.success() {
        bail!("git command is available but returned an error");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_git_url() {
        // Valid SSH URLs
        assert!(validate_git_url("git@github.com:user/repo.git").is_ok());
        assert!(validate_git_url("ssh://git@github.com/user/repo.git").is_ok());

        // Valid HTTPS URLs
        assert!(validate_git_url("https://github.com/user/repo.git").is_ok());
        assert!(validate_git_url("http://github.com/user/repo.git").is_ok());

        // Invalid URLs
        assert!(validate_git_url("not-a-git-url").is_err());
        assert!(validate_git_url("ftp://github.com/user/repo.git").is_err());
        assert!(validate_git_url("/local/path/to/repo").is_err());
    }

    #[test]
    fn test_parse_worktree_list() {
        let output = "worktree /path/to/main\nHEAD abc123\nbranch refs/heads/main\n\n\
                      worktree /path/to/feature\nHEAD def456\nbranch refs/heads/feature-x\n\n";

        let worktrees = parse_worktree_list(output).unwrap();
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0], ("/path/to/main".to_string(), "main".to_string()));
        assert_eq!(
            worktrees[1],
            ("/path/to/feature".to_string(), "feature-x".to_string())
        );
    }

    #[test]
    fn test_check_git_available() {
        // This should succeed if git is installed
        // We can't guarantee git is installed in all test environments,
        // so this is more of a smoke test
        let result = check_git_available();
        assert!(result.is_ok(), "git should be available for testing");
    }
}
