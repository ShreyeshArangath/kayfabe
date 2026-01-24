pub mod migrations;
pub mod models;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

use crate::utils;

/// Database handle with connection management
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the global database
    pub fn open() -> Result<Self> {
        // Ensure ~/.kayfabe directory exists
        utils::init_kayfabe_dir()?;

        let db_path = utils::get_database_path()?;
        Self::open_at(&db_path)
    }

    /// Open database at a specific path
    pub fn open_at(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {}", path.display()))?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        let db = Self { conn };

        // Run migrations
        migrations::run_migrations(&db.conn)?;

        Ok(db)
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to open in-memory database")?;

        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        let db = Self { conn };

        // Run migrations
        migrations::run_migrations(&db.conn)?;

        Ok(db)
    }

    /// Get a reference to the underlying connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get the current schema version
    pub fn schema_version(&self) -> Result<Option<i32>> {
        migrations::get_schema_version(&self.conn)
    }

    // ========== Project Operations ==========

    /// Insert a new project
    pub fn insert_project(&self, project: &models::Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                &project.id,
                &project.name,
                project.repo_path.to_str(),
                project.worktree_dir.to_str(),
                &project.git_url,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
                &project.config_json,
            ],
        )
        .context("Failed to insert project")?;

        Ok(())
    }

    /// Get a project by ID
    pub fn get_project_by_id(&self, id: &str) -> Result<models::Project> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json
             FROM projects WHERE id = ?1"
        )?;

        let project = stmt.query_row([id], |row| {
            Ok(models::Project {
                id: row.get(0)?,
                name: row.get(1)?,
                repo_path: row.get::<_, String>(2)?.into(),
                worktree_dir: row.get::<_, String>(3)?.into(),
                git_url: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse().unwrap(),
                updated_at: row.get::<_, String>(6)?.parse().unwrap(),
                config_json: row.get(7)?,
            })
        })
        .context("Project not found")?;

        Ok(project)
    }

    /// Get a project by name
    pub fn get_project_by_name(&self, name: &str) -> Result<models::Project> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json
             FROM projects WHERE name = ?1"
        )?;

        let project = stmt.query_row([name], |row| {
            Ok(models::Project {
                id: row.get(0)?,
                name: row.get(1)?,
                repo_path: row.get::<_, String>(2)?.into(),
                worktree_dir: row.get::<_, String>(3)?.into(),
                git_url: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse().unwrap(),
                updated_at: row.get::<_, String>(6)?.parse().unwrap(),
                config_json: row.get(7)?,
            })
        })
        .context("Project not found")?;

        Ok(project)
    }

    /// Check if a project exists by name
    pub fn project_exists(&self, name: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM projects WHERE name = ?1")?;
        let count: i32 = stmt.query_row([name], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// List all projects
    pub fn list_projects(&self) -> Result<Vec<models::Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, repo_path, worktree_dir, git_url, created_at, updated_at, config_json
             FROM projects ORDER BY created_at DESC"
        )?;

        let projects = stmt.query_map([], |row| {
            Ok(models::Project {
                id: row.get(0)?,
                name: row.get(1)?,
                repo_path: row.get::<_, String>(2)?.into(),
                worktree_dir: row.get::<_, String>(3)?.into(),
                git_url: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse().unwrap(),
                updated_at: row.get::<_, String>(6)?.parse().unwrap(),
                config_json: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(projects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_opens_and_migrates() {
        let db = Database::open_in_memory().unwrap();
        let version = db.schema_version().unwrap();
        assert_eq!(version, Some(3));
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let db = Database::open_in_memory().unwrap();
        let foreign_keys: i32 = db
            .conn()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);
    }
}
