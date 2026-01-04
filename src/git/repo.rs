use crate::error::{KayfabeError, Result};
use git2::{BranchType, Repository};
use std::path::{Path, PathBuf};

pub struct GitRepo {
    repo: Repository,
    root: PathBuf,
}

impl GitRepo {
    pub fn discover(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path)
            .map_err(|_| KayfabeError::NotARepository(path.display().to_string()))?;

        let root = repo
            .workdir()
            .ok_or_else(|| KayfabeError::Other("Bare repository not supported".to_string()))?
            .to_path_buf();

        Ok(Self { repo, root })
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

    pub fn is_worktree_layout(&self) -> bool {
        self.root.join("main").is_dir() && self.root.join("wt").is_dir()
    }

    pub fn convert_to_worktree_layout(&self) -> Result<()> {
        if self.is_worktree_layout() {
            return Ok(());
        }

        let main_dir = self.root.join("main");
        let wt_dir = self.root.join("wt");

        std::fs::create_dir_all(&wt_dir)?;

        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();

            if name == ".git" || name == "main" || name == "wt" {
                continue;
            }

            let dest = main_dir.join(name.as_ref());
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(&path, &dest)?;
        }

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
            self.root.join("wt").join(name)
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
