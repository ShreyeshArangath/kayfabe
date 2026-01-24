use anyhow::{Context, Result, bail};
use git2::Repository;
use std::path::Path;

/// Validates a git URL (SSH or HTTPS format)
pub fn validate_git_url(url: &str) -> Result<()> {
    // Check for SSH format (git@github.com:user/repo.git)
    let is_ssh = url.starts_with("git@") && url.contains(':');

    // Check for HTTPS format (https://github.com/user/repo.git)
    let is_https = url.starts_with("https://") || url.starts_with("http://");

    if !is_ssh && !is_https {
        bail!(
            "Invalid git URL format. Expected SSH (git@host:user/repo.git) or HTTPS (https://host/user/repo.git), got: {}",
            url
        );
    }

    // Basic validation - URL should contain a path component
    if is_ssh {
        if !url.contains('/') {
            bail!("SSH URL must contain a path (e.g., git@github.com:user/repo.git)");
        }
    } else {
        if url.split('/').count() < 4 {
            bail!("HTTPS URL must contain a full path (e.g., https://github.com/user/repo.git)");
        }
    }

    Ok(())
}

/// Detects the default branch of a repository (main or master)
pub fn detect_default_branch(repo: &Repository) -> Result<String> {
    // Try to find the HEAD reference
    let head = repo.head()
        .context("Failed to get repository HEAD")?;

    if let Some(name) = head.shorthand() {
        return Ok(name.to_string());
    }

    // If HEAD doesn't have a name, look for common default branches
    for branch_name in &["main", "master"] {
        if repo.find_branch(branch_name, git2::BranchType::Local).is_ok() {
            return Ok(branch_name.to_string());
        }
    }

    bail!("Could not detect default branch (tried 'main' and 'master')");
}

/// Validates that a repository has commits
pub fn validate_repo_has_commits(repo: &Repository) -> Result<()> {
    // Try to get HEAD
    let head = repo.head()
        .context("Repository HEAD not found - repository may be empty")?;

    // Try to resolve to a commit
    let _commit = head.peel_to_commit()
        .context("Repository has no commits")?;

    Ok(())
}

/// Clone a bare repository
pub fn clone_bare(url: &str, path: &Path) -> Result<Repository> {
    let mut fetch_options = git2::FetchOptions::new();
    let mut callbacks = git2::RemoteCallbacks::new();

    // Set up credential callback for SSH keys
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    builder.fetch_options(fetch_options);

    builder.clone(url, path)
        .with_context(|| format!("Failed to clone repository from {}", url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_git_url_ssh() {
        assert!(validate_git_url("git@github.com:user/repo.git").is_ok());
        assert!(validate_git_url("git@gitlab.com:group/subgroup/project.git").is_ok());
    }

    #[test]
    fn test_validate_git_url_https() {
        assert!(validate_git_url("https://github.com/user/repo.git").is_ok());
        assert!(validate_git_url("https://gitlab.com/group/project.git").is_ok());
    }

    #[test]
    fn test_validate_git_url_invalid() {
        assert!(validate_git_url("not-a-url").is_err());
        assert!(validate_git_url("ftp://example.com/repo").is_err());
        assert!(validate_git_url("git@github.com").is_err()); // Missing path
    }
}
