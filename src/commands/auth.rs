use anyhow::Result;
use crate::github::GitHubClient;

pub fn run(logout: bool) -> Result<()> {
    if logout {
        GitHubClient::logout()?;
    } else {
        let client = GitHubClient::new();
        if client.has_auth() {
            println!("✓ You are already authenticated with GitHub");
            println!("Run 'gwt auth github --logout' to remove stored credentials");
        } else {
            client.authenticate()?;
        }
    }
    Ok(())
}