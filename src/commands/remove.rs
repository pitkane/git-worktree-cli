use anyhow::Result;

pub fn run(branch_name: Option<&str>) -> Result<()> {
    match branch_name {
        Some(branch) => println!("Removing worktree for branch: {}", branch),
        None => println!("Removing current worktree"),
    }
    // TODO: Implement remove functionality
    Ok(())
}