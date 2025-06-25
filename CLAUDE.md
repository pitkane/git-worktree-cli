# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- **Run TypeScript files**: `pnpm tsx ./src/<filename>.ts`
- **Type checking**: `pnpm typecheck`
- **Linting**: `pnpm lint`
- **Fix linting issues**: `pnpm lint:fix`
- **Run tests**: `pnpm test` - Comprehensive integration tests for all worktree commands

### Initialize Script
- **Install bash functions**: `pnpm initialize` - Adds git worktree helper functions to ~/.zshrc

### Git Worktree Scripts  
- **git-worktree-init**: `pnpm script:git-worktree-init` - Initialize a new worktree from a repository URL
- **git-worktree-add**: `pnpm script:git-worktree-add` - Create new worktrees from branch names
- **git-worktree-switch**: `pnpm script:git-worktree-switch` - Switch between existing worktrees
- **git-worktree-list**: `pnpm script:git-worktree-list` - List all worktrees in a formatted table
- **git-worktree-completion**: `pnpm script:git-worktree-completion` - Tab completion for worktree commands
- **git-worktree-remove**: `pnpm script:git-worktree-remove` - Remove existing worktrees with confirmation

## Architecture

This project provides shell functions for managing git worktrees more efficiently. The architecture consists of:

1. **TypeScript Scripts** (`src/*.ts`): Core functionality written in TypeScript using the `zx` library for shell scripting
   - Entry points execute via `tsx` (TypeScript execute)
   - Scripts are meant to be called from shell functions
   - Each script handles a specific worktree operation

2. **Shell Functions** (`src/bash-functionality.sh`): Bash/Zsh functions that wrap the TypeScript scripts
   - Implements `gwtinit`, `gwtlist`, `gwtadd`, `gwtswitch`, and `gwtremove` functions
   - Includes tab completion for `gwtswitch`, `gwtadd`, and `gwtremove` commands
   - Functions are installed to `~/.zshrc` via the initialize script
   - Handles directory navigation and interactive prompts

3. **Key Dependencies**:
   - `zx`: Provides a better shell scripting experience in JavaScript/TypeScript
   - `tsx`: Direct TypeScript execution without compilation
   - `@biomejs/biome`: Code formatting and linting (replaces ESLint/Prettier)
   - `yaml`: For parsing and generating git-worktree-config.yaml files

## Current Implementation

### Implemented Features

1. **`gwtinit`**: Initialize worktrees from repository URLs
   - Clones a repository from a given URL
   - Detects the default branch name
   - Renames the cloned directory to match the branch name
   - Creates `git-worktree-config.yaml` with repository metadata
   - This helps organize worktrees by branch name rather than repository name

2. **`gwtlist`**: Display all worktrees in a formatted table
   - Shows path and branch for each worktree
   - Works from both project root and worktree directories
   - Indicates bare repositories
   - Properly aligned columnar output

3. **`gwtadd`**: Create new worktrees from branch names
   - Takes folder name as parameter (supports slashes like `feature/IP-487`)
   - Automatically branches from main/default branch
   - Detects existing branches and checks them out instead of creating duplicates
   - Works from both project root and worktree directories
   - Automatically navigates to new worktree after creation
   - Supports nested directory structures

4. **`gwtswitch`**: Switch between existing worktrees
   - Quick navigation between worktree directories
   - Shows available worktrees when no branch specified
   - Tab completion for available worktree branches
   - Helpful error messages and suggestions
   - Works from both project root and worktree directories

5. **`gwtremove`**: Remove worktrees with safety checks
   - Remove current worktree when no parameter given
   - Remove specified worktree when parameter provided
   - Safety checks to prevent removing main/bare repository
   - Interactive confirmation prompts
   - Automatic navigation to project root when removing current worktree
   - Also removes associated git branches (except main branches)
   - Tab completion for available worktree branches

6. **Tab Completion**: Comprehensive completion system
   - `gwtswitch`: Lists available worktree branches
   - `gwtremove`: Lists removable worktree branches
   - `gwtadd`: Placeholder for future remote branch completion
   - Filters completions based on current input
   - Shows helpful messages when no completions available

## Test Suite

The project includes comprehensive integration tests in `src/test.ts`:
- Tests all major worktree operations end-to-end
- Tests both bash function wrappers and direct TypeScript calls
- Tests error handling for invalid inputs
- Tests tab completion functionality
- Tests directory navigation and cleanup
- Includes 15 different test scenarios

## TODO Tracking

Project TODOs are maintained in `TODO.md` for persistence across Claude Code sessions. This includes:
- Completed tasks (extensive list of implemented features)
- Pending features (like Bitbucket PR integration, gwtadd tab completion)
- Future enhancement ideas (filtering, colors, metadata tracking)

## Code Style

- **Formatting**: Biome with tabs for indentation and double quotes (120 char line width)
- **TypeScript**: Strict mode enabled, ES2022 target
- **Module System**: ES modules (`"type": "module"` in package.json)
- **Linting**: Biome with recommended rules enabled
- **File Organization**: Each command has its own TypeScript file, shared utilities in common functions