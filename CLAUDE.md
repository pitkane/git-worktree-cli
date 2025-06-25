# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- **Build release binary**: `cargo build --release` - Creates optimized binary at `target/release/gwt`
- **Build debug binary**: `cargo build` - Creates debug binary for development
- **Run tests**: `cargo test` - Comprehensive unit and integration tests
- **Type checking**: `cargo check` - Fast compilation check without building binary
- **Run with cargo**: `cargo run -- <command>` - Run directly with cargo for development

### Git Worktree Tool (gwt)
- **Initialize project**: `gwt init <repository-url>` - Initialize a new worktree project from a repository URL
- **Add worktree**: `gwt add <branch-name>` - Create new worktrees from branch names  
- **List worktrees**: `gwt list` - List all worktrees in a formatted table
- **Switch worktree**: `gwt switch <branch-name>` - Switch between existing worktrees
- **Remove worktree**: `gwt remove [branch-name]` - Remove existing worktrees with confirmation
- **Generate completions**: `gwt completions <shell>` - Generate shell completion scripts

### Legacy TypeScript Version
The original TypeScript implementation has been moved to `typescript-version/` directory for reference.

## Architecture

This project is a **single Rust binary** that provides git worktree management functionality. The architecture consists of:

1. **Rust Binary** (`src/`): Core functionality written in Rust
   - `main.rs`: CLI entry point using clap for argument parsing
   - `commands/`: Individual command implementations (init, add, list, switch, remove)
   - `config.rs`: YAML configuration file handling using serde
   - `git.rs`: Git operations with native process execution and streaming output
   - `hooks.rs`: Hook execution system with real-time output streaming
   - `utils.rs`: Shared utility functions

2. **Key Dependencies**:
   - `clap`: Command-line argument parsing with derive macros
   - `serde` + `serde_yaml`: Configuration file serialization/deserialization
   - `anyhow`: Error handling and context
   - `colored`: Terminal output colorization
   - `chrono`: Date/time handling for config timestamps

3. **Test Infrastructure**:
   - `tests/`: Integration tests using `assert_cmd` and `tempfile`
   - Unit tests embedded in source modules
   - Real git repository testing with streaming output verification

## Hooks System

The project includes a flexible hooks system that allows users to run custom commands after various worktree operations. Hooks are defined in the `git-worktree-config.yaml` file and support variable substitution.

### Available Hooks

- **`postInit`**: Executed after `gwtinit` creates a new project
- **`postAdd`**: Executed after `gwtadd` creates a new worktree
- **`postSwitch`**: Executed after `gwtswitch` switches to a worktree
- **`postRemove`**: Executed after `gwtremove` removes a worktree

### Variable Substitution

Hooks support the following variables:
- `${branchName}`: The name of the branch
- `${worktreePath}`: The full path to the worktree directory

### Default Configuration

When `gwtinit` creates a new project, it generates a `git-worktree-config.yaml` with all hooks **commented out by default**:

```yaml
hooks:
  postAdd:
    - "# npm install"
  postSwitch:
    - "# echo 'Switched to branch ${branchName}'"
  postRemove:
    - "# echo 'Removed worktree for branch ${branchName}'"
  postInit:
    - "# echo 'Initialized git worktree project'"
```

### Active Configuration Example

To enable hooks, simply remove the `#` comments:

```yaml
hooks:
  postAdd:
    - "echo 'Created worktree for ${branchName} at ${worktreePath}'"
    - "npm install"
    - "npm run init"
  postSwitch:
    - "echo 'Switched to branch ${branchName}'"
  postRemove:
    - "echo 'Removed worktree for branch ${branchName}'"
  postInit:
    - "echo 'Initialized git worktree project'"
```

### Hook Behavior

- **Real-time output**: Commands stream output live using `execSync` with `stdio: 'inherit'`
- **Execution context**: 
  - `postAdd`/`postSwitch`: Execute in the worktree directory
  - `postRemove`/`postInit`: Execute in the project root directory
- **Comment handling**: Lines starting with `#` are automatically skipped
- **Error handling**: Failed hooks show warnings but don't stop execution
- **Sequential execution**: Hooks run in the order they're defined
- **Environment**: Hooks inherit the current environment with `FORCE_COLOR: '1'` for colored output

### Common Hook Patterns

```yaml
hooks:
  postAdd:
    - "npm install"                    # Install dependencies
    - "npm run build"                  # Build project
    - "code ."                         # Open in VS Code
  postSwitch:
    - "git status"                     # Show current status
  postRemove:
    - "echo 'Cleaned up ${branchName}'" # Log cleanup
```

### Troubleshooting Hooks

- **Hook not executing**: Check if the line starts with `#` (commented out)
- **No output visible**: Hooks use real-time streaming - output should appear immediately
- **Hook fails**: Check the command syntax and file permissions
- **Variable not substituted**: Ensure correct syntax: `${branchName}` or `${worktreePath}`

## Current Implementation Status

### ✅ Implemented Features (Rust)

1. **`gwt init`**: Initialize worktrees from repository URLs ✅
   - ✅ Clones repository with **real-time streaming output** (major improvement!)
   - ✅ Detects the default branch name
   - ✅ Renames the cloned directory to match the branch name
   - ✅ Creates `git-worktree-config.yaml` with repository metadata
   - ✅ Executes post-init hooks with streaming output

### 🔄 Partially Implemented Features

2. **`gwt add`**: Create new worktrees from branch names 🔄
   - ⚠️ Stub implementation only - needs full functionality

3. **`gwt list`**: Display all worktrees in a formatted table 🔄
   - ⚠️ Stub implementation only - needs full functionality

4. **`gwt switch`**: Switch between existing worktrees 🔄
   - ⚠️ Stub implementation only - needs full functionality

5. **`gwt remove`**: Remove worktrees with safety checks 🔄
   - ⚠️ Stub implementation only - needs full functionality

### 🎯 Major Improvements Achieved

- **✅ Real-time streaming output**: Git commands show progress in real-time using Rust's native `Command` with `Stdio::inherit()`
- **✅ Single binary distribution**: No Node.js runtime or shell wrapper functions needed
- **✅ Built-in shell completions**: Generate completion scripts for bash/zsh/fish with `gwt completions`
- **✅ Better error handling**: Rust's `Result` type provides robust error propagation
- **✅ Faster execution**: Compiled binary vs interpreted TypeScript
- **✅ Cross-platform compatibility**: Easy to build for different OS/architectures

## Test Suite

The project includes comprehensive testing in Rust:
- **Integration tests** (`tests/`): Uses `assert_cmd` and `tempfile` for real command testing
- **Unit tests**: Embedded in source modules with `#[cfg(test)]`
- **Real streaming verification**: Tests actually verify git clone output streaming
- **6 integration tests**: Covering init command, help, version, error handling
- **4 unit tests**: Testing config module functionality
- **Fast execution**: All tests run in ~6 seconds vs 15+ seconds for TypeScript version

## TODO Tracking

Project TODOs are maintained in `TODO.md` for persistence across Claude Code sessions. This includes:
- Completed tasks (extensive list of implemented features)
- Pending features (like Bitbucket PR integration, gwtadd tab completion)
- Future enhancement ideas (filtering, colors, metadata tracking)

## Code Style

- **Language**: Rust 2021 edition with standard formatting
- **Error Handling**: `anyhow::Result` for error propagation with context
- **CLI Framework**: `clap` with derive macros for argument parsing
- **Serialization**: `serde` with `serde_yaml` for configuration files
- **Testing**: Integration tests with `assert_cmd`, unit tests with `#[cfg(test)]`
- **File Organization**: Each command has its own module, shared utilities in dedicated modules

## Important Technical Notes

### Real-time Streaming Output with Rust

The Rust implementation naturally handles streaming output using native `std::process::Command`:

```rust
// ✅ Real-time streaming with Rust - simple and effective!
use std::process::{Command, Stdio};

Command::new("git")
    .args(["clone", repo_url, repo_name])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()?;
```

**Key advantages over the TypeScript/zx approach:**
- **No buffering issues**: Output streams directly to terminal
- **Native support**: No workarounds needed for real-time output
- **Better performance**: Direct process spawning without Node.js overhead
- **Cross-platform**: Works consistently across different operating systems

### Hook Execution with Streaming

Hooks also benefit from native process execution:

```rust
// Hooks execute with real-time output
Command::new("sh")
    .arg("-c")
    .arg(command)
    .current_dir(working_directory)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .env("FORCE_COLOR", "1")
    .status()?;
```

This eliminates the complex issues that existed with the TypeScript implementation where streaming output required workarounds with `execSync` and `stdio: 'inherit'`.