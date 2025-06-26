# Git Worktree CLI (gwt) TODO

## ✅ Completed

### Rust Implementation - Core Features
- ✅ **Converted from TypeScript to Rust** - Single binary, no Node.js dependency
- ✅ **`gwt init`** - Initialize worktrees from repository URLs with real-time git output
  - ✅ Clones repository with streaming progress
  - ✅ Detects default branch name
  - ✅ Creates git-worktree-config.yaml
  - ✅ Executes post-init hooks
- ✅ **`gwt list`** - Display worktrees in formatted table
  - ✅ Finds all worktrees in project
  - ✅ Sharp table formatting with Unicode borders
  - ✅ Shows path, branch, and HEAD commit
- ✅ **Configuration system** - YAML-based config with serde
- ✅ **Hooks system** - postInit, postAdd, postRemove with variable substitution
- ✅ **Test suite** - Integration and unit tests with assert_cmd

### Shell Completions
- ✅ **Built-in completion generation** - Generate for bash/zsh/fish
- ✅ **Auto-install completions** - `gwt completions install` command
- ✅ **Completion status check** - `gwt completions` shows install status
- ✅ **Smart shell detection** - Auto-detects user's shell
- ✅ **Branch name completion** - Tab completion for add/remove commands
- ✅ **Fixed completion parsing** - Correctly parses table output for branch names

### Code Quality
- ✅ **Removed all build warnings** - Clean compilation
- ✅ **Proper error handling** - Using anyhow for context
- ✅ **Real-time streaming output** - Native Rust process handling

### Legacy
- ✅ Original TypeScript version preserved in `typescript-version/` directory

## 🚧 In Progress / Partial Implementation

### Core Commands
- 🔄 **`gwt add`** - Create worktrees from branch names
  - ⚠️ Stub implementation only
  - [ ] Create worktree from existing branch
  - [ ] Create worktree with new branch
  - [ ] Handle remote branches
  - [ ] Execute post-add hooks
  - [ ] Auto-navigate to new worktree

- 🔄 **`gwt remove`** - Remove worktrees with safety checks
  - ⚠️ Stub implementation only
  - [ ] Remove worktree directory
  - [ ] Optional branch deletion
  - [ ] Safety checks for uncommitted changes
  - [ ] Execute post-remove hooks
  - [ ] Handle current directory removal

## 📋 Future Enhancements

### Shell Integration
- [ ] **Directory change on `gwt add`** - Auto-navigate to new worktree
  - [ ] Add `--print-path` flag for shell function integration
  - [ ] Create shell wrapper functions for auto-cd behavior
  - [ ] Update completion scripts to include wrapper

### Core Features
- [ ] **`gwt switch`** - Quick navigation between worktrees
- [ ] **PR Integration** - Show Bitbucket/GitHub PR status in list
- [ ] **Cleanup command** - Remove stale worktrees in bulk
- [ ] **Clone existing worktrees** - Support cloning projects with existing worktrees

### Quality of Life
- [ ] **Filtering** - Filter list by branch pattern, age, status
- [ ] **Metadata tracking** - Last commit date, creation time, PR links
- [ ] **Enhanced hooks** - Pre-hooks, conditional execution, error handling
- [ ] **Better error messages** - Actionable suggestions for common issues
- [ ] **Progress indicators** - For long-running operations
- [ ] **Colored output** - Better visual hierarchy in output
- [ ] **Config validation** - Validate hooks and settings

### Distribution
- [ ] **GitHub Releases** - Automated binary releases
- [ ] **Homebrew formula** - Easy macOS installation
- [ ] **AUR package** - Arch Linux support
- [ ] **Cargo crates.io** - Publish to Rust package registry

## 🐛 Known Issues

- [ ] Completion parsing assumes specific table format
- [ ] No Windows support tested
- [ ] Limited error recovery in hook execution

## 📝 Notes

The Rust rewrite has successfully achieved:
1. Better performance with compiled binary
2. Real-time streaming output for git commands
3. Enhanced completion system with auto-install
4. Clean, warning-free codebase
5. Professional table output

Priority should be on completing the `gwt add` and `gwt remove` commands to achieve feature parity with the TypeScript version.
