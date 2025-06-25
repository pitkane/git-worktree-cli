use std::path::PathBuf;

pub fn clean_branch_name(branch: &str) -> String {
    branch.strip_prefix("refs/heads/").unwrap_or(branch).to_string()
}