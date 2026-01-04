use crate::agents::{AgentGenerator, ProjectContext};
use crate::error::Result;

pub struct CursorGenerator;

impl AgentGenerator for CursorGenerator {
    fn generate(&self, context: &ProjectContext) -> Result<String> {
        let mut content = format!("# {} Rules\n\n", context.name);

        content.push_str("## Project Context\n");
        content.push_str(&format!("This is a {} project.\n\n", context.project_type));

        content.push_str("## Tech Stack\n");
        match context.project_type.as_str() {
            "rust" => {
                content.push_str("- Language: Rust\n");
                content.push_str("- Build System: Cargo\n");
                content.push_str("- Testing: Built-in test framework\n");
            }
            "typescript" => {
                content.push_str("- Language: TypeScript\n");
                content.push_str("- Runtime: Node.js\n");
                content.push_str("- Build: tsc/bundler\n");
            }
            "javascript" => {
                content.push_str("- Language: JavaScript\n");
                content.push_str("- Runtime: Node.js\n");
            }
            "python" => {
                content.push_str("- Language: Python\n");
                content.push_str("- Testing: pytest\n");
                content.push_str("- Linting: ruff\n");
            }
            "go" => {
                content.push_str("- Language: Go\n");
                content.push_str("- Build: go build\n");
                content.push_str("- Testing: go test\n");
            }
            _ => {
                content.push_str("- See project documentation\n");
            }
        }
        content.push('\n');

        content.push_str("## Conventions\n");
        if let Some(test_cmd) = &context.test_cmd {
            content.push_str(&format!("- Run tests with: `{}`\n", test_cmd));
        }
        if let Some(lint_cmd) = &context.lint_cmd {
            content.push_str(&format!("- Run linter with: `{}`\n", lint_cmd));
        }
        content.push_str("- Follow existing code patterns\n");
        content.push_str("- Write clear, self-documenting code\n");
        content.push('\n');

        content.push_str("## Testing\n");
        content.push_str("- Write tests for new features\n");
        content.push_str("- Maintain or improve test coverage\n");
        content.push_str("- Ensure all tests pass before committing\n");

        Ok(content)
    }

    fn file_path(&self) -> &str {
        ".cursorrules"
    }
}
