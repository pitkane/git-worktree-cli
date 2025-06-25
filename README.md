# Git Worktree Scripts

🌿 **Enhanced Git Worktree Management** 🌿

Stop juggling multiple git clones or constantly switching branches. Git worktrees let you have multiple working directories from the same repository, each checked out to different branches. This tool makes managing them effortless.

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

1. **Clone this repository:**
   ```bash
   git clone git@github.com:pitkane/git-worktree-scripts.git ~/.git-worktree-scripts
   cd ~/.git-worktree-scripts
   ```

2. **Install dependencies:**
   ```bash
   pnpm install
   ```

3. **Add functions to your shell:**
   ```bash
   pnpm initialize
   source ~/.zshrc  # or restart your terminal
   ```

## Quick Start Workflow

### 1. Initialize a Project
```bash
# Clone and setup worktree structure
gwtinit git@github.com:username/repo.git

# This creates:
# - main/ directory (default branch)
# - git-worktree-config.yaml (project metadata)
```

### 2. Create Feature Branches
```bash
# Create new feature worktree
gwtadd feature/user-auth
# Creates feature/user-auth/ directory and automatically switches to it

# Create bugfix worktree  
gwtadd bugfix/login-error
```

### 3. Switch Between Work
```bash
# Quick switch between worktrees
gwtswitch main
gwtswitch feature/user-auth
gwtswitch bugfix/login-error

# See all your worktrees
gwtlist
```

### 4. Clean Up When Done
```bash
# Remove completed feature
gwtremove feature/user-auth  # Removes worktree and branch

# Or remove current worktree
gwtremove  # Removes current worktree you're in
```

## Real-World Example

```bash
# Start new project
gwtinit git@github.com:company/web-app.git
cd main

# Work on main features
gwtadd feature/shopping-cart
# Now in feature/shopping-cart/ directory
# Make commits, push changes...

# Urgent bugfix needed - no stashing required!
gwtadd hotfix/payment-bug
# Now in hotfix/payment-bug/ directory  
# Fix bug, commit, push...

# Back to feature work
gwtswitch feature/shopping-cart
# Continue where you left off

# Review all work
gwtlist
# Shows:
# PATH                          BRANCH
# /path/to/main                 main
# /path/to/feature/shopping-cart feature/shopping-cart  
# /path/to/hotfix/payment-bug   hotfix/payment-bug

# Clean up merged work
gwtremove hotfix/payment-bug
```

## Commands Reference

| Command | Description | Example |
|---------|-------------|---------|
| `gwtinit <url>` | Initialize worktree project from repo | `gwtinit git@github.com:user/repo.git` |
| `gwtlist` | List all worktrees | `gwtlist` |
| `gwtadd <branch>` | Create new worktree/branch | `gwtadd feature/new-ui` |
| `gwtswitch <branch>` | Switch to existing worktree | `gwtswitch main` |
| `gwtremove [branch]` | Remove worktree (current if no args) | `gwtremove old-feature` |

All commands include tab completion and helpful error messages.

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
