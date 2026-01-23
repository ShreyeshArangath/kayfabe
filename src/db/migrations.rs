use anyhow::{Context, Result};
use rusqlite::Connection;

/// Migration definition
pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

/// All migrations in order
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema - projects, tasks, worktrees, activities",
        sql: include_str!("migrations/001_initial_schema.sql"),
    },
    Migration {
        version: 2,
        description: "Add execution_processes table for process monitoring",
        sql: include_str!("migrations/002_add_execution_processes.sql"),
    },
    Migration {
        version: 3,
        description: "Add composite indexes for performance",
        sql: include_str!("migrations/003_add_composite_indexes.sql"),
    },
];

/// Initialize the schema_migrations table
fn init_schema_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL,
            description TEXT
        )",
        [],
    )
    .context("Failed to create schema_migrations table")?;

    Ok(())
}

/// Check if a migration has been applied
fn is_migration_applied(conn: &Connection, version: i32) -> Result<bool> {
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM schema_migrations WHERE version = ?1")
        .context("Failed to prepare migration check query")?;

    let count: i32 = stmt
        .query_row([version], |row| row.get(0))
        .context("Failed to check migration status")?;

    Ok(count > 0)
}

/// Apply a single migration
fn apply_migration(conn: &Connection, migration: &Migration) -> Result<()> {
    // Start transaction
    let tx = conn
        .unchecked_transaction()
        .context("Failed to start transaction")?;

    // Execute migration SQL
    tx.execute_batch(migration.sql)
        .with_context(|| format!("Failed to execute migration {}", migration.version))?;

    // Record migration
    tx.execute(
        "INSERT INTO schema_migrations (version, applied_at, description) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            migration.version,
            chrono::Utc::now().to_rfc3339(),
            migration.description,
        ],
    )
    .with_context(|| format!("Failed to record migration {}", migration.version))?;

    // Commit transaction
    tx.commit().context("Failed to commit migration")?;

    println!(
        "✓ Applied migration {}: {}",
        migration.version, migration.description
    );

    Ok(())
}

/// Run all pending migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Initialize schema_migrations table
    init_schema_migrations(conn)?;

    // Apply pending migrations
    let mut applied_count = 0;
    for migration in MIGRATIONS {
        if !is_migration_applied(conn, migration.version)? {
            apply_migration(conn, migration)?;
            applied_count += 1;
        }
    }

    if applied_count == 0 {
        println!("✓ Database schema is up to date");
    } else {
        println!("✓ Applied {} migration(s)", applied_count);
    }

    Ok(())
}

/// Get the current schema version
pub fn get_schema_version(conn: &Connection) -> Result<Option<i32>> {
    // Check if schema_migrations table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| {
                let count: i32 = row.get(0)?;
                Ok(count > 0)
            },
        )
        .context("Failed to check for schema_migrations table")?;

    if !table_exists {
        return Ok(None);
    }

    // Get the latest version
    let version: Option<i32> = conn
        .query_row(
            "SELECT MAX(version) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_run_successfully() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify schema version
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, Some(3));

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"worktrees".to_string()));
        assert!(tables.contains(&"activities".to_string()));
        assert!(tables.contains(&"execution_processes".to_string()));
        assert!(tables.contains(&"schema_migrations".to_string()));
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        // Should still be at version 3
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, Some(3));
    }
}
