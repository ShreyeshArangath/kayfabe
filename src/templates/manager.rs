use crate::error::Result;
use crate::templates::Template;
use std::path::{Path, PathBuf};

pub struct TemplateManager;

impl TemplateManager {
    pub fn templates_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            crate::error::KayfabeError::Other("Could not find config directory".to_string())
        })?;
        Ok(config_dir.join("kayfabe").join("templates"))
    }

    pub fn list() -> Result<Vec<Template>> {
        let templates_dir = Self::templates_dir()?;

        if !templates_dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();

        for entry in std::fs::read_dir(&templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                if let Ok(template) = Self::load_template(&path) {
                    templates.push(template);
                }
            }
        }

        Ok(templates)
    }

    pub fn load(name: &str) -> Result<Template> {
        let templates_dir = Self::templates_dir()?;
        let path = templates_dir.join(format!("{}.toml", name));

        if !path.exists() {
            return Err(crate::error::KayfabeError::Other(format!(
                "Template not found: {}",
                name
            )));
        }

        Self::load_template(&path)
    }

    pub fn save(template: &Template) -> Result<()> {
        let templates_dir = Self::templates_dir()?;
        std::fs::create_dir_all(&templates_dir)?;

        let path = templates_dir.join(format!("{}.toml", template.name));

        let content = format!(
            "name = \"{}\"\ndescription = \"{}\"\n\n[content]\nvalue = \"\"\"\n{}\n\"\"\"",
            template.name, template.description, template.content
        );

        std::fs::write(&path, content)?;

        Ok(())
    }

    fn load_template(path: &Path) -> Result<Template> {
        let content = std::fs::read_to_string(path)?;
        let parsed: toml::Value = toml::from_str(&content).map_err(|e| {
            crate::error::KayfabeError::Other(format!("Failed to parse template: {}", e))
        })?;

        let name = parsed
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let description = parsed
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let template_content = parsed
            .get("content")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(Template {
            name,
            description,
            content: template_content,
            variables: Vec::new(),
        })
    }
}
