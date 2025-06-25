# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- **Run TypeScript files**: `pnpm tsx ./src/<filename>.ts`
- **Type checking**: `pnpm typecheck`
- **Linting**: `pnpm lint`
- **Fix linting issues**: `pnpm lint:fix`
- **Run tests**: `pnpm test`

### Initialize Script
- **Install bash functions**: `pnpm initialize` - Adds git worktree helper functions to ~/.zshrc

### Git Worktree Scripts
- **git-worktree-init**: `pnpm script:git-worktree-init` - Initialize a new worktree from a repository URL
- **git-worktree-add**: `pnpm script:git-worktree-add` (not implemented)
- **git-worktree-remove**: `pnpm script:git-worktree-remove` (not implemented)
- **git-worktree-completion**: `pnpm script:git-worktree-completion` (not implemented)
- **git-worktree-list**: `pnpm script:git-worktree-list` - List all worktrees in a formatted table

## Architecture

This project provides shell functions for managing git worktrees more efficiently. The architecture consists of:

1. **TypeScript Scripts** (`src/*.ts`): Core functionality written in TypeScript using the `zx` library for shell scripting
   - Entry points execute via `tsx` (TypeScript execute)
   - Scripts are meant to be called from shell functions

2. **Shell Functions** (`src/bash-functionality.sh`): Bash/Zsh functions that wrap the TypeScript scripts
   - Currently implements `gwtinit` function
   - Functions are installed to `~/.zshrc` via the initialize script

3. **Key Dependencies**:
   - `zx`: Provides a better shell scripting experience in JavaScript/TypeScript
   - `tsx`: Direct TypeScript execution without compilation
   - `@biomejs/biome`: Code formatting and linting (replaces ESLint/Prettier)

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
   - Relative paths from current directory
   - Indicates bare repositories

## TODO Tracking

Project TODOs are maintained in `TODO.md` for persistence across Claude Code sessions. This includes:
- Completed tasks
- Pending features (like Bitbucket PR integration)
- Future enhancement ideas

## Code Style

- **Formatting**: Biome with tabs for indentation and double quotes
- **TypeScript**: Strict mode enabled, ES2022 target
- **Module System**: ES modules (`"type": "module"` in package.json)