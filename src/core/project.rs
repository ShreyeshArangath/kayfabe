use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::db::{models::Project as ProjectModel, Database};
use crate::utils::{git, paths};

/// Project initialization and management
pub struct Project;

impl Project {
    /// Initialize a new Kayfabe project
    pub async fn init(name: &str, git_url: &str) -> Result<ProjectModel> {
        // 1. Validate git URL format
        git::validate_git_url(git_url)
            .context("Invalid git URL")?;

        // 2. Check if project directory already exists
        let project_path = PathBuf::from(name);
        if project_path.exists() {
            bail!(
                "Directory '{}' already exists. Please choose a different name or remove the existing directory.",
                name
            );
        }

        // 3. Check if project name already exists in database
        let db = Database::open()?;
        if db.project_exists(name)? {
            bail!(
                "Project '{}' already exists in the database. Use a different name.",
                name
            );
        }

        // 4. Create project directory
        fs::create_dir_all(&project_path)
            .with_context(|| format!("Failed to create project directory: {}", project_path.display()))?;

        // 5. Clone bare repository
        println!("📦 Cloning repository...");
        let repo_path = project_path.join(".git");
        let repo = git::clone_bare(git_url, &repo_path)
            .context("Failed to clone repository")?;

        // 6. Validate repository has commits
        git::validate_repo_has_commits(&repo)
            .context("Repository is empty or has no commits")?;

        // 7. Detect default branch
        let default_branch = git::detect_default_branch(&repo)
            .context("Failed to detect default branch")?;
        println!("✓ Detected default branch: {}", default_branch);

        // 8. Create directory structure
        let worktree_dir = project_path.join(format!("{}-wt", name));
        fs::create_dir_all(&worktree_dir)
            .with_context(|| format!("Failed to create worktree directory: {}", worktree_dir.display()))?;

        let thoughts_dir = project_path.join("thoughts");
        fs::create_dir_all(&thoughts_dir)
            .with_context(|| format!("Failed to create thoughts directory: {}", thoughts_dir.display()))?;

        // Create .gitkeep in thoughts directory
        let gitkeep_path = thoughts_dir.join(".gitkeep");
        fs::write(&gitkeep_path, "")
            .context("Failed to create .gitkeep file")?;

        // Create .kayfabe directory for local state
        let kayfabe_local_dir = project_path.join(".kayfabe");
        fs::create_dir_all(&kayfabe_local_dir)
            .context("Failed to create .kayfabe directory")?;

        // 9. Copy template files
        println!("📝 Creating configuration files...");
        Self::create_config_files(&project_path)?;

        // 10. Create project model
        let project = ProjectModel::new(
            name.to_string(),
            git_url.to_string(),
            repo_path,
            worktree_dir,
        );

        // 11. Register in database
        db.insert_project(&project)
            .context("Failed to register project in database")?;

        println!("✓ Project '{}' initialized successfully!", name);
        println!();
        println!("Directory structure created:");
        println!("  {}/", name);
        println!("  ├── .git/           # Bare repository");
        println!("  ├── {}-wt/          # Worktree base (empty initially)", name);
        println!("  ├── thoughts/       # AI artifacts");
        println!("  ├── .kayfabe/       # Local state");
        println!("  ├── kayfabe.md      # Project documentation");
        println!("  └── claude.md       # Claude instructions");
        println!();
        println!("Next steps:");
        println!("  cd {}", name);
        println!("  kayfabe add <task-name>  # Create your first task");

        Ok(project)
    }

    /// Create configuration files from templates
    fn create_config_files(project_path: &PathBuf) -> Result<()> {
        let kayfabe_md_path = project_path.join("kayfabe.md");
        let claude_md_path = project_path.join("claude.md");

        // Try to get templates from global directory, fall back to embedded defaults
        let templates_dir = paths::get_templates_dir()?;
        let template_kayfabe_md = templates_dir.join("kayfabe.md");
        let template_claude_md = templates_dir.join("claude.md");

        // Copy or create kayfabe.md
        if template_kayfabe_md.exists() {
            fs::copy(&template_kayfabe_md, &kayfabe_md_path)
                .context("Failed to copy kayfabe.md template")?;
        } else {
            fs::write(&kayfabe_md_path, Self::default_kayfabe_md())
                .context("Failed to create kayfabe.md")?;
        }

        // Copy or create claude.md
        if template_claude_md.exists() {
            fs::copy(&template_claude_md, &claude_md_path)
                .context("Failed to copy claude.md template")?;
        } else {
            fs::write(&claude_md_path, Self::default_claude_md())
                .context("Failed to create claude.md")?;
        }

        Ok(())
    }

    /// Default kayfabe.md content
    fn default_kayfabe_md() -> &'static str {
        r#"# Kayfabe Project

This project is managed by [Kayfabe](https://github.com/shreyesharangath/kayfabe), an AI-enhanced CLI task manager with git worktree orchestration.

## What is Kayfabe?

Kayfabe accelerates engineer productivity by combining task management with isolated git worktrees. Each task gets its own worktree and git branch, enabling parallel development with AI assistants like Claude.

## Available Commands

### Task Management
```bash
kayfabe add <task-name>              # Create a new task with isolated worktree
kayfabe list                          # List all tasks
kayfabe list --tui                    # Launch interactive TUI
kayfabe remove <task-name>            # Remove a task and its worktree
kayfabe status                        # Show project status
```

### Task Execution
```bash
kayfabe execute <task-name>           # Execute task in tmux with Claude
kayfabe attach <task-name>            # Attach to running task session
kayfabe kill <task-name>              # Kill running task
kayfabe logs <task-name>              # View task execution logs
```

### Worktree Management
```bash
kayfabe worktree list                 # List all worktrees
kayfabe worktree list --stale 14      # List worktrees older than 14 days
kayfabe worktree remove <name>        # Remove a specific worktree
kayfabe worktree cleanup              # Clean up stale worktrees
```

### Cleanup
```bash
kayfabe clean --stale 7d              # Clean worktrees older than 7 days
kayfabe clean --orphans               # Clean orphaned worktrees
kayfabe clean --all --dry-run         # Preview cleanup without deleting
```

## Workflow

1. **Create a task**: `kayfabe add feature-x --description "Add new feature"`
2. **Work on it**: The task automatically gets an isolated worktree and git branch
3. **Execute with AI**: `kayfabe execute feature-x` launches Claude in the task worktree
4. **Track progress**: AI thoughts and artifacts are saved in `thoughts/feature-x/`
5. **Clean up**: `kayfabe remove feature-x` removes the task and worktree when done

## Directory Structure

```
project/
├── .git/                    # Bare repository
├── project-wt/              # Worktree base directory
│   ├── feature-x/           # Task worktree (isolated)
│   ├── feature-y/           # Another task worktree
│   └── ...
├── thoughts/                # AI artifacts and thoughts
│   ├── feature-x/
│   │   ├── 2026-01-23_140530_thought.md
│   │   ├── 2026-01-23_141205_artifact.md
│   │   └── summary.md
│   └── ...
├── .kayfabe/                # Local Kayfabe state
├── kayfabe.md               # This file
└── claude.md                # Claude AI instructions
```

## Tips

- Each task has its own isolated worktree - no conflicts!
- Use `kayfabe execute` to run AI assistants in task context
- The TUI (`kayfabe list --tui`) provides a visual overview
- Stale worktrees can be cleaned up automatically
- All AI interactions are tracked in `thoughts/` for review

## Learn More

- GitHub: https://github.com/shreyesharangath/kayfabe
- Documentation: See implementation-plan.md for architecture details
"#
    }

    /// Default claude.md content
    fn default_claude_md() -> &'static str {
        r#"# Claude AI Instructions

This file provides context and instructions for Claude Code when working on tasks in this Kayfabe project.

## Project Context

This is a Kayfabe-managed project. Key things to know:

1. **Isolated Worktrees**: Each task runs in its own git worktree with a dedicated branch
2. **Parallel Development**: Multiple tasks can be worked on simultaneously without conflicts
3. **Thought Tracking**: Your reasoning and artifacts are saved in `thoughts/<task-name>/`

## Working with Kayfabe Tasks

When you're executing a task via `kayfabe execute <task-name>`:

- You're in an isolated worktree at `../<project>-wt/<task-name>/`
- Your branch is `kayfabe/<task-name>`
- Save important thoughts/artifacts to `../../thoughts/<task-name>/`
- The worktree is fully isolated - feel free to experiment

## Thought and Artifact Format

Save your reasoning and outputs as markdown files in the thoughts directory:

```markdown
---
type: thought
task: feature-x
timestamp: 2026-01-23T14:05:30Z
tags: [planning, architecture]
---

# Planning Feature X

Your reasoning and planning content here...
```

Types: `thought`, `artifact`, `error`, `summary`

## Development Workflow

1. Understand the task context from the task description
2. Plan your approach and save thoughts
3. Implement changes in the worktree
4. Document key decisions and learnings
5. Commit your work with clear messages

## Tips for AI Assistants

- **Be thorough**: Document your reasoning for complex decisions
- **Be organized**: Use clear file names and structure in thoughts/
- **Be cautious**: This is an isolated worktree, but still respect the codebase
- **Be helpful**: Leave good commit messages and documentation
- **Be efficient**: You have full context of the project via kayfabe.md

## Project-Specific Guidelines

[Add any project-specific coding standards, conventions, or requirements here]

## Additional Context

[Add any additional context that would help AI assistants work effectively on this project]
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_project_init_with_invalid_url() {
        let result = Project::init("test-project", "not-a-valid-url").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_project_init_creates_default_templates() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-templates");

        // Create a minimal project structure to test template creation
        fs::create_dir_all(&project_path).unwrap();

        Project::create_config_files(&project_path).unwrap();

        assert!(project_path.join("kayfabe.md").exists());
        assert!(project_path.join("claude.md").exists());
    }
}
