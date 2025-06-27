use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "gwt",
    version,
    author,
    about = "Git worktree management tool",
    long_about = "A tool for managing git worktrees efficiently with hooks and configuration support",
    disable_version_flag = true
)]
pub struct Cli {
    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum CompletionAction {
    /// Generate completions to stdout
    Generate {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Install completions for your shell
    Install {
        /// Shell to install completions for (auto-detected if not specified)
        #[arg(value_enum)]
        shell: Option<clap_complete::Shell>,
    },
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Authenticate with GitHub
    Github {
        /// Logout and remove stored credentials
        #[arg(long)]
        logout: bool,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new worktree project from a repository URL
    Init {
        /// The repository URL to clone
        repo_url: String,
    },

    /// Add a new worktree for a branch
    Add {
        /// Branch name (can include slashes like feature/branch-name)
        branch_name: String,
    },

    /// List all worktrees in the current project
    List,

    /// Remove a worktree
    Remove {
        /// Branch name to remove (current worktree if not specified)
        branch_name: Option<String>,
    },

    /// Manage authentication for external services
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Generate or install shell completions
    Completions {
        /// Action to perform (defaults to generate)
        #[command(subcommand)]
        action: Option<CompletionAction>,
    },
}
