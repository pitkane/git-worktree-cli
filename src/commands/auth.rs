use anyhow::Result;
use crate::github::GitHubClient;

pub fn run(logout: bool) -> Result<()> {
    if logout {
        GitHubClient::logout()?;
    } else {
        let client = GitHubClient::new();
        if client.has_auth() {
            println!("✓ You are already authenticated with GitHub via gh CLI");
            println!("Run 'gh auth logout' to remove credentials");
        } else {
            println!("Please authenticate with GitHub using: gh auth login");
        }
    }
    Ok(())
}