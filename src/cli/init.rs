use anyhow::Result;
use crate::core::project::Project;
use crate::utils::interactive::{wizard_init, HEADER_COLOR, RESET_COLOR};

/// Handle the init command
pub async fn init(
    name: Option<String>,
    git_url: Option<String>,
    interactive: bool,
) -> Result<()> {
    // Determine mode: use wizard if interactive flag is set OR if params are missing
    let use_wizard = interactive || name.is_none() || git_url.is_none();

    let (project_name, repo_url) = if use_wizard {
        // Run wizard to collect inputs
        wizard_init()?
    } else {
        // Use provided CLI arguments (direct mode)
        (name.unwrap(), git_url.unwrap())
    };

    println!("\n{}🚀 Initializing Kayfabe project '{}'...{}", HEADER_COLOR, project_name, RESET_COLOR);
    println!("📍 Repository: {}", repo_url);
    println!();

    // Initialize the project
    Project::init(&project_name, &repo_url).await?;

    Ok(())
}
