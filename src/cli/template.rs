use crate::error::Result;
use crate::templates::{Template, TemplateManager};
use console::style;

pub struct TemplateCommand;

impl TemplateCommand {
    pub fn list() -> Result<()> {
        let templates = TemplateManager::list()?;

        if templates.is_empty() {
            println!("{}", style("No templates found").yellow());
            println!(
                "\nCreate a template with: {}",
                style("kayfabe template create <name>").cyan()
            );
            return Ok(());
        }

        println!("{}", style("Available Templates:").bold());
        println!();

        for template in templates {
            println!(
                "  {} - {}",
                style(&template.name).cyan().bold(),
                style(&template.description).dim()
            );
        }

        Ok(())
    }

    pub fn create(name: String, description: Option<String>) -> Result<()> {
        let template = Template {
            name: name.clone(),
            description: description.unwrap_or_else(|| format!("Template: {}", name)),
            content: String::new(),
            variables: Vec::new(),
        };

        TemplateManager::save(&template)?;

        println!(
            "{} Template created: {}",
            style("✓").green(),
            style(&name).cyan()
        );

        let templates_dir = TemplateManager::templates_dir()?;
        let template_path = templates_dir.join(format!("{}.toml", name));

        println!(
            "\nEdit the template at: {}",
            style(template_path.display()).cyan()
        );

        Ok(())
    }

    pub fn show(name: String) -> Result<()> {
        let template = TemplateManager::load(&name)?;

        println!(
            "{}",
            style(format!("=== Template: {} ===", template.name))
                .bold()
                .cyan()
        );
        println!("{}", style(&template.description).dim());
        println!();
        println!("{}", template.content);

        Ok(())
    }

    pub fn delete(name: String) -> Result<()> {
        let templates_dir = TemplateManager::templates_dir()?;
        let path = templates_dir.join(format!("{}.toml", name));

        if !path.exists() {
            return Err(crate::error::KayfabeError::Other(format!(
                "Template not found: {}",
                name
            )));
        }

        std::fs::remove_file(&path)?;

        println!(
            "{} Template deleted: {}",
            style("✓").green(),
            style(&name).cyan()
        );

        Ok(())
    }
}
