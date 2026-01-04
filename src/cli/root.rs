use crate::error::Result;
use crate::git::KayfabeRoot;
use console::style;

pub struct RootCommand;

impl RootCommand {
    pub fn execute() -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let root = KayfabeRoot::discover(&current_dir)?;

        println!("{} {}", style("Kayfabe root:").bold(), style(root.display()).cyan());

        Ok(())
    }
}
