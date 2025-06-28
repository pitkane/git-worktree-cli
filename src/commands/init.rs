use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;

use crate::cli::Provider;
use crate::config::{GitWorktreeConfig, CONFIG_FILENAME};
use crate::git;
use crate::hooks;
use crate::{github, bitbucket_api};

pub fn run(repo_url: &str, provider: Option<Provider>) -> Result<()> {
    // Detect or validate the repository provider
    let detected_provider = detect_repository_provider(repo_url, provider)?;
    
    println!("{}", format!("✓ Detected provider: {:?}", detected_provider).green());
    
    // Extract repository name from URL
    let repo_name = extract_repo_name(repo_url)?;
    let project_root = std::env::current_dir()?;
    
    // Remove existing clone directory if it exists
    if Path::new(&repo_name).exists() {
        fs::remove_dir_all(&repo_name)
            .context("Failed to remove existing directory")?;
    }
    
    // Clone the repository with streaming output (this is the key improvement!)
    git::clone(repo_url, &repo_name)?;
    
    // Get the default branch name
    let repo_path = PathBuf::from(&repo_name);
    let default_branch = git::get_default_branch(&repo_path)
        .context("Failed to get default branch")?;
    
    // Rename directory to match branch name
    let final_dir_name = &default_branch;
    if Path::new(final_dir_name).exists() {
        fs::remove_dir_all(final_dir_name)
            .context("Failed to remove existing directory")?;
    }
    
    fs::rename(&repo_name, final_dir_name)
        .context("Failed to rename directory")?;
    
    // Create configuration file
    let config = GitWorktreeConfig::new(repo_url.to_string(), default_branch.clone(), detected_provider);
    let config_path = project_root.join(CONFIG_FILENAME);
    config.save(&config_path)
        .context("Failed to save configuration")?;
    
    // Print success messages
    println!("{}", format!("✓ Repository cloned to: {}", final_dir_name).green());
    println!("{}", format!("✓ Default branch: {}", default_branch).green());
    println!("{}", format!("✓ Config saved to: {}", config_path.display()).green());
    
    // Execute post-init hooks
    let final_dir_path = project_root.join(final_dir_name);
    hooks::execute_hooks(
        "postInit",
        &final_dir_path,
        &[
            ("branchName", &default_branch),
            ("worktreePath", final_dir_path.to_str().unwrap()),
        ]
    )?;
    
    Ok(())
}

fn extract_repo_name(repo_url: &str) -> Result<String> {
    let name = repo_url
        .split('/')
        .last()
        .context("Invalid repository URL")?
        .strip_suffix(".git")
        .unwrap_or_else(|| repo_url.split('/').last().unwrap());
    
    Ok(name.to_string())
}

fn detect_repository_provider(repo_url: &str, provider: Option<Provider>) -> Result<Provider> {
    // First, try to auto-detect from URL
    let auto_detected = if github::GitHubClient::parse_github_url(repo_url).is_some() {
        Some(Provider::Github)
    } else if bitbucket_api::is_bitbucket_repository(repo_url) {
        Some(Provider::BitbucketCloud)
    } else {
        None
    };
    
    match (auto_detected, provider) {
        // Auto-detected and no explicit provider given - use auto-detected
        (Some(detected), None) => Ok(detected),
        
        // Auto-detected and explicit provider matches - use it
        (Some(detected), Some(explicit)) if std::mem::discriminant(&detected) == std::mem::discriminant(&explicit) => {
            Ok(explicit)
        }
        
        // Auto-detected and explicit provider conflicts - warn but use explicit
        (Some(detected), Some(explicit)) => {
            println!("{}", format!("⚠ URL suggests {:?} but --provider {:?} specified. Using {:?}.", 
                     detected, explicit, explicit).yellow());
            Ok(explicit)
        }
        
        // Not auto-detected and explicit provider given - use explicit
        (None, Some(explicit)) => Ok(explicit),
        
        // Not auto-detected and no explicit provider - error
        (None, None) => {
            Err(anyhow::anyhow!(
                "Could not detect repository provider from URL: {}\n\
                 Please specify the provider using --provider:\n\
                 - For GitHub: --provider github\n\
                 - For Bitbucket Cloud: --provider bitbucket-cloud\n\
                 - For Bitbucket Data Center: --provider bitbucket-data-center",
                repo_url
            ))
        }
    }
}