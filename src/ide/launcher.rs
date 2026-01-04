use crate::error::{KayfabeError, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum IDE {
    Cursor,
    Windsurf,
    Idea,
    Code,
    Claude,
}

impl IDE {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cursor" => Some(IDE::Cursor),
            "windsurf" => Some(IDE::Windsurf),
            "idea" => Some(IDE::Idea),
            "code" => Some(IDE::Code),
            "claude" => Some(IDE::Claude),
            _ => None,
        }
    }

    fn command(&self) -> &str {
        match self {
            IDE::Cursor => "cursor",
            IDE::Windsurf => "windsurf",
            IDE::Idea => "idea",
            IDE::Code => "code",
            IDE::Claude => "claude",
        }
    }

    fn is_available(&self) -> bool {
        Command::new("which")
            .arg(self.command())
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

pub struct IDELauncher;

impl IDELauncher {
    pub fn detect_available() -> Vec<IDE> {
        vec![
            IDE::Cursor,
            IDE::Windsurf,
            IDE::Idea,
            IDE::Code,
            IDE::Claude,
        ]
        .into_iter()
        .filter(|ide| ide.is_available())
        .collect()
    }

    pub fn launch(ide: IDE, path: &Path) -> Result<()> {
        if !ide.is_available() {
            return Err(KayfabeError::IdeNotFound(ide.command().to_string()));
        }

        Command::new(ide.command()).arg(path).spawn()?;

        Ok(())
    }
}
