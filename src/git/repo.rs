use crate::error::{KayfabeError, Result};
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
        let is_layout_root = Self::is_worktree_layout_root(path);
        let discover_path = if is_layout_root {
            path.join("main")
        } else {
            path.to_path_buf()
        };

        let repo = Repository::discover(&discover_path)
            .map_err(|_| KayfabeError::NotARepository(path.display().to_string()))?;

        let root = repo
            .workdir()
            .ok_or_else(|| KayfabeError::Other("Bare repository not supported".to_string()))?
            .to_path_buf();

        let layout_root = if is_layout_root {
            path.to_path_buf()
        } else {
            root.clone()
        };

        Ok(Self { repo, root, layout_root })
    }

    fn is_worktree_layout_root(path: &Path) -> bool {
        path.join("main").is_dir() && path.join("wt").is_dir()
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
                "Repo is a git checkout AND already has a main/ dir. Refusing to guess.".to_string(),
            ));
        }

        let parent = self.root.parent().ok_or_else(|| {
            KayfabeError::Other("Cannot get parent directory".to_string())
        })?;

        let repo_name = self.root.file_name().ok_or_else(|| {
            KayfabeError::Other("Cannot get repo name".to_string())
        })?;

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

    fn setup_git_repo(temp_dir: &Path, branch_name: &str) -> Result<()> {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir)
            .output()?;

        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp_dir)
            .output()?;

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir)
            .output()?;

        std::process::Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(temp_dir)
            .output()?;

        fs::write(temp_dir.join("README.md"), "test content")?;
        fs::create_dir(temp_dir.join("src"))?;
        fs::write(temp_dir.join("src/main.rs"), "fn main() {}")?;

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir)
            .output()?;

        std::process::Command::new("git")
            .args(["commit", "-m", "initial commit"])
            .current_dir(temp_dir)
            .output()?;

        Ok(())
    }

    #[test]
    fn test_convert_master_branch_repo() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "master")?;

        let repo = GitRepo::discover(&repo_path)?;
        assert_eq!(repo.get_default_branch()?, "master");
        assert!(!repo.is_worktree_layout());

        repo.convert_to_worktree_layout()?;

        assert!(repo.is_worktree_layout());
        assert!(repo_path.join("main").is_dir());
        assert!(repo_path.join("wt").is_dir());
        assert!(repo_path.join("main/.git").is_dir());
        assert!(repo_path.join("main/README.md").exists());
        assert!(repo_path.join("main/src/main.rs").exists());

        let main_repo = GitRepo::discover(&repo_path)?;
        assert_eq!(main_repo.get_default_branch()?, "master");

        Ok(())
    }

    #[test]
    fn test_convert_main_branch_repo() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "main")?;

        let repo = GitRepo::discover(&repo_path)?;
        assert_eq!(repo.get_default_branch()?, "main");
        assert!(!repo.is_worktree_layout());

        repo.convert_to_worktree_layout()?;

        assert!(repo.is_worktree_layout());
        assert!(repo_path.join("main").is_dir());
        assert!(repo_path.join("wt").is_dir());
        assert!(repo_path.join("main/.git").is_dir());
        assert!(repo_path.join("main/README.md").exists());
        assert!(repo_path.join("main/src/main.rs").exists());

        let main_repo = GitRepo::discover(&repo_path)?;
        assert_eq!(main_repo.get_default_branch()?, "main");

        Ok(())
    }

    #[test]
    fn test_discover_from_layout_root_master() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "master")?;

        let repo = GitRepo::discover(&repo_path)?;
        repo.convert_to_worktree_layout()?;

        let discovered = GitRepo::discover(&repo_path)?;
        assert_eq!(discovered.get_default_branch()?, "master");
        assert!(discovered.is_worktree_layout());
        assert_eq!(discovered.root().canonicalize()?, repo_path.join("main").canonicalize()?);

        Ok(())
    }

    #[test]
    fn test_discover_from_layout_root_main() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "main")?;

        let repo = GitRepo::discover(&repo_path)?;
        repo.convert_to_worktree_layout()?;

        let discovered = GitRepo::discover(&repo_path)?;
        assert_eq!(discovered.get_default_branch()?, "main");
        assert!(discovered.is_worktree_layout());
        assert_eq!(discovered.root().canonicalize()?, repo_path.join("main").canonicalize()?);

        Ok(())
    }

    #[test]
    fn test_idempotent_conversion() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "master")?;

        let repo = GitRepo::discover(&repo_path)?;
        repo.convert_to_worktree_layout()?;

        let repo2 = GitRepo::discover(&repo_path)?;
        repo2.convert_to_worktree_layout()?;

        assert!(repo_path.join("main").is_dir());
        assert!(repo_path.join("wt").is_dir());
        assert!(!repo_path.join("main/main").exists());

        Ok(())
    }

    #[test]
    fn test_create_worktree_after_conversion_master() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "master")?;

        let repo = GitRepo::discover(&repo_path)?;
        repo.convert_to_worktree_layout()?;

        let repo_after = GitRepo::discover(&repo_path)?;
        let wt_path = repo_after.create_worktree("feature-test", "master")?;

        assert_eq!(wt_path.canonicalize()?, repo_path.join("wt/feature-test").canonicalize()?);
        assert!(wt_path.is_dir());
        assert!(wt_path.join(".git").exists());

        let branch_output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&wt_path)
            .output()?;

        let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
        assert_eq!(branch, "feature-test");

        Ok(())
    }

    #[test]
    fn test_create_worktree_after_conversion_main() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "main")?;

        let repo = GitRepo::discover(&repo_path)?;
        repo.convert_to_worktree_layout()?;

        let repo_after = GitRepo::discover(&repo_path)?;
        let wt_path = repo_after.create_worktree("feature-test", "main")?;

        assert_eq!(wt_path.canonicalize()?, repo_path.join("wt/feature-test").canonicalize()?);
        assert!(wt_path.is_dir());
        assert!(wt_path.join(".git").exists());

        let branch_output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&wt_path)
            .output()?;

        let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
        assert_eq!(branch, "feature-test");

        Ok(())
    }

    #[test]
    fn test_get_default_branch_master() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "master")?;

        let repo = GitRepo::discover(&repo_path)?;
        assert_eq!(repo.get_default_branch()?, "master");

        Ok(())
    }

    #[test]
    fn test_get_default_branch_main() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "main")?;

        let repo = GitRepo::discover(&repo_path)?;
        assert_eq!(repo.get_default_branch()?, "main");

        Ok(())
    }

    #[test]
    fn test_git_state_clean_after_conversion_master() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "master")?;

        let repo = GitRepo::discover(&repo_path)?;
        repo.convert_to_worktree_layout()?;

        let status_output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path.join("main"))
            .output()?;

        let status = String::from_utf8_lossy(&status_output.stdout);
        assert!(status.is_empty(), "Git status should be clean after conversion");

        Ok(())
    }

    #[test]
    fn test_git_state_clean_after_conversion_main() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path)?;

        setup_git_repo(&repo_path, "main")?;

        let repo = GitRepo::discover(&repo_path)?;
        repo.convert_to_worktree_layout()?;

        let status_output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path.join("main"))
            .output()?;

        let status = String::from_utf8_lossy(&status_output.stdout);
        assert!(status.is_empty(), "Git status should be clean after conversion");

        Ok(())
    }
}
