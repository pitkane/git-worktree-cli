use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};
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
    
    // Find the worktree to remove
    let target_worktree = match branch_name {
        None => {
            // No parameter given, remove current worktree
            let current_dir = std::env::current_dir()?;
            worktrees.iter().find(|wt| {
                current_dir.starts_with(&wt.path)
            }).ok_or_else(|| {
                anyhow::anyhow!("Not in a git worktree. Please specify a branch to remove.")
            })?
        }
        Some(target_branch) => {
            // Find worktree by branch name
            let found = worktrees.iter().find(|wt| {
                wt.branch.as_ref().map(|b| {
                    let clean_branch = if b.starts_with("refs/heads/") {
                        &b[11..]
                    } else {
                        b
                    };
                    clean_branch == target_branch
                }).unwrap_or(false)
            });
            
            if let Some(worktree) = found {
                worktree
            } else {
                // Try to find by path (last part of path)
                let found_by_path = worktrees.iter().find(|wt| {
                    wt.path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name == target_branch)
                        .unwrap_or(false)
                });
                
                if let Some(worktree) = found_by_path {
                    worktree
                } else {
                    println!("{}", format!("Error: Worktree for '{}' not found.", target_branch).red());
                    println!("\n{}", "Available worktrees:".yellow());
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
                        println!("  {} -> {}", branch_display.green(), worktree.path.display().to_string().dimmed());
                    }
                    bail!("Worktree not found");
                }
            }
        }
    };
    
    // Check if this is the bare repository
    if target_worktree.bare {
        bail!("Cannot remove the main (bare) repository.");
    }
    
    let branch_display = target_worktree.branch.as_ref().map(|b| {
        if b.starts_with("refs/heads/") {
            &b[11..]
        } else {
            b
        }
    }).unwrap_or_else(|| {
        &target_worktree.head[..8.min(target_worktree.head.len())]
    });
    
    // Show what will be removed
    println!("{}", "About to remove worktree:".cyan().bold());
    println!("  {}: {}", "Path".dimmed(), target_worktree.path.display());
    println!("  {}: {}", "Branch".dimmed(), branch_display.green());
    
    // Check if we're currently in the worktree being removed
    let current_dir = std::env::current_dir()?;
    let will_remove_current = current_dir.starts_with(&target_worktree.path);
    
    if will_remove_current {
        println!("\n{}", "⚠️  You are currently in this worktree. You will be moved to the project root after removal.".yellow());
    }
    
    // Ask for confirmation
    print!("\n{}", "Are you sure you want to remove this worktree? (y/N): ".cyan());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let confirmation = input.trim().to_lowercase();
    
    if confirmation != "y" && confirmation != "yes" {
        println!("{}", "Removal cancelled.".yellow());
        return Ok(());
    }
    
    // Find project root
    let project_root = find_project_root(&target_worktree.path)?;
    
    // Find another worktree to run git commands from
    let main_branches = ["main", "master", "dev", "develop"];
    let git_working_dir = worktrees.iter()
        .find(|wt| {
            // Try to find a main branch first
            wt.path != target_worktree.path && 
            wt.branch.as_ref().map(|b| {
                let clean_branch = if b.starts_with("refs/heads/") {
                    &b[11..]
                } else {
                    b
                };
                main_branches.contains(&clean_branch)
            }).unwrap_or(false)
        })
        .or_else(|| {
            // If no main branch, use any other worktree
            worktrees.iter().find(|wt| wt.path != target_worktree.path)
        })
        .ok_or_else(|| anyhow::anyhow!("No other worktrees found to execute git command from."))?;
    
    // Remove the worktree
    println!("\n{}", "Removing worktree...".cyan());
    git::execute_streaming(&[
        "worktree", "remove", 
        target_worktree.path.to_str().unwrap(), 
        "--force"
    ], Some(&git_working_dir.path))?;
    
    println!("{}", format!("✓ Worktree removed: {}", target_worktree.path.display()).green());
    
    // Delete the branch if it's not a main branch
    if !main_branches.contains(&branch_display) {
        match git::execute_streaming(&[
            "branch", "-D", branch_display
        ], Some(&git_working_dir.path)) {
            Ok(_) => {
                println!("{}", format!("✓ Branch deleted: {}", branch_display).green());
            }
            Err(_) => {
                println!("{}", format!("⚠️  Branch '{}' could not be deleted automatically", branch_display).yellow());
            }
        }
    } else {
        println!("{}", format!("✓ Branch: {} (preserved - main branch)", branch_display).green());
    }
    
    // If we removed the current worktree, change to project root before executing hooks
    if will_remove_current {
        std::env::set_current_dir(&project_root)?;
    }
    
    // Execute post-remove hooks
    hooks::execute_hooks(
        "postRemove",
        &project_root,
        &[
            ("branchName", branch_display),
            ("worktreePath", target_worktree.path.to_str().unwrap()),
        ]
    )?;
    
    // If we removed the current worktree, show message about moving to project root
    if will_remove_current {
        println!("{}", format!("✓ Please navigate to project root: {}", project_root.display()).green());
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

fn find_project_root(worktree_path: &Path) -> Result<PathBuf> {
    let mut search_path = worktree_path.to_path_buf();
    
    // Go up one level from the worktree to find the project root
    if search_path.pop() {
        let config_path = search_path.join("git-worktree-config.yaml");
        if config_path.exists() {
            return Ok(search_path);
        }
    }
    
    // If not found, try the current directory
    let current_dir = std::env::current_dir()?;
    let mut search_path = current_dir;
    
    loop {
        let config_path = search_path.join("git-worktree-config.yaml");
        if config_path.exists() {
            return Ok(search_path);
        }
        
        if !search_path.pop() {
            break;
        }
    }
    
    bail!("Could not find project root with git-worktree-config.yaml");
}