use crate::error::{KayfabeError, Result};
use crate::git::KayfabeRoot;
use git2::{BranchType, Repository};
use rand::Rng;
use std::path::{Path, PathBuf};

pub struct GitRepo {
    repo: Repository,
    root: PathBuf,
    layout_root: PathBuf,
}

impl GitRepo {
    pub fn discover(path: &Path) -> Result<Self> {
        let layout_root = KayfabeRoot::discover(path)?;
        let discover_path = if Self::is_worktree_layout_root(&layout_root) {
            layout_root.join("main")
        } else {
            layout_root.clone()
        };

        // Try to discover from the path, or if that fails, try to open it directly
        let repo = if discover_path.join(".git").exists() {
            Repository::open(&discover_path).or_else(|_| Repository::discover(&discover_path))
        } else {
            Repository::discover(&discover_path)
        }
        .map_err(|e| {
            KayfabeError::Other(format!(
                "Failed to open repository at {:?}: {}",
                path.display(),
                e
            ))
        })?;

        let root = repo
            .workdir()
            .ok_or_else(|| KayfabeError::Other("Bare repository not supported".to_string()))?
            .to_path_buf();

        Ok(Self {
            repo,
            root,
            layout_root,
        })
    }

    fn is_worktree_layout_root(path: &Path) -> bool {
        path.join("main").is_dir() && path.join("wt").is_dir() && path.join(".kayfabe").is_dir()
    }

    pub fn get_default_branch(&self) -> Result<String> {
        let main_exists = self.branch_exists("main")?;
        let master_exists = self.branch_exists("master")?;

        match (main_exists, master_exists) {
            (true, _) => Ok("main".to_string()),
            (false, true) => Ok("master".to_string()),
            (false, false) => Ok("main".to_string()),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn layout_root(&self) -> &Path {
        &self.layout_root
    }

    pub fn is_worktree_layout(&self) -> bool {
        self.layout_root.join("main").is_dir() && self.layout_root.join("wt").is_dir()
    }

    pub fn convert_to_worktree_layout(&self) -> Result<()> {
        if self.is_worktree_layout() {
            return Ok(());
        }

        if (self.root.join("main")).is_dir() {
            return Err(KayfabeError::Other(
                "Repo is a git checkout AND already has a main/ dir. Refusing to guess."
                    .to_string(),
            ));
        }

        let parent = self
            .root
            .parent()
            .ok_or_else(|| KayfabeError::Other("Cannot get parent directory".to_string()))?;

        let repo_name = self
            .root
            .file_name()
            .ok_or_else(|| KayfabeError::Other("Cannot get repo name".to_string()))?;

        let mut rng = rand::thread_rng();
        let random_num: u32 = rng.gen_range(10000..99999);
        let tmp_move = parent.join(format!(
            ".{}.tmp-move.{}",
            repo_name.to_string_lossy(),
            random_num
        ));

        std::fs::rename(&self.root, &tmp_move)?;
        std::fs::create_dir(&self.root)?;
        std::fs::rename(&tmp_move, self.root.join("main"))?;
        std::fs::create_dir(self.root.join("wt"))?;
        Ok(())
    }

    pub fn branch_exists(&self, name: &str) -> Result<bool> {
        Ok(self.repo.find_branch(name, BranchType::Local).is_ok())
    }

    pub fn remote_branch_exists(&self, name: &str) -> Result<bool> {
        Ok(self
            .repo
            .find_branch(&format!("origin/{}", name), BranchType::Remote)
            .is_ok())
    }

    pub fn fetch(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin").ok();
        if let Some(ref mut remote) = remote {
            let _ = remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], None, None);
        }
        Ok(())
    }

    pub fn create_worktree(&self, name: &str, base_branch: &str) -> Result<PathBuf> {
        let wt_path = if self.is_worktree_layout() {
            self.layout_root.join("wt").join(name)
        } else {
            self.root.join(name)
        };

        if wt_path.exists() {
            return Err(KayfabeError::WorktreeExists(name.to_string()));
        }

        let branch_exists = self.branch_exists(name)?;

        if branch_exists {
            std::process::Command::new("git")
                .args(["worktree", "add", wt_path.to_str().unwrap(), name])
                .current_dir(&self.root)
                .output()?;
        } else {
            std::process::Command::new("git")
                .args([
                    "worktree",
                    "add",
                    "-b",
                    name,
                    wt_path.to_str().unwrap(),
                    base_branch,
                ])
                .current_dir(&self.root)
                .output()?;
        }

        Ok(wt_path)
    }

    pub fn list_worktrees(&self) -> Result<Vec<PathBuf>> {
        let output = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&self.root)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();

        for line in stdout.lines() {
            if line.starts_with("worktree ") {
                let path = line.strip_prefix("worktree ").unwrap();
                worktrees.push(PathBuf::from(path));
            }
        }

        Ok(worktrees)
    }

    pub fn remove_worktree(&self, path: &Path) -> Result<()> {
        std::process::Command::new("git")
            .args(["worktree", "remove", path.to_str().unwrap()])
            .current_dir(&self.root)
            .output()?;

        Ok(())
    }

    pub fn remove_worktree_force(&self, path: &Path) -> Result<()> {
        std::process::Command::new("git")
            .args(["worktree", "remove", "--force", path.to_str().unwrap()])
            .current_dir(&self.root)
            .output()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_from_worktree_subdir_uses_layout_root() {
        let temp_dir = TempDir::new().unwrap();
        let layout_root = temp_dir.path().canonicalize().unwrap();

        fs::create_dir(layout_root.join(".kayfabe")).unwrap();
        fs::create_dir(layout_root.join("wt")).unwrap();

        let main_dir = layout_root.join("main");
        fs::create_dir(&main_dir).unwrap();
        Repository::init(&main_dir).unwrap();

        let subdir = layout_root.join("wt").join("feature").join("src");
        fs::create_dir_all(&subdir).unwrap();

        let repo = GitRepo::discover(&subdir).unwrap();
        assert_eq!(repo.layout_root(), layout_root.as_path());
        assert_eq!(repo.root(), main_dir.as_path());
        assert!(repo.is_worktree_layout());
    }
}
