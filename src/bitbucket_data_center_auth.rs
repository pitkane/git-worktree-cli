use anyhow::Result;
use std::env;

const TOKEN_ENV_VAR: &str = "BITBUCKET_DATA_CENTER_HTTP_ACCESS_TOKEN";

pub struct BitbucketDataCenterAuth;

impl BitbucketDataCenterAuth {
    pub fn new(_project_key: String, _repo_slug: String, _base_url: String) -> Result<Self> {
        Ok(BitbucketDataCenterAuth)
    }

    pub fn get_token(&self) -> Result<String> {
        env::var(TOKEN_ENV_VAR)
            .map_err(|_| anyhow::anyhow!(
                "No Bitbucket Data Center access token found. Please set the {} environment variable.\n\
                Run 'gwt auth bitbucket-data-center setup' for instructions.",
                TOKEN_ENV_VAR
            ))
            .and_then(|token| {
                if token.is_empty() {
                    Err(anyhow::anyhow!(
                        "Bitbucket Data Center access token is empty. Please set the {} environment variable.\n\
                        Run 'gwt auth bitbucket-data-center setup' for instructions.",
                        TOKEN_ENV_VAR
                    ))
                } else {
                    Ok(token)
                }
            })
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
    }

    #[test]
    fn test_auth_structure() {
        let auth = BitbucketDataCenterAuth::new(
            "PROJ".to_string(),
            "repo".to_string(),
            "https://git.example.com".to_string()
        );
        assert!(auth.is_ok());
    }
}