# Git Worktree Scripts TODO

## Completed
- [x] Create gwtlist command to display worktrees in formatted manner
- [x] Add git-worktree-config.yaml creation to gwtinit command
  - Stores repository URL, main branch, and creation timestamp
  - Helps with future Bitbucket PR integration
- [x] Implement gwtadd command to create new worktrees
  - Works from project root or worktree directory
  - Supports folder names with slashes (e.g., feature/IP-487-replace-icon-buttons)
  - Automatically branches from main/default branch
  - Detects existing branches and checks them out instead of creating new ones
  - Automatically navigates to new worktree after creation
- [x] Implement gwtswitch command
  - Quick switching between existing worktrees
  - Navigate to selected worktree directory
  - Shows available worktrees when no branch specified
  - Helpful error messages when branch not found
- [x] Add tab completion for gwtswitch command
  - Shows available local worktrees for completion
  - Displays helpful message when no worktrees found
  - Filters completions based on current input
- [x] Implement gwtremove command
  - Remove current worktree when no parameter given
  - Remove specified worktree when parameter provided
  - Safety checks to prevent removing main/bare repository
  - Automatic navigation to project root when removing current worktree
  - Clear confirmation messages and error handling

## Pending
- [ ] Add Bitbucket PR search functionality for branches
  - Search for open PRs associated with each worktree branch
  - Display PR status inline with worktree list
  - Add ability to open PR in browser
  - Use repository URL from git-worktree-config.yaml
- [ ] Add tab completion for gwtadd command
  - Show available remote branches for checkout
  - Auto-complete branch names

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