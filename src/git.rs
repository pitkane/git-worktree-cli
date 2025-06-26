use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Execute a git command with real-time output streaming
pub fn execute_streaming(args: &[&str], cwd: Option<&Path>) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let status = cmd.status().context("Failed to execute git command")?;

    if !status.success() {
        bail!("Git command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

/// Execute a git command and capture output
pub fn execute_capture(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output().context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git command failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Clone a repository with streaming output
pub fn clone(repo_url: &str, target_dir: &str) -> Result<()> {
    println!("{}", format!("Cloning {}...", repo_url).cyan());
    execute_streaming(&["clone", repo_url, target_dir], None)
}

/// Get the default branch name of a repository
pub fn get_default_branch(repo_path: &Path) -> Result<String> {
    execute_capture(&["symbolic-ref", "--short", "HEAD"], Some(repo_path))
}

/// Add a new worktree
#[allow(dead_code)]
pub fn add_worktree(
    git_dir: &Path,
    worktree_path: &Path,
    branch: &str,
    base_branch: &str,
) -> Result<()> {
    execute_streaming(
        &[
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            branch,
            base_branch,
        ],
        Some(git_dir),
    )
}

/// List all worktrees
pub fn list_worktrees(git_dir: Option<&Path>) -> Result<Vec<Worktree>> {
    let output = execute_capture(&["worktree", "list", "--porcelain"], git_dir)?;
    parse_worktree_list(&output)
}

/// Remove a worktree
#[allow(dead_code)]
pub fn remove_worktree(git_dir: &Path, worktree_path: &Path) -> Result<()> {
    execute_streaming(
        &["worktree", "remove", worktree_path.to_str().unwrap()],
        Some(git_dir),
    )
}

/// Delete a branch
#[allow(dead_code)]
pub fn delete_branch(git_dir: &Path, branch_name: &str) -> Result<()> {
    execute_streaming(&["branch", "-D", branch_name], Some(git_dir))
}

/// Check if a branch exists
pub fn branch_exists(git_dir: &Path, branch_name: &str) -> Result<(bool, bool)> {
    let local =
        execute_capture(&["branch", "--list", branch_name], Some(git_dir)).unwrap_or_default();

    let remote = execute_capture(
        &["branch", "-r", "--list", &format!("origin/{}", branch_name)],
        Some(git_dir),
    )
    .unwrap_or_default();

    Ok((!local.is_empty(), !remote.is_empty()))
}

/// Get the current git root directory
pub fn get_git_root() -> Result<Option<PathBuf>> {
    match execute_capture(&["rev-parse", "--show-toplevel"], None) {
        Ok(path) => Ok(Some(PathBuf::from(path))),
        Err(_) => Ok(None),
    }
}

#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub head: String,
    pub branch: Option<String>,
    pub bare: bool,
}

fn parse_worktree_list(output: &str) -> Result<Vec<Worktree>> {
    let mut worktrees = Vec::new();
    let mut current_worktree: Option<PartialWorktree> = None;

    #[derive(Default)]
    struct PartialWorktree {
        path: Option<PathBuf>,
        head: Option<String>,
        branch: Option<String>,
        bare: bool,
    }

    for line in output.lines() {
        if line.starts_with("worktree ") {
            if let Some(wt) = current_worktree.take() {
                if let (Some(path), Some(head)) = (wt.path, wt.head) {
                    worktrees.push(Worktree {
                        path,
                        head,
                        branch: wt.branch,
                        bare: wt.bare,
                    });
                }
            }
            current_worktree = Some(PartialWorktree {
                path: Some(PathBuf::from(&line[9..])),
                ..Default::default()
            });
        } else if line.starts_with("HEAD ") {
            if let Some(ref mut wt) = current_worktree {
                wt.head = Some(line[5..].to_string());
            }
        } else if line.starts_with("branch ") {
            if let Some(ref mut wt) = current_worktree {
                wt.branch = Some(line[7..].to_string());
            }
        } else if line == "bare" {
            if let Some(ref mut wt) = current_worktree {
                wt.bare = true;
            }
        }
    }

    if let Some(wt) = current_worktree {
        if let (Some(path), Some(head)) = (wt.path, wt.head) {
            worktrees.push(Worktree {
                path,
                head,
                branch: wt.branch,
                bare: wt.bare,
            });
        }
    }

    Ok(worktrees)
}
