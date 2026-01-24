use anyhow::Result;

use crate::core::Project;

/// Handle the init command
pub async fn init(name: String, git_url: String) -> Result<()> {
    println!("🚀 Initializing Kayfabe project '{}'", name);
    println!("📍 Repository: {}", git_url);
    println!();

    // Initialize the project
    Project::init(&name, &git_url).await?;

    Ok(())
}
