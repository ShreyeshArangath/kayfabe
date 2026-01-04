use crate::agents::{AgentGenerator, ProjectContext};
use crate::error::Result;

pub struct ClaudeGenerator;

impl AgentGenerator for ClaudeGenerator {
    fn generate(&self, context: &ProjectContext) -> Result<String> {
        let mut content = format!("# Project: {}\n\n", context.name);

        content.push_str("## Commands\n");
        if let Some(build_cmd) = &context.build_cmd {
            content.push_str(&format!("- `{}`: Build the project\n", build_cmd));
        }
        if let Some(test_cmd) = &context.test_cmd {
            content.push_str(&format!("- `{}`: Run tests\n", test_cmd));
        }
        if let Some(lint_cmd) = &context.lint_cmd {
            content.push_str(&format!("- `{}`: Run linter\n", lint_cmd));
        }
        content.push('\n');

        content.push_str("## Code Style\n");
        match context.project_type.as_str() {
            "rust" => {
                content.push_str("- Follow Rust 2021 edition idioms\n");
                content
                    .push_str("- Use `thiserror` for library errors, `anyhow` for applications\n");
                content.push_str("- Prefer `impl Trait` over generics where appropriate\n");
                content.push_str("- Run `cargo fmt` before committing\n");
            }
            "typescript" | "javascript" => {
                content.push_str("- Use TypeScript strict mode\n");
                content.push_str("- Prefer functional patterns\n");
                content.push_str("- Use explicit return types for exported functions\n");
            }
            "python" => {
                content.push_str("- Type hints required for public functions\n");
                content.push_str("- Docstrings for public modules/classes/functions\n");
                content.push_str("- Follow PEP 8 style guide\n");
            }
            "go" => {
                content.push_str("- Follow effective Go guidelines\n");
                content.push_str("- Use gofmt for formatting\n");
                content.push_str("- Write idiomatic Go code\n");
            }
            _ => {
                content.push_str("- Follow project conventions\n");
            }
        }
        content.push('\n');

        content.push_str("## Architecture\n");
        if context.is_workspace {
            content.push_str("This is a workspace/monorepo with the following members:\n");
            for member in &context.workspace_members {
                content.push_str(&format!("- `{}`\n", member));
            }
        } else {
            content.push_str("Single project structure.\n");
        }
        content.push('\n');

        content.push_str("## Workflow\n");
        content.push_str("- Create feature branches for new work\n");
        content.push_str("- Write tests for new functionality\n");
        content.push_str("- Ensure all tests pass before committing\n");
        content.push_str("- Keep commits atomic and well-described\n");

        Ok(content)
    }

    fn file_path(&self) -> &str {
        "CLAUDE.md"
    }
}
