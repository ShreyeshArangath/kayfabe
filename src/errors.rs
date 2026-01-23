use thiserror::Error;

/// Custom error types for Kayfabe
#[derive(Error, Debug)]
pub enum KayfabeError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Task already exists: {0}")]
    TaskAlreadyExists(String),

    #[error("Worktree error: {0}")]
    WorktreeError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Process execution error: {0}")]
    ProcessError(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;

/// Trait for adding helpful context to errors
pub trait KayfabeContext<T> {
    fn kayfabe_context(self, msg: &str) -> Result<T>;
}

impl<T, E> KayfabeContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn kayfabe_context(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
            .map_err(|e| e.context(get_helpful_suggestion(msg)))
    }
}

/// Provides helpful suggestions based on error context
fn get_helpful_suggestion(error_msg: &str) -> String {
    match error_msg {
        msg if msg.contains("repository") => {
            "Suggestion: Run 'kayfabe init' to initialize the project first".to_string()
        }
        msg if msg.contains("tmux") => {
            "Suggestion: Install tmux with: brew install tmux (macOS) or apt install tmux (Linux)"
                .to_string()
        }
        msg if msg.contains("task") && msg.contains("not found") => {
            "Suggestion: Run 'kayfabe list' to see available tasks".to_string()
        }
        msg if msg.contains("database") => {
            "Suggestion: Try removing ~/.kayfabe/kayfabe.db and reinitializing".to_string()
        }
        _ => String::new(),
    }
}
