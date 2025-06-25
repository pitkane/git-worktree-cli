use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::fs;
use colored::Colorize;

use crate::git;
use crate::hooks;

pub fn run(branch_name: Option<&str>) -> Result<()> {
    // Find a git directory to work with
    let git_dir = find_git_directory()?;
    
    // Get the list of worktrees
    let worktrees = git::list_worktrees(Some(&git_dir))?;
    
    if worktrees.is_empty() {
        println!("{}", "No worktrees found.".yellow());
        return Ok(());
    }
    
    match branch_name {
        None => {
            // No branch specified, show available worktrees
            println!("\n{}", "Available worktrees:".cyan().bold());
            println!("{}", "────────────────────".cyan());
            
            for worktree in &worktrees {
                let branch_display = worktree.branch.as_ref().map(|b| {
                    // Clean up branch names - remove refs/heads/ prefix
                    if b.starts_with("refs/heads/") {
                        &b[11..]
                    } else {
                        b
                    }
                }).unwrap_or_else(|| {
                    if worktree.bare {
                        "(bare)"
                    } else {
                        &worktree.head[..8.min(worktree.head.len())]
                    }
                });
                
                let bare_indicator = if worktree.bare { " (bare)" } else { "" };
                println!("  {}{}", branch_display.green(), bare_indicator.yellow());
            }
            
            println!("\n{}", "Usage: gwt switch <branch-name>".dimmed());
            return Ok(());
        }
        Some(target_branch) => {
            // Find the worktree for the specified branch
            let target_worktree = worktrees.iter().find(|wt| {
                wt.branch.as_ref().map(|b| {
                    let clean_branch = if b.starts_with("refs/heads/") {
                        &b[11..]
                    } else {
                        b
                    };
                    clean_branch == target_branch
                }).unwrap_or(false)
            });
            
            if let Some(worktree) = target_worktree {
                // Found the target worktree
                println!("{}", format!("Switching to worktree: {}", worktree.path.display()).cyan());
                
                // Execute post-switch hooks
                hooks::execute_hooks(
                    "postSwitch",
                    &worktree.path,
                    &[
                        ("branchName", target_branch),
                        ("worktreePath", worktree.path.to_str().unwrap()),
                    ]
                )?;
                
            } else {
                // Worktree not found
                println!("{}", format!("Error: Worktree for branch '{}' not found.", target_branch).red());
                println!("\n{}", "Available branches:".yellow());
                for worktree in &worktrees {
                    let branch_display = worktree.branch.as_ref().map(|b| {
                        if b.starts_with("refs/heads/") {
                            &b[11..]
                        } else {
                            b
                        }
                    }).unwrap_or_else(|| {
                        if worktree.bare {
                            "(bare)"
                        } else {
                            &worktree.head[..8.min(worktree.head.len())]
                        }
                    });
                    println!("  {}", branch_display.green());
                }
                bail!("Branch not found");
            }
        }
    }
    
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