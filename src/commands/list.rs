use anyhow::{Result, bail};
use std::path::PathBuf;
use std::fs;
use colored::Colorize;
use tabled::{Table, Tabled, settings::Style};

use crate::git;

#[derive(Tabled)]
struct WorktreeDisplay {
    #[tabled(rename = "PATH")]
    path: String,
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "HEAD")]
    head: String,
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
    
    // Convert to display format
    let display_worktrees: Vec<WorktreeDisplay> = worktrees
        .into_iter()
        .map(|wt| WorktreeDisplay {
            path: wt.path.display().to_string(),
            branch: wt.branch.map(|b| {
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
            }),
            head: if wt.head.len() > 8 { 
                format!("{}...", &wt.head[..8])
            } else { 
                wt.head 
            },
        })
        .collect();
    
    // Create and display the table
    let mut table = Table::new(display_worktrees);
    table.with(Style::sharp());
    println!("{}", table);
    
    Ok(())
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