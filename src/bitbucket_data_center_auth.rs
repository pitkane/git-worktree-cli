use anyhow::{Context, Result};
use keyring::Entry;
use std::env;

const SERVICE_NAME: &str = "git-worktree-cli-bitbucket-data-center";
const TOKEN_ENV_VAR: &str = "BITBUCKET_DATA_CENTER_HTTP_ACCESS_TOKEN";

pub struct BitbucketDataCenterAuth {
    project_key: String,
    repo_slug: String,
    base_url: String,
    token_entry: Entry,
}

impl BitbucketDataCenterAuth {
    pub fn new(project_key: String, repo_slug: String, base_url: String) -> Result<Self> {
        // Use base_url/project/repo as the key identifier for better isolation
        let key_id = format!("{}/{}/{}", base_url, project_key, repo_slug);
        let token_entry = Entry::new(SERVICE_NAME, &key_id)
            .context("Failed to create keyring entry for Bitbucket Data Center token")?;
        
        Ok(BitbucketDataCenterAuth {
            project_key,
            repo_slug,
            base_url,
            token_entry,
        })
    }

    pub fn store_token(&self, token: &str) -> Result<()> {
        self.token_entry
            .set_password(token)
            .context("Failed to store Bitbucket Data Center access token in keyring")?;
        
        println!("✓ Bitbucket Data Center access token stored securely for {}/{}/{}", 
                 self.base_url, self.project_key, self.repo_slug);
        Ok(())
    }

    pub fn get_token(&self) -> Result<String> {
        // Check environment variable first
        if let Ok(token) = env::var(TOKEN_ENV_VAR) {
            if !token.is_empty() {
                return Ok(token);
            }
        }
        
        // Then check keyring
        self.token_entry
            .get_password()
            .context(format!(
                "No Bitbucket Data Center access token found. Please set the {} environment variable.\n\
                Run 'gwt auth bitbucket-data-center setup' for instructions.",
                TOKEN_ENV_VAR
            ))
    }

    pub fn remove_token(&self) -> Result<()> {
        self.token_entry
            .delete_credential()
            .context("Failed to remove Bitbucket Data Center access token from keyring")?;
        
        println!("✓ Bitbucket Data Center access token removed for {}/{}/{}", 
                 self.base_url, self.project_key, self.repo_slug);
        Ok(())
    }

    pub fn has_stored_token(&self) -> bool {
        // Check env var first
        if let Ok(token) = env::var(TOKEN_ENV_VAR) {
            if !token.is_empty() {
                return true;
            }
        }
        
        // Then check keyring
        self.token_entry.get_password().is_ok()
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn project_key(&self) -> &str {
        &self.project_key
    }

    pub fn repo_slug(&self) -> &str {
        &self.repo_slug
    }
}

pub fn get_auth_from_config() -> Result<(String, String, String)> {
    use crate::config::GitWorktreeConfig;
    use crate::bitbucket_data_center_api::extract_bitbucket_data_center_info_from_url;
    
    let (_, config) = GitWorktreeConfig::find_config()?
        .ok_or_else(|| anyhow::anyhow!("No git-worktree-config.yaml found"))?;
    
    // Check if this is a Bitbucket Data Center repository (not bitbucket.org)
    if config.repository_url.contains("bitbucket.org") {
        return Err(anyhow::anyhow!("This appears to be a Bitbucket Cloud repository, not Bitbucket Data Center"));
    }
    
    let (base_url, project_key, repo_slug) = extract_bitbucket_data_center_info_from_url(&config.repository_url)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse Bitbucket Data Center repository URL"))?;
    
    Ok((base_url, project_key, repo_slug))
}

pub fn display_setup_instructions() {
    println!("Setting up Bitbucket Data Center authentication\n");
    println!("1. Create an HTTP Access Token in your Bitbucket Data Center instance:");
    println!("   - Go to your Bitbucket Data Center instance");
    println!("   - Navigate to your profile settings");
    println!("   - Go to 'HTTP access tokens' or 'Personal access tokens'");
    println!("   - Create a new token with repository permissions\n");
    println!("2. Required permissions for the token:");
    println!("   - Repository: Read");
    println!("   - Pull requests: Read\n");
    println!("3. Copy the generated token\n");
    println!("4. Set the environment variable:");
    println!("   export {}=YOUR_TOKEN", TOKEN_ENV_VAR);
    println!("\nExample usage:");
    println!("   curl -H \"Authorization: Bearer ${}\" \\", TOKEN_ENV_VAR);
    println!("        \"https://git.acmeorg.com/rest/api/1.0/projects/PROJECT/repos/REPO/pull-requests\"");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitbucket_data_center_auth_creation() {
        let auth = BitbucketDataCenterAuth::new(
            "PROJECT".to_string(),
            "repository".to_string(),
            "https://git.acmeorg.com".to_string()
        );
        assert!(auth.is_ok());
        
        let auth = auth.unwrap();
        assert_eq!(auth.project_key(), "PROJECT");
        assert_eq!(auth.repo_slug(), "repository");
        assert_eq!(auth.base_url(), "https://git.acmeorg.com");
    }

    #[test]
    fn test_key_generation() {
        let auth = BitbucketDataCenterAuth::new(
            "PROJ".to_string(),
            "repo".to_string(),
            "https://git.example.com".to_string()
        ).unwrap();
        
        // The keyring key should include base_url/project/repo for uniqueness
        assert_eq!(auth.project_key(), "PROJ");
        assert_eq!(auth.repo_slug(), "repo");
        assert_eq!(auth.base_url(), "https://git.example.com");
    }
}