use thiserror::Error;

#[derive(Error, Debug)]
pub enum KayfabeError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dialog error: {0}")]
    Dialog(#[from] dialoguer::Error),

    #[error("Not a git repository: {0}")]
    NotARepository(String),

    #[error("Worktree already exists: {0}")]
    WorktreeExists(String),

    #[error("Worktree not found: {0}")]
    WorktreeNotFound(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("IDE not found: {0}")]
    IdeNotFound(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, KayfabeError>;
