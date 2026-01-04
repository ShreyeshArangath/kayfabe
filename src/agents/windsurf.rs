use crate::agents::{AgentGenerator, ProjectContext};
use crate::error::Result;

pub struct WindsurfGenerator;

impl AgentGenerator for WindsurfGenerator {
    fn generate(&self, context: &ProjectContext) -> Result<String> {
        let mut content = format!("# {} - Windsurf Rules\n\n", context.name);

        content.push_str("## Project Overview\n");
        content.push_str(&format!("Project Type: {}\n\n", context.project_type));

        if context.is_workspace {
            content.push_str("This is a workspace/monorepo.\n\n");
        }

        content.push_str("## Development Commands\n");
        if let Some(build_cmd) = &context.build_cmd {
            content.push_str(&format!("- Build: `{}`\n", build_cmd));
        }
        if let Some(test_cmd) = &context.test_cmd {
            content.push_str(&format!("- Test: `{}`\n", test_cmd));
        }
        if let Some(lint_cmd) = &context.lint_cmd {
            content.push_str(&format!("- Lint: `{}`\n", lint_cmd));
        }
        content.push('\n');

        content.push_str("## Code Guidelines\n");
        content.push_str("- Maintain consistency with existing code style\n");
        content.push_str("- Write comprehensive tests for new features\n");
        content.push_str("- Document public APIs and complex logic\n");
        content.push_str("- Keep functions focused and modular\n");
        content.push('\n');

        content.push_str("## Best Practices\n");
        match context.project_type.as_str() {
            "rust" => {
                content.push_str("- Use Result types for error handling\n");
                content.push_str("- Leverage the type system for safety\n");
                content.push_str("- Prefer composition over inheritance\n");
            }
            "typescript" | "javascript" => {
                content.push_str("- Use async/await for asynchronous code\n");
                content.push_str("- Prefer immutable data structures\n");
                content.push_str("- Use TypeScript for type safety\n");
            }
            "python" => {
                content.push_str("- Use type hints for better IDE support\n");
                content.push_str("- Follow PEP 8 style guidelines\n");
                content.push_str("- Use virtual environments\n");
            }
            "go" => {
                content.push_str("- Handle errors explicitly\n");
                content.push_str("- Use interfaces for abstraction\n");
                content.push_str("- Keep packages focused\n");
            }
            _ => {
                content.push_str("- Follow project-specific guidelines\n");
            }
        }

        Ok(content)
    }

    fn file_path(&self) -> &str {
        ".windsurf/rules/rules.md"
    }
}
