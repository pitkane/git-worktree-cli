# Git Worktree Manager (gwt)

🌿 **Enhanced Git Worktree Management with Rust** 🌿

Stop juggling multiple git clones or constantly switching branches. Git worktrees let you have multiple working directories from the same repository, each checked out to different branches. This Rust-powered tool makes managing them effortless with **real-time streaming output**.

## What are Git Worktrees?

Instead of this messy workflow:
```bash
# The old way - multiple clones or constant switching
git clone repo.git feature-work
git clone repo.git bugfix-work
# OR constantly: git checkout feature && git checkout main && git checkout bugfix...
```

Git worktrees let you do this:
```bash
# One repository, multiple working directories
my-project/
├── main/           # Main branch
├── feature-123/    # Feature branch  
└── bugfix-456/     # Bugfix branch
```

Each directory is a separate working tree of the same repository. No more stashing, switching, or losing context.

## Installation

### Option 1: Build from Source (Recommended)

1. **Clone this repository:**
   ```bash
   git clone git@github.com:pitkane/git-worktree-scripts.git ~/.git-worktree-scripts
   cd ~/.git-worktree-scripts
   ```

2. **Build the binary:**
   ```bash
   cargo build --release
   ```

3. **Install the binary:**
   ```bash
   # Copy to your PATH
   sudo cp target/release/gwt /usr/local/bin/
   # Or add to your shell config
   echo 'export PATH="$HOME/.git-worktree-scripts/target/release:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

4. **Generate shell completions (optional):**
   ```bash
   # For zsh
   gwt completions zsh > ~/.zsh/completions/_gwt
   # For bash  
   gwt completions bash > ~/.bash_completion.d/gwt
   ```

### Option 2: Use Legacy TypeScript Version

The original TypeScript implementation is available in the `typescript-version/` directory:
```bash
cd typescript-version
pnpm install
pnpm initialize
source ~/.zshrc
```

## Quick Start Workflow

### 1. Initialize a Project
```bash
# Clone and setup worktree structure with REAL-TIME OUTPUT!
gwt init git@github.com:username/repo.git

# This creates:
# - main/ directory (default branch)
# - git-worktree-config.yaml (project metadata)
# You'll see git clone progress in real-time!
```

### 2. Create Feature Branches
```bash
# Create new feature worktree
gwt add feature/user-auth
# Creates feature/user-auth/ directory and automatically switches to it

# Create bugfix worktree  
gwt add bugfix/login-error
```

### 3. Switch Between Work
```bash
# Quick switch between worktrees
gwt switch main
gwt switch feature/user-auth
gwt switch bugfix/login-error

# See all your worktrees
gwt list
```

### 4. Clean Up When Done
```bash
# Remove completed feature
gwt remove feature/user-auth  # Removes worktree and branch

# Or remove current worktree
gwt remove  # Removes current worktree you're in
```

## Real-World Example

```bash
# Start new project with streaming git clone output
gwt init git@github.com:company/web-app.git
cd main

# Work on main features
gwt add feature/shopping-cart
# Now in feature/shopping-cart/ directory
# Make commits, push changes...

# Urgent bugfix needed - no stashing required!
gwt add hotfix/payment-bug
# Now in hotfix/payment-bug/ directory  
# Fix bug, commit, push...

# Back to feature work
gwt switch feature/shopping-cart
# Continue where you left off

# Review all work
gwt list
# Shows:
# PATH                          BRANCH
# /path/to/main                 main
# /path/to/feature/shopping-cart feature/shopping-cart  
# /path/to/hotfix/payment-bug   hotfix/payment-bug

# Clean up merged work
gwt remove hotfix/payment-bug
```

## Commands Reference

| Command | Description | Example | Status |
|---------|-------------|---------|---------|
| `gwt init <url>` | Initialize worktree project from repo | `gwt init git@github.com:user/repo.git` | ✅ **Working** |
| `gwt list` | List all worktrees | `gwt list` | ✅ **Working** |
| `gwt add <branch>` | Create new worktree/branch | `gwt add feature/new-ui` | ✅ **Working** |
| `gwt switch <branch>` | Switch to existing worktree | `gwt switch main` | ✅ **Working** |
| `gwt remove [branch]` | Remove worktree (current if no args) | `gwt remove old-feature` | ✅ **Working** |
| `gwt completions <shell>` | Generate shell completions | `gwt completions zsh` | ✅ **Working** |

**New in Rust version:**
- ✅ **Real-time streaming output** - See git clone progress live!
- ✅ **Single binary** - No Node.js dependency
- ✅ **Built-in completions** - Generate for bash, zsh, fish
- ✅ **Better performance** - Compiled Rust vs interpreted TypeScript

## Hooks & Automation

Git worktree scripts support **hooks** - custom commands that run automatically after worktree operations. Perfect for automating setup tasks like installing dependencies or running initialization scripts.

### Quick Setup
```bash
# After gwtinit, edit git-worktree-config.yaml
hooks:
  postAdd:
    - "npm install"      # Auto-install deps in new worktrees
    - "npm run init"     # Run your custom setup script
```

### Example Workflow with Hooks
```bash
# Initialize project (creates config with hook examples)
gwtinit git@github.com:company/web-app.git

# Edit git-worktree-config.yaml to enable hooks:
# hooks:
#   postAdd:
#     - "npm install"    # Remove # to enable

# Create new worktree - hooks run automatically!
gwtadd feature/shopping-cart
# This will:
# 1. Create the worktree  
# 2. Run "npm install" automatically
# 3. Show real-time output from npm
# 4. Switch to the new directory

# Continue with your work - dependencies already installed!
```

### Available Hook Types
- **`postInit`**: After creating a new project
- **`postAdd`**: After creating a new worktree (perfect for setup)
- **`postSwitch`**: After switching to a worktree  
- **`postRemove`**: After removing a worktree (great for cleanup)

### Variable Support
Use `${branchName}` and `${worktreePath}` in your hooks:
```yaml
hooks:
  postAdd:
    - "echo 'Created ${branchName} at ${worktreePath}'"
    - "npm install"
```

By default, all hooks are commented out (disabled) - uncomment the ones you want to use.

## Benefits

- **🚀 No Context Switching**: Each branch keeps its own working directory
- **🔄 Instant Branch Switching**: No checkout delays or merge conflicts  
- **🛡️ Safe Experimentation**: Isolated working directories prevent conflicts
- **⚡ Parallel Development**: Work on multiple features simultaneously
- **🧹 Easy Cleanup**: Remove completed work with one command
- **🪝 Smart Automation**: Hooks automatically run setup/cleanup tasks
- **📊 Real-time Feedback**: See command output as it executes

## Requirements

- Node.js 18+ and pnpm
- Git 2.5+ (for worktree support)
- Bash/Zsh shell

## Development

```bash
# Run tests
pnpm test

# Type checking
pnpm typecheck

# Linting
pnpm lint
```
