# Git Worktree Scripts TODO

## Completed
- [x] Create gwtlist command to display worktrees in formatted manner
- [x] Add git-worktree-config.yaml creation to gwtinit command
  - Stores repository URL, main branch, and creation timestamp
  - Helps with future Bitbucket PR integration

## Pending
- [ ] Add Bitbucket PR search functionality for branches
  - Search for open PRs associated with each worktree branch
  - Display PR status inline with worktree list
  - Add ability to open PR in browser
  - Use repository URL from git-worktree-config.yaml

## Future Enhancements
- [ ] Add filtering options to gwtlist (by branch pattern, age, etc.)
- [ ] Add colorized output for better readability
- [ ] Show last commit date/author for each worktree
- [ ] Add ability to switch between worktrees quickly
- [ ] Add cleanup command for stale worktrees
- [ ] Use git-worktree-config.yaml for additional features:
  - Store worktree metadata (creation date, last accessed, etc.)
  - Track branch relationships
  - Store Bitbucket project/repo keys