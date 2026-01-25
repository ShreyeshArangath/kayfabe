use anyhow::{bail, Context, Result};
use git2::Repository;
use std::fs;
use std::path::PathBuf;

use crate::db::models::{Project, Task, Worktree};
use crate::db::operations::worktrees;
use crate::utils::{git, sanitize_name};

/// Manages git worktree lifecycle and operations
pub struct WorktreeManager<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> WorktreeManager<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Create a new git worktree for a task
    ///
    /// This performs the following operations atomically:
    /// 1. Creates a new git branch (<task-name>)
    /// 2. Adds a git worktree at the specified path
    /// 3. Inserts worktree record in database
    /// 4. Creates thoughts directory structure
    ///
    /// Returns the created Worktree on success
    pub fn create_worktree(&self, project: &Project, task: &Task) -> Result<Worktree> {
        // 1. Generate branch name and worktree path
        let branch_name = sanitize_name(&task.name);
        let worktree_path = project.worktree_dir.join(&task.name);

        // 2. Validate worktree doesn't already exist
        if worktree_path.exists() {
            bail!(
                "Worktree path already exists: {}",
                worktree_path.display()
            );
        }

        // 3. Open repository and create branch
        let repo = Repository::open(&project.repo_path).with_context(|| {
            format!(
                "Failed to open repository at {}",
                project.repo_path.display()
            )
        })?;

        // Get default branch to branch from
        let default_branch = git::detect_default_branch(&repo)
            .context("Failed to detect default branch")?;

        // Find the commit to branch from
        let head_ref = repo
            .find_reference(&format!("refs/heads/{}", default_branch))
            .with_context(|| format!("Failed to find branch: {}", default_branch))?;

        let commit = head_ref
            .peel_to_commit()
            .context("Failed to get commit from branch")?;

        // Create new branch
        repo.branch(&branch_name, &commit, false)
            .with_context(|| format!("Failed to create branch: {}", branch_name))?;

        // 4. Add worktree using git CLI (git2-rs doesn't support this directly)
        git::add_worktree(&project.repo_path, &worktree_path, &branch_name)
            .context("Failed to add git worktree")?;

        // 5. Create worktree database record
        let worktree = Worktree::new(task.id.clone(), worktree_path.clone(), branch_name);

        worktrees::insert(self.conn, &worktree)
            .context("Failed to insert worktree record")?;

        // 6. Create thoughts directory structure
        self.create_thoughts_directory(project, task)
            .context("Failed to create thoughts directory")?;

        println!(
            "✓ Created worktree for task '{}' at {}",
            task.name,
            worktree_path.display()
        );

        Ok(worktree)
    }

    /// Remove a git worktree for a task
    ///
    /// This performs cleanup:
    /// 1. Removes git worktree (with --force to handle uncommitted changes)
    /// 2. Deletes worktree database record
    /// 3. Cleans up thoughts directory (optional, based on policy)
    ///
    /// Returns Ok if successful, even if worktree was already removed
    pub fn remove_worktree(
        &self,
        project: &Project,
        task: &Task,
        force: bool,
    ) -> Result<()> {
        // 1. Get worktree record
        let worktree = worktrees::get_by_task_id(self.conn, &task.id)
            .context("Failed to query worktree")?;

        if worktree.is_none() {
            println!("⚠️  No worktree found for task '{}'", task.name);
            return Ok(());
        }

        let worktree = worktree.unwrap();

        // 2. Check for uncommitted changes (unless force is true)
        if !force && worktree.path.exists() {
            if git::has_uncommitted_changes(&worktree.path)? {
                bail!(
                    "Worktree has uncommitted changes. Use --force to remove anyway.\n\
                     Path: {}",
                    worktree.path.display()
                );
            }
        }

        // 3. Remove worktree using git CLI
        if worktree.path.exists() {
            git::remove_worktree(&project.repo_path, &worktree.path, force)
                .context("Failed to remove git worktree")?;
        } else {
            println!(
                "⚠️  Worktree path doesn't exist (may be orphaned): {}",
                worktree.path.display()
            );
            // Prune stale worktree references
            git::prune_worktrees(&project.repo_path)
                .context("Failed to prune worktrees")?;
        }

        // 4. Delete worktree database record
        worktrees::delete_by_task_id(self.conn, &task.id)
            .context("Failed to delete worktree record")?;

        println!("✓ Removed worktree for task '{}'", task.name);

        Ok(())
    }

    /// List all worktrees for a project
    pub fn list_worktrees(&self, project: &Project) -> Result<Vec<(Task, Worktree)>> {
        // Get all worktrees from database
        let worktrees = worktrees::list_all(self.conn)?;

        // Get tasks for each worktree
        let mut result = Vec::new();
        for worktree in worktrees {
            if let Some(task) =
                crate::db::operations::tasks::get_by_id(self.conn, &worktree.task_id)?
            {
                if task.project_id == project.id {
                    result.push((task, worktree));
                }
            }
        }

        Ok(result)
    }

    /// Verify worktree integrity - check that filesystem matches database
    pub fn verify_worktree(&self, worktree: &Worktree) -> Result<bool> {
        // Check if worktree path exists
        if !worktree.path.exists() {
            return Ok(false);
        }

        // Check if it's a valid git repository
        if !git::is_git_repository(&worktree.path) {
            return Ok(false);
        }

        // Check if branch matches
        let repo = Repository::open(&worktree.path)?;
        let current_branch = git::get_current_branch(&repo)?;

        Ok(current_branch == worktree.branch_name)
    }

    /// Touch a worktree to update its last_accessed_at timestamp
    pub fn touch_worktree(&self, task_id: &str) -> Result<()> {
        worktrees::touch(self.conn, task_id)?;
        Ok(())
    }

    /// Create thoughts directory structure for a task
    fn create_thoughts_directory(&self, project: &Project, task: &Task) -> Result<()> {
        let thoughts_dir = project.repo_path.parent()
            .context("Failed to get project root")?
            .join("thoughts")
            .join(&task.name);

        fs::create_dir_all(&thoughts_dir).with_context(|| {
            format!("Failed to create thoughts directory: {}", thoughts_dir.display())
        })?;

        // Create a .gitkeep file to ensure directory is tracked
        let gitkeep_path = thoughts_dir.join(".gitkeep");
        fs::write(&gitkeep_path, "").with_context(|| {
            format!("Failed to create .gitkeep file: {}", gitkeep_path.display())
        })?;

        Ok(())
    }

    /// Find orphaned worktrees (filesystem entries without database records)
    pub fn find_orphaned_worktrees(&self, project: &Project) -> Result<Vec<PathBuf>> {
        let mut orphans = Vec::new();

        // Scan worktree directory
        if !project.worktree_dir.exists() {
            return Ok(orphans);
        }

        for entry in fs::read_dir(&project.worktree_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            // Check if there's a database record for this path
            let path_str = path.to_str().context("Invalid UTF-8 in path")?;
            if worktrees::get_by_path(self.conn, path_str)?.is_none() {
                orphans.push(path);
            }
        }

        Ok(orphans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::path::Path;
    use tempfile::TempDir;

    // Helper to create a test git repository
    fn create_test_repo(path: &Path) -> Result<()> {
        let repo = Repository::init_bare(path)?;

        // Create an initial commit (required for branching)
        let sig = git2::Signature::now("Test", "test@example.com")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        )?;

        // Create main branch
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        repo.branch("main", &commit, false)?;

        Ok(())
    }

    #[test]
    fn test_worktree_lifecycle() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join(".git");
        let worktree_dir = temp_dir.path().join("worktrees");
        fs::create_dir_all(&worktree_dir)?;

        // Create test repository
        create_test_repo(&repo_path)?;

        // Create database
        let db = Database::open_in_memory()?;
        let conn = db.conn();

        // Create manager with connection reference
        let manager = WorktreeManager::new(conn);

        // Create test project
        let project = Project::new(
            "test".to_string(),
            "https://github.com/test/repo.git".to_string(),
            repo_path.clone(),
            worktree_dir.clone(),
        );
        crate::db::operations::projects::insert(conn, &project)?;

        // Create test task
        let task = Task::new(
            project.id.clone(),
            "test-task".to_string(),
            Some("Test task".to_string()),
        );
        crate::db::operations::tasks::insert(conn, &task)?;

        // Create worktree
        let worktree = manager.create_worktree(&project, &task)?;
        assert_eq!(worktree.branch_name, "test-task");
        assert!(worktree.path.exists());

        // Verify worktree
        let is_valid = manager.verify_worktree(&worktree)?;
        assert!(is_valid);

        // List worktrees
        let worktrees_list = manager.list_worktrees(&project)?;
        assert_eq!(worktrees_list.len(), 1);

        // Remove worktree
        manager.remove_worktree(&project, &task, true)?;
        assert!(!worktree.path.exists());

        // Verify removed from database
        let worktree_opt = worktrees::get_by_task_id(conn, &task.id)?;
        assert!(worktree_opt.is_none());

        Ok(())
    }
}
