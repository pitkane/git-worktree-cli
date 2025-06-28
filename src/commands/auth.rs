use anyhow::Result;
use crate::github::GitHubClient;
use crate::bitbucket_auth::{self, BitbucketAuth};
use crate::bitbucket_api::BitbucketClient;

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

use crate::cli::BitbucketCloudAuthAction;

#[tokio::main]
pub async fn run_bitbucket_cloud(action: Option<BitbucketCloudAuthAction>) -> Result<()> {
    match action {
        None | Some(BitbucketCloudAuthAction::Setup) => {
            bitbucket_auth::display_setup_instructions();
        }
        Some(BitbucketCloudAuthAction::Test) => {
            let (workspace, repo, email) = bitbucket_auth::get_auth_from_config()?;
            let auth = BitbucketAuth::new(workspace, repo, email)?;
            let client = BitbucketClient::new(auth);
            client.test_connection().await?;
        }
        Some(BitbucketCloudAuthAction::Remove) => {
            let (workspace, repo, email) = bitbucket_auth::get_auth_from_config()?;
            let auth = BitbucketAuth::new(workspace, repo, email)?;
            auth.remove_token()?;
        }
    }
    Ok(())
}