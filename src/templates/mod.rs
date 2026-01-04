pub mod manager;

pub use manager::TemplateManager;

use crate::error::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub content: String,
    pub variables: Vec<String>,
}

impl Template {
    pub fn render(&self, context: &HashMap<String, String>) -> Result<String> {
        let mut result = self.content.clone();

        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}
