use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::fs;
use colored::Colorize;

use crate::config::GitWorktreeConfig;
use crate::git;
use crate::hooks;

pub fn run(branch_name: &str, print_path: bool) -> Result<()> {
    if branch_name.is_empty() {
        bail!("Error: Branch name is required\nUsage: gwt add <branch-name>");
    }

    // Determine git root and target path
    let (git_working_dir, target_path, project_root) = determine_paths(branch_name)?;
    
    if !print_path {
        println!("{}", format!("Preparing worktree (new branch '{}')", branch_name).cyan());
    }

    // Get main branch from config
    let main_branch = get_main_branch(&project_root)?;
    
    // Check if branch exists locally or remotely
    let (local_exists, remote_exists) = git::branch_exists(&git_working_dir, branch_name)?;
    
    // Create worktree based on branch existence
    if local_exists {
        if !print_path {
            println!("{}", format!("Branch '{}' exists locally, checking out existing branch...", branch_name).yellow());
        }
        git::execute_streaming(&[
            "worktree", "add", 
            target_path.to_str().unwrap(), 
            branch_name
        ], Some(&git_working_dir))?;
    } else if remote_exists {
        if !print_path {
            println!("{}", format!("Branch '{}' exists remotely, checking out remote branch...", branch_name).yellow());
        }
        git::execute_streaming(&[
            "worktree", "add", 
            target_path.to_str().unwrap(), 
            "-b", branch_name, 
            &format!("origin/{}", branch_name)
        ], Some(&git_working_dir))?;
    } else {
        if !print_path {
            println!("{}", format!("Creating new branch '{}' from 'origin/{}'...", branch_name, main_branch).cyan());
        }
        git::execute_streaming(&[
            "worktree", "add", 
            "--no-track",
            target_path.to_str().unwrap(), 
            "-b", branch_name, 
            &format!("origin/{}", main_branch)
        ], Some(&git_working_dir))?;
    }

    // Handle output based on print_path flag
    if print_path {
        // For shell integration - print only the path
        println!("{}", target_path.display());
    } else {
        // Normal success messages
        println!("{}", format!("✓ Worktree created at: {}", target_path.display()).green());
        println!("{}", format!("✓ Branch: {}", branch_name).green());
        
        // Execute post-add hooks only in normal mode
        hooks::execute_hooks(
            "postAdd",
            &target_path,
            &[
                ("branchName", branch_name),
                ("worktreePath", target_path.to_str().unwrap()),
            ]
        )?;
    }

    Ok(())
}

fn determine_paths(branch_name: &str) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let current_dir = std::env::current_dir()?;
    
    // First, look for git-worktree-config.yaml in current directory or upward
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
        // Found config file, use this as the project root
        let target_path = project_root.join(branch_name);
        
        // Find an existing worktree to use for git commands
        let entries = fs::read_dir(&project_root)?;
        let mut found_worktree: Option<PathBuf> = None;

        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_path = entry.path();
                let git_path = dir_path.join(".git");
                if git_path.exists() {
                    found_worktree = Some(dir_path);
                    break;
                }
            }
        }

        let git_working_dir = found_worktree
            .ok_or_else(|| anyhow::anyhow!("No existing worktrees found in project root. Create one first using gwt init."))?;
        
        Ok((git_working_dir, target_path, project_root))
    } else {
        // No config found, check if we're in a git repository
        if let Some(_git_root) = git::get_git_root()? {
            bail!("Found git repository but no git-worktree-config.yaml. This doesn't appear to be a worktree project.");
        } else {
            bail!("Not in a git repository or project root with git-worktree-config.yaml");
        }
    }
}

fn get_main_branch(project_root: &Path) -> Result<String> {
    let config_path = project_root.join("git-worktree-config.yaml");
    if config_path.exists() {
        let config = GitWorktreeConfig::load(&config_path)?;
        Ok(config.main_branch)
    } else {
        // Fallback to detecting from git if no config
        if let Some(git_root) = git::get_git_root()? {
            git::get_default_branch(&git_root)
        } else {
            Ok("main".to_string())
        }
    }
}