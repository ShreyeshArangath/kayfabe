use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Project represents a Kayfabe-managed repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_path: PathBuf,
    pub worktree_dir: PathBuf,
    pub git_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config_json: Option<String>,
}

impl Project {
    pub fn new(name: String, git_url: String, repo_path: PathBuf, worktree_dir: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            repo_path,
            worktree_dir,
            git_url,
            created_at: now,
            updated_at: now,
            config_json: None,
        }
    }
}

/// Task status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Active,
    Completed,
    Archived,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Active => "active",
            TaskStatus::Completed => "completed",
            TaskStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "active" => Some(TaskStatus::Active),
            "completed" => Some(TaskStatus::Completed),
            "archived" => Some(TaskStatus::Archived),
            _ => None,
        }
    }
}

/// Task represents a unit of work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub worktree_path: Option<PathBuf>,
    pub git_branch: Option<String>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub cleanup_eligible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata_json: Option<String>,
}

impl Task {
    pub fn new(project_id: String, name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            name,
            description,
            status: TaskStatus::Pending,
            worktree_path: None,
            git_branch: None,
            last_accessed_at: None,
            cleanup_eligible: false,
            created_at: now,
            updated_at: now,
            completed_at: None,
            metadata_json: None,
        }
    }
}

/// Worktree represents a git worktree for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    pub id: String,
    pub task_id: String,
    pub path: PathBuf,
    pub branch_name: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub is_orphan: bool,
}

impl Worktree {
    pub fn new(task_id: String, path: PathBuf, branch_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id,
            path,
            branch_name,
            created_at: now,
            last_accessed_at: now,
            is_orphan: false,
        }
    }
}

/// Activity type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityType {
    Thought,
    Artifact,
    Error,
    Summary,
}

impl ActivityType {
    pub fn as_str(&self) -> &str {
        match self {
            ActivityType::Thought => "thought",
            ActivityType::Artifact => "artifact",
            ActivityType::Error => "error",
            ActivityType::Summary => "summary",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "thought" => Some(ActivityType::Thought),
            "artifact" => Some(ActivityType::Artifact),
            "error" => Some(ActivityType::Error),
            "summary" => Some(ActivityType::Summary),
            _ => None,
        }
    }
}

/// Activity tracks thoughts, artifacts, and errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub task_id: String,
    pub activity_type: ActivityType,
    pub content: Option<String>,
    pub file_path: Option<PathBuf>,
    pub created_at: DateTime<Utc>,
    pub metadata_json: Option<String>,
}

impl Activity {
    pub fn new(
        task_id: String,
        activity_type: ActivityType,
        content: Option<String>,
        file_path: Option<PathBuf>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id,
            activity_type,
            content,
            file_path,
            created_at: Utc::now(),
            metadata_json: None,
        }
    }
}

/// Process status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
    Running,
    Completed,
    Failed,
    Killed,
}

impl ProcessStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ProcessStatus::Running => "running",
            ProcessStatus::Completed => "completed",
            ProcessStatus::Failed => "failed",
            ProcessStatus::Killed => "killed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "running" => Some(ProcessStatus::Running),
            "completed" => Some(ProcessStatus::Completed),
            "failed" => Some(ProcessStatus::Failed),
            "killed" => Some(ProcessStatus::Killed),
            _ => None,
        }
    }
}

/// ExecutionProcess tracks process execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProcess {
    pub id: String,
    pub task_id: String,
    pub command: String,
    pub status: ProcessStatus,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tmux_session: Option<String>,
}

impl ExecutionProcess {
    pub fn new(task_id: String, command: String, tmux_session: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id,
            command,
            status: ProcessStatus::Running,
            exit_code: None,
            stdout: None,
            stderr: None,
            started_at: Utc::now(),
            completed_at: None,
            tmux_session,
        }
    }
}
