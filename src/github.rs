use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::thread;
use std::time;

// GitHub OAuth App Client ID
// This should be moved to a config file or environment variable in production
const CLIENT_ID: &str = "YOUR_CLIENT_ID_HERE";

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub draft: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthToken {
    access_token: String,
    token_type: String,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

pub struct GitHubClient {
    token: Option<String>,
    client: reqwest::blocking::Client,
}

impl GitHubClient {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent("git-worktree-cli")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            token: Self::load_token(),
            client,
        }
    }

    fn load_token() -> Option<String> {
        // Priority order:
        // 1. Environment variable GITHUB_TOKEN
        // 2. Stored OAuth token from device flow
        // 3. gh CLI auth token (fallback)

        // Check environment variable
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            if !token.is_empty() {
                return Some(token);
            }
        }

        // Check stored OAuth token
        if let Ok(auth_token) = Self::load_auth_token() {
            // Check if token is expired
            if let Some(expires_at) = auth_token.expires_at {
                if expires_at > Utc::now() {
                    return Some(auth_token.access_token);
                }
            } else {
                // No expiration, assume it's valid
                return Some(auth_token.access_token);
            }
        }

        // Check gh CLI as fallback
        Self::get_gh_token()
    }

    fn get_gh_token() -> Option<String> {
        std::process::Command::new("gh")
            .args(["auth", "token"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                } else {
                    None
                }
            })
    }

    fn auth_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("gwt").join("auth.json"))
    }

    fn load_auth_token() -> Result<AuthToken> {
        let path = Self::auth_file_path()?;
        if !path.exists() {
            return Err(anyhow!("Auth file does not exist"));
        }

        let content = fs::read_to_string(&path)?;
        let token: AuthToken = serde_json::from_str(&content)?;
        Ok(token)
    }

    fn save_auth_token(token: &AuthToken) -> Result<()> {
        let path = Self::auth_file_path()?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&token)?;
        fs::write(&path, json)?;

        // Set restrictive permissions (owner read/write only)
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&path)?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    pub fn has_auth(&self) -> bool {
        self.token.is_some()
    }

    pub fn authenticate(&self) -> Result<()> {
        println!("{}", "Starting GitHub authentication...".green());

        // Request device code
        let device_response = self.request_device_code()?;

        println!("\n{}", "Please visit this URL in your browser:".yellow());
        println!("{}", device_response.verification_uri.blue().underline());
        println!("\n{}", "Enter this code:".yellow());
        println!("{}", device_response.user_code.green().bold());

        // Try to open the URL in the default browser
        if open::that(&device_response.verification_uri).is_ok() {
            println!("\n{}", "Opening browser...".dimmed());
        }

        println!("\n{}", "Waiting for authorization...".dimmed());

        // Poll for access token
        let access_token = self.poll_for_token(&device_response)?;

        // Save the token
        let auth_token = AuthToken {
            access_token: access_token.access_token,
            token_type: access_token.token_type,
            scope: access_token.scope,
            expires_at: None, // GitHub tokens don't expire by default
        };

        Self::save_auth_token(&auth_token)?;

        println!("\n{}", "✓ Authentication successful!".green().bold());
        println!("{}", "Your authentication token has been saved.".dimmed());

        Ok(())
    }

    fn request_device_code(&self) -> Result<DeviceCodeResponse> {
        let params = [
            ("client_id", CLIENT_ID),
            ("scope", "repo"),
        ];

        let response = self.client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .context("Failed to request device code")?;

        if response.status().is_success() {
            response.json::<DeviceCodeResponse>()
                .context("Failed to parse device code response")
        } else {
            let error: ErrorResponse = response.json()
                .context("Failed to parse error response")?;
            Err(anyhow!("GitHub API error: {} - {}", 
                error.error, 
                error.error_description.unwrap_or_default()))
        }
    }

    fn poll_for_token(&self, device_response: &DeviceCodeResponse) -> Result<AccessTokenResponse> {
        let expiry = Utc::now() + Duration::seconds(device_response.expires_in as i64);
        let poll_interval = time::Duration::from_secs(device_response.interval as u64);

        loop {
            if Utc::now() > expiry {
                return Err(anyhow!("Device code expired. Please try again."));
            }

            thread::sleep(poll_interval);

            let params = [
                ("client_id", CLIENT_ID),
                ("device_code", &device_response.device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ];

            let response = self.client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&params)
                .send()
                .context("Failed to poll for access token")?;

            if response.status().is_success() {
                let body = response.text()?;
                
                // Check if it's an error response
                if body.contains("\"error\"") {
                    let error: ErrorResponse = serde_json::from_str(&body)?;
                    match error.error.as_str() {
                        "authorization_pending" => {
                            // User hasn't authorized yet, continue polling
                            print!(".");
                            use std::io::{self, Write};
                            io::stdout().flush().ok();
                            continue;
                        }
                        "slow_down" => {
                            // We're polling too fast, wait longer
                            thread::sleep(poll_interval);
                            continue;
                        }
                        _ => {
                            return Err(anyhow!("Authorization failed: {} - {}", 
                                error.error,
                                error.error_description.unwrap_or_default()));
                        }
                    }
                } else {
                    // Success! Parse the access token
                    let token: AccessTokenResponse = serde_json::from_str(&body)?;
                    return Ok(token);
                }
            } else {
                return Err(anyhow!("Failed to poll for access token: {}", response.status()));
            }
        }
    }

    pub fn get_pull_requests(&self, owner: &str, repo: &str, branch: &str) -> Result<Vec<PullRequest>> {
        let token = self.token.as_ref()
            .ok_or_else(|| anyhow!("No GitHub authentication found. Run 'gwt auth github' to authenticate."))?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?head={}:{}&state=all",
            owner, repo, owner, branch
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .context("Failed to fetch pull requests")?;

        if response.status().is_success() {
            let prs: Vec<serde_json::Value> = response.json()
                .context("Failed to parse pull requests")?;

            Ok(prs.into_iter().map(|pr| PullRequest {
                number: pr["number"].as_u64().unwrap_or(0) as u32,
                title: pr["title"].as_str().unwrap_or("").to_string(),
                state: pr["state"].as_str().unwrap_or("").to_string(),
                html_url: pr["html_url"].as_str().unwrap_or("").to_string(),
                draft: pr["draft"].as_bool().unwrap_or(false),
            }).collect())
        } else if response.status() == 401 {
            Err(anyhow!("GitHub authentication failed. Run 'gwt auth github' to re-authenticate."))
        } else {
            Err(anyhow!("Failed to fetch pull requests: {}", response.status()))
        }
    }

    pub fn parse_github_url(url: &str) -> Option<(String, String)> {
        // Parse both HTTPS and SSH URLs
        if let Some(captures) = url.strip_prefix("https://github.com/") {
            let parts: Vec<&str> = captures.trim_end_matches(".git").split('/').collect();
            if parts.len() >= 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        } else if let Some(captures) = url.strip_prefix("git@github.com:") {
            let parts: Vec<&str> = captures.trim_end_matches(".git").split('/').collect();
            if parts.len() >= 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        }
        None
    }

    pub fn logout() -> Result<()> {
        let path = Self::auth_file_path()?;
        if path.exists() {
            fs::remove_file(&path)?;
            println!("{}", "✓ Successfully logged out from GitHub".green());
        } else {
            println!("{}", "No stored authentication found".yellow());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        let test_cases = vec![
            ("https://github.com/owner/repo.git", Some(("owner".to_string(), "repo".to_string()))),
            ("https://github.com/owner/repo", Some(("owner".to_string(), "repo".to_string()))),
            ("git@github.com:owner/repo.git", Some(("owner".to_string(), "repo".to_string()))),
            ("git@github.com:owner/repo", Some(("owner".to_string(), "repo".to_string()))),
            ("https://gitlab.com/owner/repo", None),
        ];

        for (url, expected) in test_cases {
            assert_eq!(GitHubClient::parse_github_url(url), expected);
        }
    }
}