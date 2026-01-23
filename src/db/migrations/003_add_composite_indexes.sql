-- Add composite indexes for performance optimization
-- Based on common query patterns identified in vibe-kanban research

-- Task indexes
CREATE INDEX idx_tasks_project_status ON tasks(project_id, status);
CREATE INDEX idx_tasks_last_accessed ON tasks(last_accessed_at);
CREATE INDEX idx_tasks_cleanup_eligible ON tasks(cleanup_eligible, last_accessed_at);

-- Worktree indexes
CREATE INDEX idx_worktrees_task ON worktrees(task_id);
CREATE INDEX idx_worktrees_orphan ON worktrees(is_orphan);

-- Activity indexes
CREATE INDEX idx_activities_task_created ON activities(task_id, created_at);
CREATE INDEX idx_activities_type ON activities(activity_type, created_at);

-- Execution process indexes
CREATE INDEX idx_execution_processes_task ON execution_processes(task_id);
CREATE INDEX idx_execution_processes_status ON execution_processes(status, started_at);
