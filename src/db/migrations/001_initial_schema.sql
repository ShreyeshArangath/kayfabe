-- Initial schema for Kayfabe
-- Creates core tables for projects, tasks, worktrees, and activities

-- Projects table
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    repo_path TEXT NOT NULL,
    worktree_dir TEXT NOT NULL,
    git_url TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    config_json TEXT
);

-- Tasks table with enhanced tracking
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'active', 'completed', 'archived')),
    worktree_path TEXT,
    git_branch TEXT,
    last_accessed_at TEXT,
    cleanup_eligible INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    metadata_json TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(project_id, name)
);

-- Worktrees table
CREATE TABLE worktrees (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    branch_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_accessed_at TEXT NOT NULL,
    is_orphan INTEGER DEFAULT 0,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Activities table for thought/artifact tracking
CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    activity_type TEXT NOT NULL CHECK(activity_type IN ('thought', 'artifact', 'error', 'summary')),
    content TEXT,
    file_path TEXT,
    created_at TEXT NOT NULL,
    metadata_json TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
