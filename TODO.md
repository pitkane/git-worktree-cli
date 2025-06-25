# Git Worktree Manager (gwt) TODO

## ✅ Completed

### Rust Implementation
- ✅ **Converted from TypeScript to Rust** - Single binary, no Node.js dependency
- ✅ **`gwt init`** - Initialize worktrees from repository URLs with real-time git output
- ✅ **`gwt add`** - Create worktrees with branch detection and smart path handling
- ✅ **`gwt list`** - Display worktrees in formatted table with tabled crate
- ✅ **`gwt remove`** - Remove worktrees with safety checks and confirmations
- ✅ **Configuration system** - YAML-based config with serde
- ✅ **Hooks system** - postInit, postAdd, postRemove with variable substitution
- ✅ **Shell completions** - Built-in bash/zsh/fish completion generation
- ✅ **Test suite** - Integration and unit tests with assert_cmd

### Legacy TypeScript
- ✅ Original TypeScript version preserved in `typescript-version/` directory

## 🚧 In Progress

### Shell Integration
- [ ] **Directory change on `gwt add`** - Auto-navigate to new worktree
  - [ ] Add `--print-path` flag for shell function integration
  - [ ] Create shell wrapper functions for auto-cd behavior
  - [ ] Update completion scripts to include wrapper

## 📋 Future Enhancements

### Core Features
- [ ] **PR Integration** - Show Bitbucket/GitHub PR status in list
- [ ] **Tab completion improvements** - Context-aware branch suggestions
- [ ] **Cleanup command** - Remove stale worktrees in bulk

### Quality of Life
- [ ] **Filtering** - Filter list by branch pattern, age, etc.
- [ ] **Metadata tracking** - Last commit date, creation time, etc.
- [ ] **Enhanced hooks** - Pre-hooks, conditional execution
- [ ] **Better error messages** - Actionable suggestions for common issues