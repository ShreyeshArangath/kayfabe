pub mod installer;
pub mod templates;

pub use installer::AgentInstaller;

use crate::error::Result;
use std::path::Path;

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
