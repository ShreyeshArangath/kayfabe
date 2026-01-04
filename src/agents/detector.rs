use crate::agents::ProjectContext;
use crate::error::Result;
use std::path::Path;

pub struct ProjectDetector;

impl ProjectDetector {
    pub fn detect(path: &Path) -> Result<ProjectContext> {
        if path.join("Cargo.toml").exists() {
            Self::detect_rust(path)
        } else if path.join("package.json").exists() {
            Self::detect_javascript(path)
        } else if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
            Self::detect_python(path)
        } else if path.join("go.mod").exists() {
            Self::detect_go(path)
        } else {
            Ok(Self::default_context(path))
        }
    }

    fn detect_rust(path: &Path) -> Result<ProjectContext> {
        let cargo_toml = path.join("Cargo.toml");
        let content = std::fs::read_to_string(&cargo_toml)?;

        let name = Self::extract_toml_value(&content, "name")
            .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());

        let is_workspace = content.contains("[workspace]");
        let mut workspace_members = Vec::new();

        if is_workspace {
            if let Some(members_section) = content.split("members = [").nth(1) {
                if let Some(members_str) = members_section.split(']').next() {
                    workspace_members = members_str
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }

        Ok(ProjectContext {
            name,
            project_type: "rust".to_string(),
            build_cmd: Some("cargo build".to_string()),
            test_cmd: Some("cargo test".to_string()),
            lint_cmd: Some("cargo clippy".to_string()),
            is_workspace,
            workspace_members,
        })
    }

    fn detect_javascript(path: &Path) -> Result<ProjectContext> {
        let package_json = path.join("package.json");
        let content = std::fs::read_to_string(&package_json)?;

        let name = Self::extract_json_value(&content, "name")
            .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());

        let has_typescript = path.join("tsconfig.json").exists();
        let project_type = if has_typescript {
            "typescript"
        } else {
            "javascript"
        };

        let package_manager = if path.join("pnpm-lock.yaml").exists() {
            "pnpm"
        } else if path.join("yarn.lock").exists() {
            "yarn"
        } else {
            "npm"
        };

        Ok(ProjectContext {
            name,
            project_type: project_type.to_string(),
            build_cmd: Some(format!("{} run build", package_manager)),
            test_cmd: Some(format!("{} run test", package_manager)),
            lint_cmd: Some(format!("{} run lint", package_manager)),
            is_workspace: content.contains("\"workspaces\""),
            workspace_members: Vec::new(),
        })
    }

    fn detect_python(path: &Path) -> Result<ProjectContext> {
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        let has_poetry = path.join("poetry.lock").exists();
        let has_pipenv = path.join("Pipfile").exists();

        let (test_cmd, lint_cmd) = if has_poetry {
            (
                "poetry run pytest".to_string(),
                "poetry run ruff check .".to_string(),
            )
        } else if has_pipenv {
            (
                "pipenv run pytest".to_string(),
                "pipenv run ruff check .".to_string(),
            )
        } else {
            ("python -m pytest".to_string(), "ruff check .".to_string())
        };

        Ok(ProjectContext {
            name,
            project_type: "python".to_string(),
            build_cmd: None,
            test_cmd: Some(test_cmd),
            lint_cmd: Some(lint_cmd),
            is_workspace: false,
            workspace_members: Vec::new(),
        })
    }

    fn detect_go(path: &Path) -> Result<ProjectContext> {
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        Ok(ProjectContext {
            name,
            project_type: "go".to_string(),
            build_cmd: Some("go build".to_string()),
            test_cmd: Some("go test ./...".to_string()),
            lint_cmd: Some("golangci-lint run".to_string()),
            is_workspace: false,
            workspace_members: Vec::new(),
        })
    }

    fn default_context(path: &Path) -> ProjectContext {
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        ProjectContext {
            name,
            project_type: "unknown".to_string(),
            build_cmd: None,
            test_cmd: None,
            lint_cmd: None,
            is_workspace: false,
            workspace_members: Vec::new(),
        }
    }

    fn extract_toml_value(content: &str, key: &str) -> Option<String> {
        for line in content.lines() {
            if line.trim().starts_with(&format!("{} =", key)) {
                if let Some(value) = line.split('=').nth(1) {
                    return Some(value.trim().trim_matches('"').to_string());
                }
            }
        }
        None
    }

    fn extract_json_value(content: &str, key: &str) -> Option<String> {
        for line in content.lines() {
            if line.contains(&format!("\"{}\"", key)) {
                if let Some(value_part) = line.split(':').nth(1) {
                    let value = value_part.trim().trim_matches(',').trim_matches('"');
                    return Some(value.to_string());
                }
            }
        }
        None
    }
}
