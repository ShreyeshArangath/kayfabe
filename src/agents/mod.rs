pub mod installer;

pub use installer::AgentInstaller;

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
