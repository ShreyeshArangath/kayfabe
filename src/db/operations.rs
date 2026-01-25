use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use super::models::{Project, Task, TaskStatus, Worktree};

/// Project database operations
pub mod projects {
    use super::*;

    /// Insert a new project
    pub fn insert(conn: &Connection, project: &Project) -> Result<()> {
        conn.execute(
            "INSERT INTO projects (id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                project.id,
                project.name,
                project.repo_path.to_str(),
                project.worktree_dir.to_str(),
                project.git_url,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
                project.config_json,
            ],
        )
        .context("Failed to insert project")?;

        Ok(())
    }

    /// Get a project by ID
    pub fn get_by_id(conn: &Connection, project_id: &str) -> Result<Option<Project>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json
             FROM projects WHERE id = ?1",
        )?;

        let project = stmt
            .query_row(params![project_id], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    repo_path: PathBuf::from(row.get::<_, String>(2)?),
                    worktree_dir: PathBuf::from(row.get::<_, String>(3)?),
                    git_url: row.get(4)?,
                    created_at: row
                        .get::<_, String>(5)?
                        .parse()
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    updated_at: row
                        .get::<_, String>(6)?
                        .parse()
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    config_json: row.get(7)?,
                })
            })
            .optional()
            .context("Failed to query project")?;

        Ok(project)
    }

    /// Get a project by name
    pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<Project>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json
             FROM projects WHERE name = ?1",
        )?;

        let project = stmt
            .query_row(params![name], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    repo_path: PathBuf::from(row.get::<_, String>(2)?),
                    worktree_dir: PathBuf::from(row.get::<_, String>(3)?),
                    git_url: row.get(4)?,
                    created_at: row
                        .get::<_, String>(5)?
                        .parse()
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    updated_at: row
                        .get::<_, String>(6)?
                        .parse()
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    config_json: row.get(7)?,
                })
            })
            .optional()
            .context("Failed to query project")?;

        Ok(project)
    }

    /// List all projects
    pub fn list_all(conn: &Connection) -> Result<Vec<Project>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json
             FROM projects ORDER BY created_at DESC",
        )?;

        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    repo_path: PathBuf::from(row.get::<_, String>(2)?),
                    worktree_dir: PathBuf::from(row.get::<_, String>(3)?),
                    git_url: row.get(4)?,
                    created_at: row
                        .get::<_, String>(5)?
                        .parse()
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    updated_at: row
                        .get::<_, String>(6)?
                        .parse()
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    config_json: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect projects")?;

        Ok(projects)
    }
}

/// Task database operations
pub mod tasks {
    use super::*;

    /// Insert a new task
    pub fn insert(conn: &Connection, task: &Task) -> Result<()> {
        conn.execute(
            "INSERT INTO tasks (id, project_id, name, description, status, worktree_path, git_branch,
                                last_accessed_at, cleanup_eligible, created_at, updated_at, completed_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                task.id,
                task.project_id,
                task.name,
                task.description,
                task.status.as_str(),
                task.worktree_path.as_ref().and_then(|p| p.to_str()),
                task.git_branch,
                task.last_accessed_at.map(|t| t.to_rfc3339()),
                task.cleanup_eligible,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.completed_at.map(|t| t.to_rfc3339()),
                task.metadata_json,
            ],
        )
        .context("Failed to insert task")?;

        Ok(())
    }

    /// Update an existing task
    pub fn update(conn: &Connection, task: &Task) -> Result<()> {
        let updated_at = Utc::now();

        conn.execute(
            "UPDATE tasks SET
                name = ?2, description = ?3, status = ?4, worktree_path = ?5, git_branch = ?6,
                last_accessed_at = ?7, cleanup_eligible = ?8, updated_at = ?9, completed_at = ?10, metadata_json = ?11
             WHERE id = ?1",
            params![
                task.id,
                task.name,
                task.description,
                task.status.as_str(),
                task.worktree_path.as_ref().and_then(|p| p.to_str()),
                task.git_branch,
                task.last_accessed_at.map(|t| t.to_rfc3339()),
                task.cleanup_eligible,
                updated_at.to_rfc3339(),
                task.completed_at.map(|t| t.to_rfc3339()),
                task.metadata_json,
            ],
        )
        .context("Failed to update task")?;

        Ok(())
    }

    /// Get a task by ID
    pub fn get_by_id(conn: &Connection, task_id: &str) -> Result<Option<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, status, worktree_path, git_branch,
                    last_accessed_at, cleanup_eligible, created_at, updated_at, completed_at, metadata_json
             FROM tasks WHERE id = ?1",
        )?;

        let task = stmt
            .query_row(params![task_id], parse_task_row)
            .optional()
            .context("Failed to query task")?;

        Ok(task)
    }

    /// Get a task by name within a project
    pub fn get_by_name(conn: &Connection, project_id: &str, name: &str) -> Result<Option<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, status, worktree_path, git_branch,
                    last_accessed_at, cleanup_eligible, created_at, updated_at, completed_at, metadata_json
             FROM tasks WHERE project_id = ?1 AND name = ?2",
        )?;

        let task = stmt
            .query_row(params![project_id, name], parse_task_row)
            .optional()
            .context("Failed to query task")?;

        Ok(task)
    }

    /// List all tasks for a project
    pub fn list_by_project(conn: &Connection, project_id: &str) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, status, worktree_path, git_branch,
                    last_accessed_at, cleanup_eligible, created_at, updated_at, completed_at, metadata_json
             FROM tasks WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;

        let tasks = stmt
            .query_map(params![project_id], parse_task_row)?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect tasks")?;

        Ok(tasks)
    }

    /// List tasks by status
    pub fn list_by_status(
        conn: &Connection,
        project_id: &str,
        status: TaskStatus,
    ) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, status, worktree_path, git_branch,
                    last_accessed_at, cleanup_eligible, created_at, updated_at, completed_at, metadata_json
             FROM tasks WHERE project_id = ?1 AND status = ?2 ORDER BY created_at DESC",
        )?;

        let tasks = stmt
            .query_map(params![project_id, status.as_str()], parse_task_row)?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect tasks")?;

        Ok(tasks)
    }

    /// Delete a task by ID
    pub fn delete(conn: &Connection, task_id: &str) -> Result<()> {
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])
            .context("Failed to delete task")?;

        Ok(())
    }

    /// Update task's last_accessed_at timestamp
    pub fn touch(conn: &Connection, task_id: &str) -> Result<()> {
        let now = Utc::now();
        conn.execute(
            "UPDATE tasks SET last_accessed_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![task_id, now.to_rfc3339()],
        )
        .context("Failed to update task access time")?;

        Ok(())
    }

    /// Check if a task name exists in a project
    pub fn name_exists(conn: &Connection, project_id: &str, name: &str) -> Result<bool> {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND name = ?2",
                params![project_id, name],
                |row| row.get(0),
            )
            .context("Failed to check task name existence")?;

        Ok(count > 0)
    }

    /// Helper function to parse a task row
    fn parse_task_row(row: &rusqlite::Row) -> rusqlite::Result<Task> {
        Ok(Task {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            status: TaskStatus::from_str(&row.get::<_, String>(4)?)
                .ok_or_else(|| rusqlite::Error::InvalidColumnType(4, "status".to_string(), rusqlite::types::Type::Text))?,
            worktree_path: row.get::<_, Option<String>>(5)?.map(PathBuf::from),
            git_branch: row.get(6)?,
            last_accessed_at: row
                .get::<_, Option<String>>(7)?
                .and_then(|s| s.parse().ok()),
            cleanup_eligible: row.get(8)?,
            created_at: row
                .get::<_, String>(9)?
                .parse()
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?,
            updated_at: row
                .get::<_, String>(10)?
                .parse()
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?,
            completed_at: row
                .get::<_, Option<String>>(11)?
                .and_then(|s| s.parse().ok()),
            metadata_json: row.get(12)?,
        })
    }
}

/// Worktree database operations
pub mod worktrees {
    use super::*;

    /// Insert a new worktree
    pub fn insert(conn: &Connection, worktree: &Worktree) -> Result<()> {
        conn.execute(
            "INSERT INTO worktrees (id, task_id, path, branch_name, created_at, last_accessed_at, is_orphan)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                worktree.id,
                worktree.task_id,
                worktree.path.to_str(),
                worktree.branch_name,
                worktree.created_at.to_rfc3339(),
                worktree.last_accessed_at.to_rfc3339(),
                worktree.is_orphan,
            ],
        )
        .context("Failed to insert worktree")?;

        Ok(())
    }

    /// Get a worktree by task ID
    pub fn get_by_task_id(conn: &Connection, task_id: &str) -> Result<Option<Worktree>> {
        let mut stmt = conn.prepare(
            "SELECT id, task_id, path, branch_name, created_at, last_accessed_at, is_orphan
             FROM worktrees WHERE task_id = ?1",
        )?;

        let worktree = stmt
            .query_row(params![task_id], parse_worktree_row)
            .optional()
            .context("Failed to query worktree")?;

        Ok(worktree)
    }

    /// Get a worktree by path
    pub fn get_by_path(conn: &Connection, path: &str) -> Result<Option<Worktree>> {
        let mut stmt = conn.prepare(
            "SELECT id, task_id, path, branch_name, created_at, last_accessed_at, is_orphan
             FROM worktrees WHERE path = ?1",
        )?;

        let worktree = stmt
            .query_row(params![path], parse_worktree_row)
            .optional()
            .context("Failed to query worktree")?;

        Ok(worktree)
    }

    /// List all worktrees
    pub fn list_all(conn: &Connection) -> Result<Vec<Worktree>> {
        let mut stmt = conn.prepare(
            "SELECT id, task_id, path, branch_name, created_at, last_accessed_at, is_orphan
             FROM worktrees ORDER BY created_at DESC",
        )?;

        let worktrees = stmt
            .query_map([], parse_worktree_row)?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect worktrees")?;

        Ok(worktrees)
    }

    /// Delete a worktree by task ID
    pub fn delete_by_task_id(conn: &Connection, task_id: &str) -> Result<()> {
        conn.execute("DELETE FROM worktrees WHERE task_id = ?1", params![task_id])
            .context("Failed to delete worktree")?;

        Ok(())
    }

    /// Update worktree's last_accessed_at timestamp
    pub fn touch(conn: &Connection, task_id: &str) -> Result<()> {
        let now = Utc::now();
        conn.execute(
            "UPDATE worktrees SET last_accessed_at = ?2 WHERE task_id = ?1",
            params![task_id, now.to_rfc3339()],
        )
        .context("Failed to update worktree access time")?;

        Ok(())
    }

    /// Helper function to parse a worktree row
    fn parse_worktree_row(row: &rusqlite::Row) -> rusqlite::Result<Worktree> {
        Ok(Worktree {
            id: row.get(0)?,
            task_id: row.get(1)?,
            path: PathBuf::from(row.get::<_, String>(2)?),
            branch_name: row.get(3)?,
            created_at: row
                .get::<_, String>(4)?
                .parse()
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?,
            last_accessed_at: row
                .get::<_, String>(5)?
                .parse()
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?,
            is_orphan: row.get(6)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_project_crud() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();

        let project = Project::new(
            "test-project".to_string(),
            "https://github.com/test/repo.git".to_string(),
            PathBuf::from("/tmp/repo"),
            PathBuf::from("/tmp/repo-wt"),
        );

        // Insert
        projects::insert(conn, &project).unwrap();

        // Get by ID
        let retrieved = projects::get_by_id(conn, &project.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "test-project");

        // Get by name
        let retrieved = projects::get_by_name(conn, "test-project").unwrap().unwrap();
        assert_eq!(retrieved.id, project.id);

        // List all
        let all = projects::list_all(conn).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_task_crud() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();

        let project = Project::new(
            "test-project".to_string(),
            "https://github.com/test/repo.git".to_string(),
            PathBuf::from("/tmp/repo"),
            PathBuf::from("/tmp/repo-wt"),
        );
        projects::insert(conn, &project).unwrap();

        let task = Task::new(
            project.id.clone(),
            "test-task".to_string(),
            Some("Test description".to_string()),
        );

        // Insert
        tasks::insert(conn, &task).unwrap();

        // Get by ID
        let retrieved = tasks::get_by_id(conn, &task.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "test-task");
        assert_eq!(retrieved.status, TaskStatus::Pending);

        // Get by name
        let retrieved = tasks::get_by_name(conn, &project.id, "test-task")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, task.id);

        // Name exists
        assert!(tasks::name_exists(conn, &project.id, "test-task").unwrap());
        assert!(!tasks::name_exists(conn, &project.id, "nonexistent").unwrap());

        // Update
        let mut updated_task = task.clone();
        updated_task.status = TaskStatus::Active;
        tasks::update(conn, &updated_task).unwrap();

        let retrieved = tasks::get_by_id(conn, &task.id).unwrap().unwrap();
        assert_eq!(retrieved.status, TaskStatus::Active);

        // List by project
        let all = tasks::list_by_project(conn, &project.id).unwrap();
        assert_eq!(all.len(), 1);

        // List by status
        let active = tasks::list_by_status(conn, &project.id, TaskStatus::Active).unwrap();
        assert_eq!(active.len(), 1);

        let pending = tasks::list_by_status(conn, &project.id, TaskStatus::Pending).unwrap();
        assert_eq!(pending.len(), 0);

        // Touch
        tasks::touch(conn, &task.id).unwrap();
        let retrieved = tasks::get_by_id(conn, &task.id).unwrap().unwrap();
        assert!(retrieved.last_accessed_at.is_some());

        // Delete
        tasks::delete(conn, &task.id).unwrap();
        let retrieved = tasks::get_by_id(conn, &task.id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_worktree_crud() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();

        let project = Project::new(
            "test-project".to_string(),
            "https://github.com/test/repo.git".to_string(),
            PathBuf::from("/tmp/repo"),
            PathBuf::from("/tmp/repo-wt"),
        );
        projects::insert(conn, &project).unwrap();

        let task = Task::new(project.id.clone(), "test-task".to_string(), None);
        tasks::insert(conn, &task).unwrap();

        let worktree = Worktree::new(
            task.id.clone(),
            PathBuf::from("/tmp/repo-wt/test-task"),
            "kayfabe/test-task".to_string(),
        );

        // Insert
        worktrees::insert(conn, &worktree).unwrap();

        // Get by task ID
        let retrieved = worktrees::get_by_task_id(conn, &task.id).unwrap().unwrap();
        assert_eq!(retrieved.branch_name, "kayfabe/test-task");

        // Get by path
        let retrieved = worktrees::get_by_path(conn, "/tmp/repo-wt/test-task")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, worktree.id);

        // List all
        let all = worktrees::list_all(conn).unwrap();
        assert_eq!(all.len(), 1);

        // Touch
        worktrees::touch(conn, &task.id).unwrap();

        // Delete
        worktrees::delete_by_task_id(conn, &task.id).unwrap();
        let retrieved = worktrees::get_by_task_id(conn, &task.id).unwrap();
        assert!(retrieved.is_none());
    }
}
