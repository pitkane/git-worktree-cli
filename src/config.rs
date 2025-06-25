use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitWorktreeConfig {
    pub repository_url: String,
    pub main_branch: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hooks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_add: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_switch: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_remove: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_init: Option<Vec<String>>,
}

impl GitWorktreeConfig {
    pub fn new(repository_url: String, main_branch: String) -> Self {
        Self {
            repository_url,
            main_branch,
            created_at: Utc::now(),
            hooks: Some(Hooks {
                post_add: Some(vec!["# npm install".to_string()]),
                post_switch: Some(vec!["# echo 'Switched to branch ${branchName}'".to_string()]),
                post_remove: Some(vec!["# echo 'Removed worktree for branch ${branchName}'".to_string()]),
                post_init: Some(vec!["# echo 'Initialized git worktree project'".to_string()]),
            }),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let yaml_string = serde_yaml::to_string(self)
            .context("Failed to serialize config to YAML")?;
        
        fs::write(path, yaml_string)
            .context("Failed to write config file")?;
        
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        let config: Self = serde_yaml::from_str(&content)
            .context("Failed to parse YAML config")?;
        
        Ok(config)
    }

    pub fn find_config() -> Result<Option<(PathBuf, Self)>> {
        let mut current_dir = std::env::current_dir()?;
        
        loop {
            let config_path = current_dir.join("git-worktree-config.yaml");
            if config_path.exists() {
                let config = Self::load(&config_path)?;
                return Ok(Some((config_path, config)));
            }
            
            if !current_dir.pop() {
                break;
            }
        }
        
        Ok(None)
    }
}

pub const CONFIG_FILENAME: &str = "git-worktree-config.yaml";