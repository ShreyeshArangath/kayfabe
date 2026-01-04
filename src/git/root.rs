use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct KayfabeRoot;

impl KayfabeRoot {
    /// Find the kayfabe root by looking for .kayfabe directory
    pub fn discover(start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path.canonicalize()?;

        loop {
            let kayfabe_dir = current.join(".kayfabe");
            if kayfabe_dir.exists() && kayfabe_dir.is_dir() {
                return Ok(current);
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }

        Err(crate::error::KayfabeError::Other(
            "Not in a kayfabe-managed repository. Run 'kayfabe init' first.".to_string(),
        ))
    }

    /// Check if we're in a kayfabe-managed repository
    pub fn is_kayfabe_repo(path: &Path) -> bool {
        Self::discover(path).is_ok()
    }

    /// Get the worktree directory relative to kayfabe root
    pub fn worktree_dir(root: &Path) -> PathBuf {
        root.join("wt")
    }

    /// Get the main directory relative to kayfabe root
    pub fn main_dir(root: &Path) -> PathBuf {
        root.join("main")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_kayfabe_root() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().canonicalize().unwrap();

        // Create .kayfabe directory
        fs::create_dir(root.join(".kayfabe")).unwrap();

        // Test discovery from root
        let discovered = KayfabeRoot::discover(&root).unwrap();
        assert_eq!(discovered, root);

        // Test discovery from subdirectory
        let subdir = root.join("src").join("cli");
        fs::create_dir_all(&subdir).unwrap();
        let discovered = KayfabeRoot::discover(&subdir).unwrap();
        assert_eq!(discovered, root);
    }

    #[test]
    fn test_discover_from_worktree_subdir() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().canonicalize().unwrap();

        fs::create_dir(root.join(".kayfabe")).unwrap();
        fs::create_dir_all(root.join("wt").join("feature").join("src")).unwrap();

        let subdir = root.join("wt").join("feature").join("src");
        let discovered = KayfabeRoot::discover(&subdir).unwrap();
        assert_eq!(discovered, root);
    }

    #[test]
    fn test_discover_not_in_kayfabe_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = KayfabeRoot::discover(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_kayfabe_repo() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Should be false without .kayfabe
        assert!(!KayfabeRoot::is_kayfabe_repo(root));

        // Should be true with .kayfabe
        fs::create_dir(root.join(".kayfabe")).unwrap();
        assert!(KayfabeRoot::is_kayfabe_repo(root));
    }

    #[test]
    fn test_worktree_dir() {
        let root = Path::new("/test/repo");
        let wt_dir = KayfabeRoot::worktree_dir(root);
        assert_eq!(wt_dir, Path::new("/test/repo/wt"));
    }

    #[test]
    fn test_main_dir() {
        let root = Path::new("/test/repo");
        let main = KayfabeRoot::main_dir(root);
        assert_eq!(main, Path::new("/test/repo/main"));
    }
}
