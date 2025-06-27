use anyhow::{Result, bail};
use std::path::PathBuf;
use std::fs;
use colored::Colorize;
use tabled::{Table, Tabled, settings::Style};

use crate::{config, git, github};

#[derive(Tabled)]
struct WorktreeDisplay {
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "PULL REQUEST")]
    pull_request: String,
}

pub fn run() -> Result<()> {
    // Find a git directory to work with
    let git_dir = find_git_directory()?;
    
    // Get the list of worktrees
    let worktrees = git::list_worktrees(Some(&git_dir))?;
    
    if worktrees.is_empty() {
        println!("{}", "No worktrees found.".yellow());
        return Ok(());
    }
    
    // Try to get GitHub info
    let github_client = github::GitHubClient::new();
    let (owner, repo) = if let Some((_, config)) = config::GitWorktreeConfig::find_config()? {
        github::GitHubClient::parse_github_url(&config.repository_url)
            .unwrap_or_else(|| ("".to_string(), "".to_string()))
    } else {
        ("".to_string(), "".to_string())
    };
    
    let has_github_info = !owner.is_empty() && !repo.is_empty() && github_client.has_auth();
    
    // Convert to display format
    let display_worktrees: Vec<WorktreeDisplay> = worktrees
        .into_iter()
        .map(|wt| {
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
            let pull_request = if has_github_info && !wt.bare && branch != "(bare)" {
                match github_client.get_pull_requests(&owner, &repo, &branch) {
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
            };
            
            WorktreeDisplay {
                branch,
                pull_request,
            }
        })
        .collect();
    
    // Create and display the table
    let mut table = Table::new(display_worktrees);
    table.with(Style::sharp());
    
    // Apply coloring to the table output
    let table_string = table.to_string();
    let colored_table = apply_colors_to_table(&table_string);
    println!("{}", colored_table);
    
    if !has_github_info && !owner.is_empty() && !repo.is_empty() {
        println!("\n{}", "Tip: Run 'gh auth login' to enable GitHub pull request information".dimmed());
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