# Git Worktree Manager (gwt) TODO

## ✅ Completed (Rust Conversion)

### Major Architecture Change
- [x] **Converted entire project from TypeScript to Rust** 🎉
  - Single binary distribution (no Node.js dependency)
  - Real-time streaming output for git commands (major improvement!)
  - Better error handling with `anyhow::Result`
  - Cross-platform compatibility
  - Faster execution (compiled vs interpreted)

### Implemented Features (Rust)
- [x] **`gwt init` command** - Initialize worktrees from repository URLs ✅
  - ✅ Real-time streaming git clone output (solves TypeScript buffering issues!)
  - ✅ Detects default branch name
  - ✅ Creates `git-worktree-config.yaml` with repository metadata
  - ✅ Executes post-init hooks with streaming output
  - ✅ Directory cleanup and proper error handling

- [x] **Configuration system** - YAML-based config management ✅
  - ✅ Serde-based serialization/deserialization
  - ✅ Config file discovery up directory tree
  - ✅ Timestamp tracking with chrono

- [x] **Hooks system** - Real-time streaming hook execution ✅
  - ✅ Four hook types: postInit, postAdd, postSwitch, postRemove
  - ✅ Variable substitution for ${branchName} and ${worktreePath}
  - ✅ Real-time output streaming using Rust's native Command
  - ✅ Comment-based configuration (hooks disabled by default)
  - ✅ Proper error handling with warnings

- [x] **CLI framework** - clap-based argument parsing ✅
  - ✅ Built-in help and version commands
  - ✅ Shell completion generation (bash, zsh, fish)
  - ✅ Subcommand structure with proper argument validation

- [x] **Test suite** - Comprehensive Rust testing ✅
  - ✅ 6 integration tests using assert_cmd and tempfile
  - ✅ 4 unit tests for config module
  - ✅ Real streaming output verification
  - ✅ Faster execution (~6s vs ~15s for TypeScript)

### Legacy TypeScript Implementation
- [x] Moved TypeScript version to `typescript-version/` directory
- [x] All original features preserved for reference:
  - gwtinit, gwtadd, gwtswitch, gwtlist, gwtremove commands
  - Tab completion system
  - Hooks with streaming output (via execSync workarounds)
  - 15 integration tests
  - Context-aware execution (hooks run in appropriate directories)
  - Configuration auto-generation with helpful examples in git-worktree-config.yaml

## 🔄 In Progress (Rust Implementation)

### Core Commands (Need Full Implementation)
- [x] **`gwt add` command** - Create new worktrees from branch names ✅
  - ✅ Support folder names with slashes (e.g., feature/IP-487)
  - ✅ Auto-branch from main/default branch
  - ✅ Detect existing branches and check them out
  - ✅ Execute post-add hooks with streaming output
  - ✅ Smart path detection to find project root

- [x] **`gwt list` command** - Display worktrees in formatted table ✅
  - ✅ Show path and branch for each worktree
  - ✅ Work from both project root and worktree directories
  - ✅ Clean branch name display (removes refs/heads/ prefix)
  - ✅ Properly aligned columnar output using tabled crate
  - ✅ Handle bare repositories gracefully

- [x] **`gwt switch` command** - Switch between existing worktrees ✅
  - ✅ Quick navigation between worktree directories
  - ✅ Show available worktrees when no branch specified
  - ✅ Helpful error messages and suggestions
  - ✅ Execute post-switch hooks
  - ✅ Directory change output for shell integration

- [x] **`gwt remove` command** - Remove worktrees with safety checks ✅
  - ✅ Remove current worktree when no parameter given
  - ✅ Remove specified worktree when parameter provided
  - ✅ Safety checks to prevent removing main/bare repository
  - ✅ Interactive confirmation prompts
  - ✅ Auto-navigate to project root when removing current worktree
  - ✅ Remove associated git branches (except main branches)
  - ✅ Execute post-remove hooks
  - ✅ Find worktrees by branch name or path

### Shell Integration
- [ ] **Tab completion** for all commands
  - [ ] `gwt add`: Show available remote branches
  - [ ] `gwt switch`: List available worktree branches
  - [ ] `gwt remove`: List removable worktree branches
  - [ ] Integrate with built-in clap completion system

## 📋 Future Enhancements

### Core Features
- [ ] Add Bitbucket/GitHub PR integration
  - Search for open PRs associated with each worktree branch
  - Display PR status inline with worktree list
  - Add ability to open PR in browser
  - Use repository URL from git-worktree-config.yaml

### Quality of Life
- [ ] Add filtering options to `gwt list` (by branch pattern, age, etc.)
- [ ] Show last commit date/author for each worktree
- [ ] Add cleanup command for stale worktrees
- [ ] Enhanced configuration options:
  - Store worktree metadata (creation date, last accessed, etc.)
  - Track branch relationships
  - Store project-specific settings

### Performance & Polish
- [ ] Add colored output (already using `colored` crate)
- [ ] Progress bars for long operations
- [ ] Better error messages with suggestions
- [ ] Parallel operations where possible