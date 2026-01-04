pub mod claude;
pub mod cursor;
pub mod detector;
pub mod windsurf;

pub use claude::ClaudeGenerator;
pub use cursor::CursorGenerator;
pub use detector::ProjectDetector;
pub use windsurf::WindsurfGenerator;

use crate::error::Result;
use std::path::Path;

pub trait AgentGenerator {
    fn generate(&self, context: &ProjectContext) -> Result<String>;
    fn file_path(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub name: String,
    pub project_type: String,
    pub build_cmd: Option<String>,
    pub test_cmd: Option<String>,
    pub lint_cmd: Option<String>,
    pub is_workspace: bool,
    pub workspace_members: Vec<String>,
}

impl ProjectContext {
    pub fn detect(path: &Path) -> Result<Self> {
        ProjectDetector::detect(path)
    }
}
