use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::Colorize;

mod commands;
mod config;
mod git;
mod hooks;
mod utils;

use commands::{init, add, list, switch, remove};

#[derive(Parser)]
#[command(
    name = "gwt",
    version,
    author,
    about = "Git worktree management tool",
    long_about = "A tool for managing git worktrees efficiently with hooks and configuration support"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    
    /// Switch to a different worktree
    Switch {
        /// Branch name to switch to
        branch_name: String,
    },
    
    /// Remove a worktree
    Remove {
        /// Branch name to remove (current worktree if not specified)
        branch_name: Option<String>,
    },
    
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { repo_url } => {
            init::run(&repo_url)?;
        }
        Commands::Add { branch_name } => {
            add::run(&branch_name)?;
        }
        Commands::List => {
            list::run()?;
        }
        Commands::Switch { branch_name } => {
            switch::run(&branch_name)?;
        }
        Commands::Remove { branch_name } => {
            remove::run(branch_name.as_deref())?;
        }
        Commands::Completions { shell } => {
            generate_completions(shell);
        }
    }
    
    Ok(())
}

fn generate_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut std::io::stdout());
}