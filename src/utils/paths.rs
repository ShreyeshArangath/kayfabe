use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

/// Get the global Kayfabe directory (~/.kayfabe)
pub fn get_kayfabe_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".kayfabe"))
}

/// Get the global database path (~/.kayfabe/kayfabe.db)
pub fn get_database_path() -> Result<PathBuf> {
    Ok(get_kayfabe_dir()?.join("kayfabe.db"))
}

/// Get the global config path (~/.kayfabe/config.toml)
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_kayfabe_dir()?.join("config.toml"))
}

/// Get the templates directory (~/.kayfabe/templates)
pub fn get_templates_dir() -> Result<PathBuf> {
    Ok(get_kayfabe_dir()?.join("templates"))
}

/// Initialize the global Kayfabe directory structure
pub fn init_kayfabe_dir() -> Result<()> {
    let kayfabe_dir = get_kayfabe_dir()?;

    // Create main directory
    if !kayfabe_dir.exists() {
        fs::create_dir_all(&kayfabe_dir)
            .context("Failed to create ~/.kayfabe directory")?;
    }

    // Create templates directory
    let templates_dir = get_templates_dir()?;
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)
            .context("Failed to create templates directory")?;
    }

    Ok(())
}

/// Check if a directory is a valid Kayfabe project
pub fn is_kayfabe_project(path: &Path) -> bool {
    path.join(".kayfabe").exists() || path.join("kayfabe.md").exists()
}

/// Find the root of the current Kayfabe project by searching upwards
pub fn find_project_root() -> Result<PathBuf> {
    let current = std::env::current_dir()
        .context("Could not determine current directory")?;

    let mut path = current.as_path();
    loop {
        if is_kayfabe_project(path) {
            return Ok(path.to_path_buf());
        }

        match path.parent() {
            Some(parent) => path = parent,
            None => anyhow::bail!("Not in a Kayfabe project directory"),
        }
    }
}

/// Sanitize a task name for use in file paths and branch names
pub fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' | '_' => '-',
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Feature X"), "feature-x");
        assert_eq!(sanitize_name("Fix Bug #123"), "fix-bug-123");
        assert_eq!(sanitize_name("Add-New_Feature"), "add-new-feature");
        assert_eq!(sanitize_name("Test   Multiple   Spaces"), "test-multiple-spaces");
    }
}
