# Git Worktree CLI (gwt)

More important fix

🌿 **Enhanced Git Worktree Management with Rust** 🌿

Stop juggling multiple git clones or constantly switching branches. Git worktrees let you have multiple working directories from the same repository, each checked out to different branches. This Rust-powered tool makes managing them effortless with **real-time streaming output**.

*This project was built with [Claude Code](https://claude.ai/code) using Opus 4 and Sonnet 4 models.*

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
   git clone git@github.com:pitkane/git-worktree-cli.git ~/.git-worktree-cli
   cd ~/.git-worktree-cli
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
   echo 'export PATH="$HOME/.git-worktree-cli/target/release:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

4. **Install shell completions (recommended):**
   ```bash
   # Check if completions are installed
   gwt completions
   
   # Automatically install completions for your shell
   gwt completions install
   
   # Or install for a specific shell
   gwt completions install bash
   gwt completions install zsh
   gwt completions install fish
   gwt completions install powershell
   gwt completions install elvish
   
   # Generate completions to stdout (for manual installation)
   gwt completions generate <shell>
   ```
   
   **Note**: Completions are embedded in the binary, so they're always available!

### Option 2: Direct Binary Download (Coming Soon)
Pre-built binaries will be available for:
- macOS (Intel & Apple Silicon)
- Linux (x86_64 & ARM64)
- Windows

### Option 3: Use Legacy TypeScript Version
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
# - main/ directory (or master/ based on default branch)
# - git-worktree-config.yaml (project metadata)
# You'll see git clone progress in real-time!
```

### 2. Create Feature Branches
```bash
# Create new feature worktree
gwt add feature/user-auth
# Creates feature/user-auth/ directory

# Create bugfix worktree  
gwt add bugfix/login-error
# Creates bugfix/login-error/ directory
```

### 3. List Your Worktrees
```bash
# See all your worktrees in a clean table
gwt list

# Output:
# ┌────────────────────┬─────────────────────────────────────────────────────┐
# │ BRANCH             │ PULL REQUEST                                        │
# ├────────────────────┼─────────────────────────────────────────────────────┤
# │ main               │ -                                                   │
# │ feature/user-auth  │ https://github.com/owner/repo/pull/42 (open)       │
# │ bugfix/login-error │ https://github.com/owner/repo/pull/41 (draft)      │
# └────────────────────┴─────────────────────────────────────────────────────┘
```

### 4. Switch Between Work
```bash
# Navigate to any worktree directory
cd ../feature/user-auth
cd ../main
# No git checkout needed!
```

### 5. Clean Up When Done
```bash
# Remove completed feature
gwt remove feature/user-auth  # Removes worktree

# Or remove current worktree
cd ../feature/old-feature
gwt remove  # Removes current worktree you're in
```

## Real-World Example

```bash
# Start new project with streaming git clone output
gwt init git@github.com:company/web-app.git
cd main

# Work on a new feature
gwt add feature/shopping-cart
cd ../feature/shopping-cart
# Make commits, push changes...

# Urgent bugfix needed - no stashing required!
gwt add hotfix/payment-bug
cd ../hotfix/payment-bug
# Fix bug, commit, push...

# Back to feature work
cd ../feature/shopping-cart
# Continue where you left off

# Review all work
gwt list

# Clean up merged work
gwt remove hotfix/payment-bug
```

## Commands Reference

| Command | Description | Example | Status |
|---------|-------------|---------|---------|
| `gwt init <url>` | Initialize worktree project from repo | `gwt init git@github.com:user/repo.git` | ✅ **Working** |
| `gwt list` | List all worktrees in a table | `gwt list` | ✅ **Working** |
| `gwt add <branch>` | Create new worktree for branch | `gwt add feature/new-ui` | ✅ **Working** |
| `gwt remove [branch]` | Remove worktree (current if no args) | `gwt remove old-feature` | ✅ **Working** |
| `gwt completions` | Check completion status | `gwt completions` | ✅ **Working** |
| `gwt completions install [shell]` | Auto-install completions | `gwt completions install` | ✅ **Working** |
| `gwt completions generate <shell>` | Generate completions | `gwt completions generate zsh` | ✅ **Working** |

**New in Rust version:**
- ✅ **Real-time streaming output** - See git clone progress live!
- ✅ **Single binary** - No Node.js dependency
- ✅ **Embedded completions** - Completions built into the binary at compile time
- ✅ **Multi-shell support** - Bash, Zsh, Fish, PowerShell, and Elvish
- ✅ **Smart completions** - Auto-detect shell and install with one command
- ✅ **Better performance** - Compiled Rust vs interpreted TypeScript
- ✅ **Sharp table output** - Clean, modern table formatting

## Hooks & Automation

Git worktree scripts support **hooks** - custom commands that run automatically after worktree operations. Perfect for automating setup tasks like installing dependencies or running initialization scripts.

### Quick Setup
```bash
# After gwt init, edit git-worktree-config.yaml
hooks:
  postAdd:
    - "npm install"      # Auto-install deps in new worktrees
    - "npm run init"     # Run your custom setup script
```

### Example Workflow with Hooks
```bash
# Initialize project (creates config with hook examples)
gwt init git@github.com:company/web-app.git

# Edit git-worktree-config.yaml to enable hooks:
# hooks:
#   postAdd:
#     - "npm install"    # Remove # to enable

# Create new worktree - hooks run automatically!
gwt add feature/shopping-cart
# This will:
# 1. Create the worktree  
# 2. Run "npm install" automatically with streaming output
# 3. cd to the new directory

# Continue with your work - dependencies already installed!
```

### Available Hook Types
- **`postInit`**: After creating a new project
- **`postAdd`**: After creating a new worktree (perfect for setup)
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

## GitHub Integration

View GitHub pull request information directly in your worktree list!

### Setup GitHub Authentication
```bash
# Authenticate with GitHub (one-time setup)
gwt auth github

# This will:
# 1. Open your browser to GitHub
# 2. Display a code to enter
# 3. Save your authentication securely
```

### View PR Status
```bash
# List worktrees with PR info
gwt list

# Shows PR URL and status for each branch
# ┌────────────────────┬─────────────────────────────────────────────────────┐
# │ BRANCH             │ PULL REQUEST                                        │
# ├────────────────────┼─────────────────────────────────────────────────────┤
# │ main               │ -                                                   │
# │ feature/new-ui     │ https://github.com/owner/repo/pull/123 (open)      │
# │ fix/memory-leak    │ https://github.com/owner/repo/pull/122 (draft)     │
# └────────────────────┴─────────────────────────────────────────────────────┘
```

### Managing Authentication
```bash
# Check authentication status
gwt auth github

# Remove stored credentials
gwt auth github --logout
```

For setup instructions, see [docs/GITHUB_AUTH_SETUP.md](docs/GITHUB_AUTH_SETUP.md).

## Benefits

- **🚀 No Context Switching**: Each branch keeps its own working directory
- **🔄 Instant Branch Switching**: Just cd to the directory
- **🛡️ Safe Experimentation**: Isolated working directories prevent conflicts
- **⚡ Parallel Development**: Work on multiple features simultaneously
- **🧹 Easy Cleanup**: Remove completed work with one command
- **🪝 Smart Automation**: Hooks automatically run setup/cleanup tasks
- **📊 Real-time Feedback**: See command output as it executes
- **🎯 Tab Completion**: Branch names auto-complete for add/remove commands
- **🔗 GitHub Integration**: View pull request status directly in worktree list

## Requirements

- **Rust 1.70+** (for building from source)
- **Git 2.5+** (for worktree support)
- **Bash/Zsh/Fish shell** (for completions)

## Development

```bash
# Build debug binary
cargo build

# Build release binary
cargo build --release

# Run tests
cargo test

# Type checking
cargo check

# Run with cargo
cargo run -- <command>
```

## Troubleshooting

### Completions not working?
```bash
# Check if completions are installed
gwt completions

# Auto-install completions for your detected shell
gwt completions install

# Or specify a shell explicitly
gwt completions install zsh

# For manual installation, generate completions
gwt completions generate zsh > ~/.local/share/zsh/site-functions/_gwt

# Reload your shell
source ~/.zshrc  # or exec zsh
```

**Supported shells**: bash, zsh, fish, powershell, elvish

### Command not found?
Make sure the binary is in your PATH:
```bash
which gwt
# If not found, add to PATH:
export PATH="$HOME/.git-worktree-cli/target/release:$PATH"
```

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests (`cargo test`)
5. Submit a pull request

## License

MIT License - see LICENSE file for details
