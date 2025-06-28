use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::bitbucket_data_center_auth::BitbucketDataCenterAuth;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BitbucketDataCenterUser {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    pub id: u64,
    pub slug: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BitbucketDataCenterProject {
    pub key: String,
    pub name: String,
    pub id: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BitbucketDataCenterRepository {
    pub slug: String,
    pub name: String,
    pub id: u64,
    pub project: BitbucketDataCenterProject,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BitbucketDataCenterBranch {
    pub id: String,
    #[serde(rename = "displayId")]
    pub display_id: String,
    pub repository: Option<BitbucketDataCenterRepository>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BitbucketDataCenterPullRequestRef {
    pub id: String,
    #[serde(rename = "displayId")]
    pub display_id: String,
    #[serde(rename = "latestCommit")]
    pub latest_commit: String,
    pub repository: BitbucketDataCenterRepository,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BitbucketDataCenterPullRequest {
    pub id: u64,
    pub version: u32,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub open: bool,
    pub closed: bool,
    pub author: BitbucketDataCenterUser,
    #[serde(rename = "fromRef")]
    pub from_ref: BitbucketDataCenterPullRequestRef,
    #[serde(rename = "toRef")]
    pub to_ref: BitbucketDataCenterPullRequestRef,
    #[serde(rename = "createdDate")]
    pub created_date: u64,
    #[serde(rename = "updatedDate")]
    pub updated_date: u64,
    pub links: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketDataCenterPullRequestsResponse {
    pub values: Vec<BitbucketDataCenterPullRequest>,
}

pub struct BitbucketDataCenterClient {
    client: Client,
    auth: BitbucketDataCenterAuth,
    base_url: String,
}

impl BitbucketDataCenterClient {
    pub fn new(auth: BitbucketDataCenterAuth, base_url: String) -> Self {
        let client = Client::new();
        BitbucketDataCenterClient { client, auth, base_url }
    }

    pub async fn get_pull_requests(
        &self,
        project_key: &str,
        repo_slug: &str,
    ) -> Result<Vec<BitbucketDataCenterPullRequest>> {
        let token = self.auth.get_token()?;
        let url = format!(
            "{}/rest/api/1.0/projects/{}/repos/{}/pull-requests",
            self.base_url.trim_end_matches('/'),
            project_key,
            repo_slug
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to Bitbucket Data Center API")?;

        if response.status().is_client_error() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            
            if status == 401 {
                return Err(anyhow::anyhow!(
                    "Authentication failed. Please check your Bitbucket Data Center access token and run 'gwt auth bitbucket-data-center' to update it."
                ));
            } else if status == 404 {
                return Err(anyhow::anyhow!(
                    "Repository not found: {}/{}. Please check the project key and repository slug.",
                    project_key, repo_slug
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "API request failed with status {}: {}",
                    status, text
                ));
            }
        }

        let pr_response: BitbucketDataCenterPullRequestsResponse = response
            .json()
            .await
            .context("Failed to parse Bitbucket Data Center API response")?;

        Ok(pr_response.values)
    }

    pub async fn test_connection(&self) -> Result<()> {
        let token = self.auth.get_token()?;
        let url = format!("{}/rest/api/1.0/users", self.base_url.trim_end_matches('/'));

        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to test Bitbucket Data Center API connection")?;

        if response.status().is_success() {
            println!("✓ Bitbucket Data Center API connection successful");
            Ok(())
        } else {
            let status = response.status();
            if status == 401 {
                Err(anyhow::anyhow!(
                    "Authentication failed. Please check your Bitbucket Data Center access token."
                ))
            } else {
                Err(anyhow::anyhow!(
                    "API connection failed with status: {}",
                    status
                ))
            }
        }
    }
}

pub fn extract_bitbucket_data_center_info_from_url(url: &str) -> Option<(String, String, String)> {
    // Parse URLs like:
    // https://git.acmeorg.com/scm/PROJECT/repository.git
    // https://git.acmeorg.com/projects/PROJECT/repos/repository
    // git@git.acmeorg.com:PROJECT/repository.git
    
    // Pattern for Data Center URLs with /scm/ path
    if let Some(captures) = regex::Regex::new(r"([^/]+)/scm/([^/]+)/([^/\.]+)")
        .ok()?
        .captures(url)
    {
        let base_url = captures.get(1)?.as_str();
        let project = captures.get(2)?.as_str();
        let repo = captures.get(3)?.as_str();
        
        // Reconstruct the base URL for API calls
        let api_base_url = if base_url.starts_with("http") {
            base_url.to_string()
        } else {
            format!("https://{}", base_url)
        };
        
        return Some((api_base_url, project.to_string(), repo.to_string()));
    }
    
    // Pattern for Data Center URLs with /projects/ path
    if let Some(captures) = regex::Regex::new(r"([^/]+)/projects/([^/]+)/repos/([^/\.]+)")
        .ok()?
        .captures(url)
    {
        let base_url = captures.get(1)?.as_str();
        let project = captures.get(2)?.as_str();
        let repo = captures.get(3)?.as_str();
        
        let api_base_url = if base_url.starts_with("http") {
            base_url.to_string()
        } else {
            format!("https://{}", base_url)
        };
        
        return Some((api_base_url, project.to_string(), repo.to_string()));
    }
    
    // Pattern for SSH URLs: git@host:project/repo.git
    if let Some(captures) = regex::Regex::new(r"git@([^:]+):([^/]+)/([^/\.]+)")
        .ok()?
        .captures(url)
    {
        let host = captures.get(1)?.as_str();
        let project = captures.get(2)?.as_str();
        let repo = captures.get(3)?.as_str();
        
        return Some((format!("https://{}", host), project.to_string(), repo.to_string()));
    }
    
    // Pattern for SSH URLs with protocol: ssh://git@host/project/repo.git
    if let Some(captures) = regex::Regex::new(r"ssh://git@([^/]+)/([^/]+)/([^/\.]+)")
        .ok()?
        .captures(url)
    {
        let host = captures.get(1)?.as_str();
        let project = captures.get(2)?.as_str();
        let repo = captures.get(3)?.as_str();
        
        return Some((format!("https://{}", host), project.to_string(), repo.to_string()));
    }

    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bitbucket_data_center_info_scm() {
        let url = "https://git.acmeorg.com/scm/PROJ/repo";
        let result = extract_bitbucket_data_center_info_from_url(url);
        assert_eq!(result, Some((
            "https://git.acmeorg.com".to_string(),
            "PROJ".to_string(),
            "repo".to_string()
        )));
    }

    #[test]
    fn test_extract_bitbucket_data_center_info_scm_git() {
        let url = "https://git.acmeorg.com/scm/PROJ/repo.git";
        let result = extract_bitbucket_data_center_info_from_url(url);
        assert_eq!(result, Some((
            "https://git.acmeorg.com".to_string(),
            "PROJ".to_string(),
            "repo".to_string()
        )));
    }

    #[test]
    fn test_extract_bitbucket_data_center_info_projects() {
        let url = "https://git.acmeorg.com/projects/PROJ/repos/repo";
        let result = extract_bitbucket_data_center_info_from_url(url);
        assert_eq!(result, Some((
            "https://git.acmeorg.com".to_string(),
            "PROJ".to_string(),
            "repo".to_string()
        )));
    }

    #[test]
    fn test_extract_bitbucket_data_center_info_ssh() {
        let url = "git@git.acmeorg.com:PROJ/repo.git";
        let result = extract_bitbucket_data_center_info_from_url(url);
        assert_eq!(result, Some((
            "https://git.acmeorg.com".to_string(),
            "PROJ".to_string(),
            "repo".to_string()
        )));
    }

    #[test]
    fn test_extract_bitbucket_data_center_info_ssh_protocol() {
        let url = "ssh://git@git.acmeorg.com/PROJECT_ID/REPO_ID.git";
        let result = extract_bitbucket_data_center_info_from_url(url);
        assert_eq!(result, Some((
            "https://git.acmeorg.com".to_string(),
            "PROJECT_ID".to_string(),
            "REPO_ID".to_string()
        )));
    }

    #[test]
    fn test_extract_bitbucket_data_center_info_invalid() {
        let url = "https://github.com/user/repo";
        let result = extract_bitbucket_data_center_info_from_url(url);
        assert_eq!(result, None);
    }

}