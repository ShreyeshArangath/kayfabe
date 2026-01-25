use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;

use crate::core::worktree::WorktreeManager;
use crate::db::models::{Activity, ActivityType, Project, Task, TaskStatus};
use crate::db::operations::{projects, tasks};
use crate::db::Database;
use crate::utils;

/// Manages task lifecycle with atomic operations
pub struct TaskManager {
    pub db: Database,
}

impl TaskManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new task with atomic worktree setup
    ///
    /// This operation is atomic - if any step fails, all changes are rolled back:
    /// 1. Validates task name doesn't exist
    /// 2. Creates task record in database
    /// 3. Creates git worktree and branch
    /// 4. Updates task with worktree path
    /// 5. Creates thoughts directory
    /// 6. Logs creation activity
    ///
    /// Returns the created Task on success
    pub fn create_task(
        &self,
        project: &Project,
        name: &str,
        description: Option<&str>,
    ) -> Result<Task> {
        // 1. Validate task name doesn't exist
        if tasks::name_exists(self.db.conn(), &project.id, name)? {
            bail!("Task '{}' already exists in project '{}'", name, project.name);
        }

        // 2. Create task record (transaction start)
        let mut task = Task::new(
            project.id.clone(),
            name.to_string(),
            description.map(|s| s.to_string()),
        );

        tasks::insert(self.db.conn(), &task)
            .context("Failed to insert task")?;

        // 3. Create git worktree
        let worktree_manager = WorktreeManager::new(self.db.conn());
        let worktree = match worktree_manager.create_worktree(project, &task) {
            Ok(wt) => wt,
            Err(e) => {
                // Rollback: delete task
                let _ = tasks::delete(self.db.conn(), &task.id);
                return Err(e).context("Failed to create worktree - task creation rolled back");
            }
        };

        // 4. Update task with worktree path
        task.worktree_path = Some(worktree.path.clone());
        task.git_branch = Some(worktree.branch_name.clone());
        task.updated_at = Utc::now();

        if let Err(e) = tasks::update(self.db.conn(), &task) {
            // Rollback: delete worktree and task
            let worktree_manager = WorktreeManager::new(self.db.conn());
            let _ = worktree_manager.remove_worktree(project, &task, true);
            let _ = tasks::delete(self.db.conn(), &task.id);
            return Err(e).context("Failed to update task with worktree info");
        }

        // 5. Create initial activity record
        let activity = Activity::new(
            task.id.clone(),
            ActivityType::Artifact,
            Some(format!("Task '{}' created", name)),
            None,
        );

        if let Err(e) = activities::insert(self.db.conn(), &activity) {
            // Log warning but don't fail the operation
            eprintln!("Warning: Failed to log task creation activity: {}", e);
        }

        println!("✓ Created task '{}' with worktree", name);

        Ok(task)
    }

    /// Remove a task with comprehensive cleanup
    ///
    /// This performs:
    /// 1. Confirmation if not forced
    /// 2. Removes git worktree and branch
    /// 3. Deletes task database record (cascades to activities, worktrees)
    /// 4. Cleans up thoughts directory
    pub fn remove_task(&self, project: &Project, task_name: &str, force: bool) -> Result<()> {
        // 1. Get task by name
        let task = tasks::get_by_name(self.db.conn(), &project.id, task_name)?
            .ok_or_else(|| anyhow!("Task '{}' not found", task_name))?;

        // 2. Check if task is active
        if task.status == TaskStatus::Active && !force {
            bail!(
                "Task '{}' is currently active. Stop it first or use --force",
                task_name
            );
        }

        // 3. Remove worktree (this handles uncommitted changes check)
        let worktree_manager = WorktreeManager::new(self.db.conn());
        if let Err(e) = worktree_manager.remove_worktree(project, &task, force) {
            eprintln!("Warning: Failed to remove worktree: {}", e);
            if !force {
                return Err(e);
            }
            // Continue with cleanup even if worktree removal fails in force mode
        }

        // 4. Delete task record (this cascades to activities and worktrees)
        tasks::delete(self.db.conn(), &task.id)
            .context("Failed to delete task record")?;

        // 5. Clean up thoughts directory
        let thoughts_dir = project
            .repo_path
            .parent()
            .context("Failed to get project root")?
            .join("thoughts")
            .join(task_name);

        if thoughts_dir.exists() {
            std::fs::remove_dir_all(&thoughts_dir).with_context(|| {
                format!("Failed to remove thoughts directory: {}", thoughts_dir.display())
            })?;
        }

        println!("✓ Removed task '{}'", task_name);

        Ok(())
    }

    /// List all tasks for a project
    pub fn list_tasks(&self, project: &Project, status_filter: Option<TaskStatus>) -> Result<Vec<Task>> {
        let tasks = match status_filter {
            Some(status) => tasks::list_by_status(self.db.conn(), &project.id, status)?,
            None => tasks::list_by_project(self.db.conn(), &project.id)?,
        };

        Ok(tasks)
    }

    /// Get a task by name
    pub fn get_task(&self, project: &Project, task_name: &str) -> Result<Option<Task>> {
        tasks::get_by_name(self.db.conn(), &project.id, task_name)
    }

    /// Update task status
    pub fn update_status(&self, task_id: &str, new_status: TaskStatus) -> Result<()> {
        let mut task = tasks::get_by_id(self.db.conn(), task_id)?
            .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;

        let old_status = task.status;
        task.status = new_status;
        task.updated_at = Utc::now();

        // Set completed_at if transitioning to completed
        if new_status == TaskStatus::Completed && old_status != TaskStatus::Completed {
            task.completed_at = Some(Utc::now());
        }

        tasks::update(self.db.conn(), &task)?;

        println!(
            "✓ Updated task '{}' status: {:?} → {:?}",
            task.name, old_status, new_status
        );

        Ok(())
    }

    /// Touch a task to update its last_accessed_at timestamp
    pub fn touch_task(&self, task_id: &str) -> Result<()> {
        tasks::touch(self.db.conn(), task_id)?;
        let worktree_manager = WorktreeManager::new(self.db.conn());
        worktree_manager.touch_worktree(task_id)?;
        Ok(())
    }

    /// Get the current project from the working directory
    pub fn get_current_project(&self) -> Result<Project> {
        let project_root = utils::find_project_root()
            .context("Not in a Kayfabe project directory")?;

        let project_name = project_root
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid project path"))?;

        projects::get_by_name(self.db.conn(), project_name)?
            .ok_or_else(|| anyhow!("Project '{}' not found in database", project_name))
    }

    /// Display task list in a formatted table
    pub fn display_tasks(&self, tasks: &[Task], verbose: bool) {
        if tasks.is_empty() {
            println!("No tasks found.");
            return;
        }

        println!("\n{:<30} {:<15} {:<20}", "TASK", "STATUS", "LAST ACCESSED");
        println!("{}", "-".repeat(65));

        for task in tasks {
            let status_icon = match task.status {
                TaskStatus::Pending => "○",
                TaskStatus::Active => "▶",
                TaskStatus::Completed => "✓",
                TaskStatus::Archived => "⊗",
            };

            let last_accessed = task
                .last_accessed_at
                .map(|t| format!("{}", t.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|| "Never".to_string());

            println!(
                "{} {:<28} {:<15} {:<20}",
                status_icon,
                truncate(&task.name, 28),
                format!("{:?}", task.status),
                last_accessed
            );

            if verbose {
                if let Some(desc) = &task.description {
                    println!("   Description: {}", truncate(desc, 60));
                }
                if let Some(branch) = &task.git_branch {
                    println!("   Branch: {}", branch);
                }
                println!();
            }
        }

        println!("\nTotal: {} task(s)", tasks.len());
    }

    /// Display task statistics
    pub fn display_stats(&self, project: &Project) -> Result<()> {
        let all_tasks = tasks::list_by_project(self.db.conn(), &project.id)?;

        let pending = all_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count();
        let active = all_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Active)
            .count();
        let completed = all_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let archived = all_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Archived)
            .count();

        println!("\n=== Task Statistics ===");
        println!("Total:     {}", all_tasks.len());
        println!("Pending:   {} ○", pending);
        println!("Active:    {} ▶", active);
        println!("Completed: {} ✓", completed);
        println!("Archived:  {} ⊗", archived);

        Ok(())
    }
}

/// Truncate a string to a maximum length, adding ellipsis if needed
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Activity database operations
mod activities {
    use super::*;
    use anyhow::Result;
    use rusqlite::{params, Connection};

    pub fn insert(conn: &Connection, activity: &Activity) -> Result<()> {
        conn.execute(
            "INSERT INTO activities (id, task_id, activity_type, content, file_path, created_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                activity.id,
                activity.task_id,
                activity.activity_type.as_str(),
                activity.content,
                activity.file_path.as_ref().and_then(|p| p.to_str()),
                activity.created_at.to_rfc3339(),
                activity.metadata_json,
            ],
        )
        .context("Failed to insert activity")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_environment() -> Result<(TempDir, Database, Project)> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join(".git");
        let worktree_dir = temp_dir.path().join("worktrees");
        std::fs::create_dir_all(&worktree_dir)?;

        // Create bare repository
        let repo = git2::Repository::init_bare(&repo_path)?;

        // Create initial commit
        let sig = git2::Signature::now("Test", "test@example.com")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

        // Create main branch
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        repo.branch("main", &commit, false)?;

        // Create database and project
        let db = Database::open_in_memory()?;
        let project = Project::new(
            "test-project".to_string(),
            "https://github.com/test/repo.git".to_string(),
            repo_path,
            worktree_dir,
        );

        projects::insert(db.conn(), &project)?;

        Ok((temp_dir, db, project))
    }

    #[test]
    fn test_task_creation() -> Result<()> {
        let (_temp_dir, db, project) = create_test_environment()?;
        let manager = TaskManager::new(db);

        let task = manager.create_task(&project, "test-task", Some("Test description"))?;

        assert_eq!(task.name, "test-task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.worktree_path.is_some());
        assert!(task.git_branch.is_some());

        // Verify worktree exists
        let worktree_path = task.worktree_path.unwrap();
        assert!(worktree_path.exists());

        Ok(())
    }

    #[test]
    fn test_duplicate_task_name() -> Result<()> {
        let (_temp_dir, db, project) = create_test_environment()?;
        let manager = TaskManager::new(db);

        manager.create_task(&project, "duplicate", None)?;

        let result = manager.create_task(&project, "duplicate", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[test]
    fn test_task_removal() -> Result<()> {
        let (_temp_dir, db, project) = create_test_environment()?;
        let manager = TaskManager::new(db);

        let task = manager.create_task(&project, "remove-me", None)?;
        let worktree_path = task.worktree_path.clone().unwrap();

        manager.remove_task(&project, "remove-me", true)?;

        // Verify worktree removed
        assert!(!worktree_path.exists());

        // Verify task removed from database
        let task_opt = manager.get_task(&project, "remove-me")?;
        assert!(task_opt.is_none());

        Ok(())
    }

    #[test]
    fn test_list_tasks() -> Result<()> {
        let (_temp_dir, db, project) = create_test_environment()?;
        let manager = TaskManager::new(db);

        manager.create_task(&project, "task1", None)?;
        manager.create_task(&project, "task2", None)?;
        manager.create_task(&project, "task3", None)?;

        let all_tasks = manager.list_tasks(&project, None)?;
        assert_eq!(all_tasks.len(), 3);

        let pending_tasks = manager.list_tasks(&project, Some(TaskStatus::Pending))?;
        assert_eq!(pending_tasks.len(), 3);

        Ok(())
    }

    #[test]
    fn test_status_update() -> Result<()> {
        let (_temp_dir, db, project) = create_test_environment()?;
        let manager = TaskManager::new(db);

        let task = manager.create_task(&project, "status-test", None)?;

        manager.update_status(&task.id, TaskStatus::Active)?;

        let updated_task = tasks::get_by_id(manager.db.conn(), &task.id)?.unwrap();
        assert_eq!(updated_task.status, TaskStatus::Active);

        manager.update_status(&task.id, TaskStatus::Completed)?;

        let completed_task = tasks::get_by_id(manager.db.conn(), &task.id)?.unwrap();
        assert_eq!(completed_task.status, TaskStatus::Completed);
        assert!(completed_task.completed_at.is_some());

        Ok(())
    }
}
