use anyhow::{Result, bail};
use std::path::PathBuf;
use std::fs;
use colored::Colorize;
use tabled::{Table, Tabled, settings::Style};

use crate::{config, git, github, bitbucket_api, bitbucket_auth, bitbucket_data_center_api, bitbucket_data_center_auth};

#[derive(Tabled)]
struct WorktreeDisplay {
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "PULL REQUEST")]
    pull_request: String,
}

#[tokio::main]
pub async fn run() -> Result<()> {
    // Find a git directory to work with
    let git_dir = find_git_directory()?;
    
    // Get the list of worktrees
    let worktrees = git::list_worktrees(Some(&git_dir))?;
    
    if worktrees.is_empty() {
        println!("{}", "No worktrees found.".yellow());
        return Ok(());
    }
    
    // Try to get GitHub/Bitbucket info automatically
    let (github_client, bitbucket_client, bitbucket_data_center_client, repo_info) = {
        let github_client = github::GitHubClient::new();
        let mut bitbucket_client: Option<bitbucket_api::BitbucketClient> = None;
        let mut bitbucket_data_center_client: Option<bitbucket_data_center_api::BitbucketDataCenterClient> = None;
        
        if let Some((_, config)) = config::GitWorktreeConfig::find_config()? {
            let repo_url = &config.repository_url;
            
            // Use the configured sourceControl instead of URL pattern matching
            match config.source_control.as_str() {
                "bitbucket-cloud" => {
                    if let Some((workspace, repo)) = bitbucket_api::extract_bitbucket_info_from_url(repo_url) {
                        // Try to get Bitbucket Cloud auth
                        if let Ok(auth) = bitbucket_auth::BitbucketAuth::new(
                            workspace.clone(), 
                            repo.clone(), 
                            config.bitbucket_email.clone()
                        ) {
                            if auth.has_stored_token() {
                                bitbucket_client = Some(bitbucket_api::BitbucketClient::new(auth));
                            }
                        }
                        (Some(github_client), bitbucket_client, None, Some(("bitbucket-cloud".to_string(), workspace, repo)))
                    } else {
                        (Some(github_client), None, None, None)
                    }
                }
                "bitbucket-data-center" => {
                    // Always use get_auth_from_config for bitbucket-data-center since it can derive the API URL
                    if let Ok((base_url, project_key, repo_slug)) = bitbucket_data_center_auth::get_auth_from_config() {
                        if let Ok(auth) = bitbucket_data_center_auth::BitbucketDataCenterAuth::new(
                            project_key.clone(), 
                            repo_slug.clone(), 
                            base_url.clone()
                        ) {
                            if auth.get_token().is_ok() {
                                bitbucket_data_center_client = Some(bitbucket_data_center_api::BitbucketDataCenterClient::new(auth, base_url));
                            }
                        }
                        (Some(github_client), None, bitbucket_data_center_client, Some(("bitbucket-data-center".to_string(), project_key, repo_slug)))
                    } else {
                        // Could not get auth config - extract repo info for display but no client
                        let (owner, repo) = github::GitHubClient::parse_github_url(repo_url)
                            .unwrap_or_else(|| ("".to_string(), "".to_string()));
                        if !owner.is_empty() && !repo.is_empty() {
                            (Some(github_client), None, None, Some(("bitbucket-data-center".to_string(), owner, repo)))
                        } else {
                            (Some(github_client), None, None, None)
                        }
                    }
                }
                "github" | _ => {
                    // Try GitHub
                    let (owner, repo) = github::GitHubClient::parse_github_url(repo_url)
                        .unwrap_or_else(|| ("".to_string(), "".to_string()));
                        
                    if !owner.is_empty() && !repo.is_empty() {
                        (Some(github_client), None, None, Some(("github".to_string(), owner, repo)))
                    } else {
                        (Some(github_client), None, None, None)
                    }
                }
            }
        } else {
            (Some(github_client), None, None, None)
        }
    };
    
    let has_pr_info = repo_info.is_some() && match &repo_info {
        Some((platform, _, _)) => {
            match platform.as_str() {
                "github" => github_client.as_ref().map(|c| c.has_auth()).unwrap_or(false),
                "bitbucket-cloud" => bitbucket_client.is_some(),
                "bitbucket-data-center" => bitbucket_data_center_client.is_some(),
                _ => false,
            }
        }
        None => false,
    };
    
    // Convert to display format
    let mut display_worktrees: Vec<WorktreeDisplay> = Vec::new();
    
    for wt in worktrees {
            let branch = wt.branch.map(|b| {
                // Clean up branch names - remove refs/heads/ prefix
                if b.starts_with("refs/heads/") {
                    b[11..].to_string()
                } else {
                    b
                }
            }).unwrap_or_else(|| {
                if wt.bare {
                    "(bare)".to_string()
                } else {
                    wt.head.chars().take(8).collect()
                }
            });
            
            // Fetch PR info if available
            let pull_request = if has_pr_info && !wt.bare && branch != "(bare)" {
                match &repo_info {
                    Some((platform, owner_or_workspace, repo)) => {
                        match platform.as_str() {
                            "github" => {
                                if let Some(ref client) = github_client {
                                    match client.get_pull_requests(owner_or_workspace, repo, &branch) {
                                        Ok(prs) => {
                                            if prs.is_empty() {
                                                "-".to_string()
                                            } else {
                                                // Show the most recent PR with URL
                                                let pr = &prs[0];
                                                let status_text = if pr.draft {
                                                    "DRAFT"
                                                } else if pr.state.to_lowercase() == "open" {
                                                    "OPEN"
                                                } else if pr.state.to_lowercase() == "closed" {
                                                    "CLOSED"
                                                } else if pr.state.to_lowercase() == "merged" {
                                                    "MERGED"
                                                } else {
                                                    &pr.state.to_uppercase()
                                                };
                                                format!("{} ({})", pr.html_url, status_text)
                                            }
                                        }
                                        Err(_) => "?".to_string(),
                                    }
                                } else {
                                    "-".to_string()
                                }
                            }
                            "bitbucket-cloud" => {
                                if let Some(ref client) = bitbucket_client {
                                    match client.get_pull_requests(owner_or_workspace, repo).await {
                                        Ok(prs) => {
                                            // Find PR for this branch
                                            let branch_pr = prs.iter().find(|pr| {
                                                pr.source.branch.name == branch
                                            });
                                            
                                            if let Some(pr) = branch_pr {
                                                let status_text = pr.state.to_uppercase();
                                                // Extract URL from links
                                                let url = if let Some(html_link) = pr.links.get("html") {
                                                    if let Some(href) = html_link.get("href") {
                                                        href.as_str().unwrap_or("").to_string()
                                                    } else {
                                                        format!("PR #{}", pr.id)
                                                    }
                                                } else {
                                                    format!("PR #{}", pr.id)
                                                };
                                                format!("{} ({})", url, status_text)
                                            } else {
                                                "-".to_string()
                                            }
                                        }
                                        Err(_) => "?".to_string(),
                                    }
                                } else {
                                    "-".to_string()
                                }
                            }
                            "bitbucket-data-center" => {
                                if let Some(ref client) = bitbucket_data_center_client {
                                    match client.get_pull_requests(owner_or_workspace, repo).await {
                                        Ok(prs) => {
                                            // Find PR for this branch
                                            let branch_pr = prs.iter().find(|pr| {
                                                pr.from_ref.display_id == branch
                                            });
                                            
                                            if let Some(pr) = branch_pr {
                                                let status_text = pr.state.to_uppercase();
                                                // Extract URL from links
                                                let url = if let Some(self_link) = pr.links.get("self") {
                                                    if let Some(links_array) = self_link.as_array() {
                                                        if let Some(first_link) = links_array.first() {
                                                            if let Some(href) = first_link.get("href") {
                                                                href.as_str().unwrap_or("").to_string()
                                                            } else {
                                                                format!("PR #{}", pr.id)
                                                            }
                                                        } else {
                                                            format!("PR #{}", pr.id)
                                                        }
                                                    } else {
                                                        format!("PR #{}", pr.id)
                                                    }
                                                } else {
                                                    format!("PR #{}", pr.id)
                                                };
                                                format!("{} ({})", url, status_text)
                                            } else {
                                                "-".to_string()
                                            }
                                        }
                                        Err(_) => "?".to_string(),
                                    }
                                } else {
                                    "-".to_string()
                                }
                            }
                            _ => "-".to_string(),
                        }
                    }
                    None => "-".to_string(),
                }
            } else {
                "-".to_string()
            };
            
            display_worktrees.push(WorktreeDisplay {
                branch,
                pull_request,
            });
    }
    
    // Create and display the table
    let mut table = Table::new(display_worktrees);
    table.with(Style::sharp());
    
    // Apply coloring to the table output
    let table_string = table.to_string();
    let colored_table = apply_colors_to_table(&table_string);
    println!("{}", colored_table);
    
    if !has_pr_info {
        if let Some((_, config)) = config::GitWorktreeConfig::find_config()? {
            match config.source_control.as_str() {
                "bitbucket-cloud" => {
                    println!("\n{}", "Tip: Run 'gwt auth bitbucket-cloud setup' to enable Bitbucket Cloud pull request information".dimmed());
                }
                "bitbucket-data-center" => {
                    println!("\n{}", "Tip: Run 'gwt auth bitbucket-data-center setup' to enable Bitbucket Data Center pull request information".dimmed());
                }
                "github" | _ => {
                    println!("\n{}", "Tip: Run 'gh auth login' to enable GitHub pull request information".dimmed());
                }
            }
        }
    }
    
    Ok(())
}

fn apply_colors_to_table(table_str: &str) -> String {
    let lines: Vec<&str> = table_str.lines().collect();
    let mut result = String::new();
    
    for line in lines {
        let mut colored_line = line.to_string();
        
        // Color status indicators
        if line.contains("(OPEN)") {
            colored_line = colored_line.replace("(OPEN)", &format!("({})", "open".green()));
        } else if line.contains("(CLOSED)") {
            colored_line = colored_line.replace("(CLOSED)", &format!("({})", "closed".red()));
        } else if line.contains("(MERGED)") {
            colored_line = colored_line.replace("(MERGED)", &format!("({})", "merged".green()));
        } else if line.contains("(DRAFT)") {
            colored_line = colored_line.replace("(DRAFT)", &format!("({})", "draft".yellow()));
        }
        
        // Color URLs - find complete URLs and color them
        if let Some(url_start) = line.find("https://github.com/") {
            if let Some(url_end) = line[url_start..].find(" (") {
                let url = &line[url_start..url_start + url_end];
                colored_line = colored_line.replace(url, &format!("{}", url.blue().underline()));
            }
        } else if let Some(url_start) = line.find("https://bitbucket.org/") {
            if let Some(url_end) = line[url_start..].find(" (") {
                let url = &line[url_start..url_start + url_end];
                colored_line = colored_line.replace(url, &format!("{}", url.blue().underline()));
            }
        } else if let Some(url_start) = line.find("https://") {
            // For Bitbucket Data Center or other https URLs
            if let Some(url_end) = line[url_start..].find(" (") {
                let url = &line[url_start..url_start + url_end];
                // Only color if it looks like a reasonable URL (contains some path)
                if url.contains("/") && url.len() > 10 {
                    colored_line = colored_line.replace(url, &format!("{}", url.blue().underline()));
                }
            }
        }
        
        result.push_str(&colored_line);
        result.push('\n');
    }
    
    result.trim_end().to_string()
}

fn find_git_directory() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    
    // First, try to find git-worktree-config.yaml to determine if we're in a worktree project
    let mut search_path = current_dir.clone();
    let mut project_root: Option<PathBuf> = None;
    
    loop {
        let config_path = search_path.join("git-worktree-config.yaml");
        if config_path.exists() {
            project_root = Some(search_path);
            break;
        }
        
        if !search_path.pop() {
            break;
        }
    }
    
    if let Some(project_root) = project_root {
        // Found config file, look for any existing worktree to use for git commands
        let entries = fs::read_dir(&project_root)?;
        
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_path = entry.path();
                let git_path = dir_path.join(".git");
                if git_path.exists() {
                    return Ok(dir_path);
                }
            }
        }
        
        bail!("No existing worktrees found in project root. Create one first using gwt init.");
    } else {
        // No config found, check if we're directly in a git repository
        if let Some(git_root) = git::get_git_root()? {
            Ok(git_root)
        } else {
            bail!("Not in a git repository or project root with git-worktree-config.yaml");
        }
    }
}