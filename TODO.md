# Git Worktree CLI (gwt) TODO

This file tracks pending tasks and future enhancements for the project.

## 🎯 High Priority

### Distribution & Release
- [ ] **GitHub Releases** - Automated binary releases with CI/CD
- [ ] **Install script** - curl-able install.sh for easy setup like other Rust tools
- [ ] **Homebrew formula** - Easy macOS installation with auto-completions
- [ ] **Cargo crates.io** - Publish to Rust package registry

## 🔧 Medium Priority  

### Core Features
- [ ] **`gwt switch`** - Quick navigation between worktrees
- [ ] **`gwt doctor`** - Health check command to diagnose issues
- [ ] **Cleanup command** - Remove stale worktrees in bulk
- [ ] **Clone existing worktrees** - Support cloning projects with existing worktrees

### Shell Integration
- [ ] **Directory change on `gwt add`** - Auto-navigate to new worktree
  - [ ] Add `--print-path` flag for shell function integration
  - [ ] Create shell wrapper functions for auto-cd behavior
  - [ ] Update completion scripts to include wrapper

### Quality of Life
- [ ] **Command flags** - Add useful flags to existing commands:
  - [ ] `gwt add --from <branch>` - Create from specific branch instead of main
  - [ ] `gwt add --no-hooks` - Skip hook execution
  - [ ] `gwt remove --force` - Skip confirmation prompts
  - [ ] `gwt remove --keep-branch` - Preserve branch even for feature branches
- [ ] **Better error messages** - Actionable suggestions for common issues
- [ ] **Progress indicators** - For long-running operations
- [ ] **Colored output** - Better visual hierarchy in output
- [ ] **Config validation** - Validate hooks and settings on startup

## 🚀 Low Priority

### Advanced Features
- [ ] **PR Integration** - Show Bitbucket/GitHub PR status in list
- [ ] **Filtering** - Filter list by branch pattern, age, status
- [ ] **Metadata tracking** - Last commit date, creation time, PR links
- [ ] **Enhanced hooks** - Pre-hooks, conditional execution, error handling
- [ ] **Bulk operations** - Remove multiple worktrees with pattern matching

### Platform Support
- [ ] **AUR package** - Arch Linux support
- [ ] **Windows testing** - Verify Windows compatibility
- [ ] **Package managers** - Support for more Linux package managers

## 🐛 Known Issues

- [ ] Completion parsing assumes specific table format
- [ ] No Windows support tested
- [ ] Limited error recovery in hook execution
- [ ] Build warnings from completion generation (cosmetic)

## 💡 Ideas for Future Consideration

- [ ] **Integration with IDEs** - VS Code extension for worktree management
- [ ] **Git hooks integration** - Automatic setup of git hooks in worktrees
- [ ] **Template system** - Predefined project templates for different types
- [ ] **Remote worktrees** - Support for remote filesystem worktrees
- [ ] **Backup/restore** - Export/import worktree configurations

---

    ╭──────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
     │ Research Summary: Shell Integration for gwt add Auto-Navigation                                                  │
     │                                                                                                                  │
     │ Based on my research into how other Rust CLI tools handle automatic directory navigation, I found several        │
     │ excellent patterns we can apply to gwt add:                                                                      │
     │                                                                                                                  │
     │ Key Findings:                                                                                                    │
     │                                                                                                                  │
     │ 1. Fundamental Challenge: Rust CLI programs cannot change their parent shell's working directory directly - only │
     │ their own process directory.                                                                                     │
     │ 2. Proven Solution Pattern: Use a combination of:                                                                │
     │   - CLI flag that outputs a path instead of performing the action                                                │
     │   - Shell wrapper function that captures the output and performs cd                                              │
     │                                                                                                                  │
     │ Successful Examples:                                                                                             │
     │                                                                                                                  │
     │ Tere - Terminal file explorer:                                                                                   │
     │ tere() {                                                                                                         │
     │     local result=$(command tere "$@")                                                                            │
     │     [ -n "$result" ] && cd -- "$result"                                                                          │
     │ }                                                                                                                │
     │                                                                                                                  │
     │ Zoxide - Smart cd replacement:                                                                                   │
     │ - Uses zoxide init to generate shell-specific integration code                                                   │
     │ - Creates wrapper functions like z and zi                                                                        │
     │ - Supports all major shells (bash, zsh, fish, PowerShell)                                                        │
     │                                                                                                                  │
     │ Implementation Plan for gwt add:                                                                                 │
     │                                                                                                                  │
     │ Phase 1: Add --print-path Flag                                                                                   │
     │                                                                                                                  │
     │ - Add --print-path flag to gwt add command                                                                       │
     │ - When flag is used, print the created worktree path to stdout instead of showing success messages               │
     │ - Modify our existing completions to include this flag                                                           │
     │                                                                                                                  │
     │ Phase 2: Create Shell Wrapper Functions                                                                          │
     │                                                                                                                  │
     │ - Create shell functions for bash, zsh, fish                                                                     │
     │ - Functions capture output from gwt add --print-path and cd to the result                                        │
     │ - Include these in our completion installation system                                                            │
     │                                                                                                                  │
     │ Phase 3: Enhanced Integration                                                                                    │
     │                                                                                                                  │
     │ - Create gwt shell-init command (like zoxide) that generates shell-specific code                                 │
     │ - Auto-detect shell and provide appropriate wrapper functions                                                    │
     │ - Include in our existing completion system                                                                      │
     │                                                                                                                  │
     │ Example Implementation:                                                                                          │
     │                                                                                                                  │
     │ Rust Side:                                                                                                       │
     │ #[arg(long, help = "Print the worktree path instead of changing to it")]                                         │
     │ print_path: bool,                                                                                                │
     │                                                                                                                  │
     │ Shell Wrapper:                                                                                                   │
     │ gwt_add() {                                                                                                      │
     │     if [[ "$*" == *"--print-path"* ]]; then                                                                      │
     │         command gwt add "$@"                                                                                     │
     │     else                                                                                                         │
     │         local result=$(command gwt add --print-path "$@")                                                        │
     │         if [ -n "$result" ] && [ -d "$result" ]; then                                                            │
     │             cd -- "$result"                                                                                      │
     │             echo "✓ Switched to worktree: $result"                                                               │
     │         fi                                                                                                       │
     │     fi                                                                                                           │
     │ }                                                                                                                │
     │                                                                                                                  │
     │ Benefits:                                                                                                        │
     │                                                                                                                  │
     │ - Leverages our existing completion system infrastructure                                                        │
     │ - Follows proven patterns from successful Rust CLI tools                                                         │
     │ - Maintains backward compatibility                                                                               │
     │ - Can be optionally installed with completions                                                                   │
     │                                                                                                                  │
     │ Would you like me to implement this shell integration system for gwt add?
