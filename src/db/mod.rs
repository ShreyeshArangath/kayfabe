pub mod migrations;
pub mod models;
pub mod operations;

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
