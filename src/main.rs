use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::Colorize;

mod commands;
mod config;
mod git;
mod hooks;
mod utils;

use commands::{init, add, list, remove};

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
enum CompletionAction {
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
    
    /// Remove a worktree
    Remove {
        /// Branch name to remove (current worktree if not specified)
        branch_name: Option<String>,
    },
    
    /// Generate or install shell completions
    Completions {
        /// Action to perform (defaults to generate)
        #[command(subcommand)]
        action: Option<CompletionAction>,
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
        Commands::Remove { branch_name } => {
            remove::run(branch_name.as_deref())?;
        }
        Commands::Completions { action } => {
            handle_completions(action)?;
        }
    }
    
    Ok(())
}

fn handle_completions(action: Option<CompletionAction>) -> Result<()> {
    match action {
        None => {
            // Default behavior: check if completions are installed
            check_completions_status()?;
        }
        Some(CompletionAction::Generate { shell }) => {
            generate_completions(shell);
        }
        Some(CompletionAction::Install { shell }) => {
            install_completions(shell)?;
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

fn check_completions_status() -> Result<()> {
    use std::env;
    use std::path::PathBuf;
    
    let shell = detect_shell()?;
    let home = env::var("HOME").map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
    let zshrc = PathBuf::from(&home).join(".zshrc");
    
    println!("Detected shell: {}", shell.to_string().green());
    
    if zshrc.exists() {
        let content = std::fs::read_to_string(&zshrc)?;
        if content.contains("gwt-completions.zsh") || content.contains("_gwt") {
            println!("✓ Completions appear to be installed");
            println!("\nTo reinstall or update, run: {}", "gwt completions install".cyan());
        } else {
            println!("✗ Completions not installed");
            println!("\nTo install completions, run: {}", "gwt completions install".cyan());
        }
    } else {
        println!("✗ No .zshrc found");
        println!("\nTo install completions, run: {}", "gwt completions install".cyan());
    }
    
    Ok(())
}

fn detect_shell() -> Result<clap_complete::Shell> {
    use std::env;
    
    // Try to detect from SHELL environment variable
    if let Ok(shell_path) = env::var("SHELL") {
        if shell_path.contains("zsh") {
            return Ok(clap_complete::Shell::Zsh);
        } else if shell_path.contains("bash") {
            return Ok(clap_complete::Shell::Bash);
        } else if shell_path.contains("fish") {
            return Ok(clap_complete::Shell::Fish);
        }
    }
    
    // Default to zsh
    Ok(clap_complete::Shell::Zsh)
}

fn install_completions(shell: Option<clap_complete::Shell>) -> Result<()> {
    
    let shell = shell.unwrap_or_else(|| detect_shell().unwrap_or(clap_complete::Shell::Zsh));
    
    match shell {
        clap_complete::Shell::Zsh => install_zsh_completions()?,
        _ => {
            println!("Automatic installation for {} is not yet implemented", shell);
            println!("\nYou can generate completions manually:");
            println!("  gwt completions generate {}", shell);
            return Ok(());
        }
    }
    
    Ok(())
}

fn install_zsh_completions() -> Result<()> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    
    let home = env::var("HOME").map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
    let zshrc_path = PathBuf::from(&home).join(".zshrc");
    
    // Get the path to the completions directory
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine executable directory"))?;
    
    // Look for completions in common locations
    let possible_paths = vec![
        exe_dir.join("../completions/gwt-completions.zsh"),
        exe_dir.join("../../completions/gwt-completions.zsh"), // for target/release/gwt
        PathBuf::from(&home).join(".git-worktree-scripts/completions/gwt-completions.zsh"),
        PathBuf::from("/usr/local/share/git-worktree-scripts/completions/gwt-completions.zsh"),
        PathBuf::from("/opt/git-worktree-scripts/completions/gwt-completions.zsh"),
    ];
    
    let completion_path = possible_paths.iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("Could not find gwt-completions.zsh file"))?;
    
    println!("Found completion file at: {}", completion_path.display());
    
    // Check if .zshrc exists
    if !zshrc_path.exists() {
        println!("Creating ~/.zshrc...");
        fs::write(&zshrc_path, "")?;
    }
    
    // Read current .zshrc content
    let mut zshrc_content = fs::read_to_string(&zshrc_path)?;
    
    // Check if completions are already installed
    if zshrc_content.contains("gwt-completions.zsh") || zshrc_content.contains("GWT completions") {
        println!("Completions already appear to be installed in ~/.zshrc");
        println!("Would you like to reinstall? This will update the configuration.");
        // For now, we'll just update
        // In a real implementation, we might want to prompt the user
    }
    
    // Prepare the completion setup code
    let completion_setup = format!(
        r#"
# GWT completions
if [[ -f {} ]]; then
    fpath=({} $fpath)
    autoload -Uz compinit && compinit
    source {}
fi"#,
        completion_path.display(),
        completion_path.parent().unwrap().display(),
        completion_path.display()
    );
    
    // Remove old completion setup if it exists
    let lines: Vec<&str> = zshrc_content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_until_fi = false;
    
    for line in lines {
        if line.contains("GWT completions") || line.contains("gwt-completions.zsh") {
            skip_until_fi = true;
            continue;
        }
        if skip_until_fi && line.trim() == "fi" {
            skip_until_fi = false;
            continue;
        }
        if !skip_until_fi {
            new_lines.push(line);
        }
    }
    
    // Rebuild content and add new completion setup
    zshrc_content = new_lines.join("\n");
    if !zshrc_content.ends_with('\n') && !zshrc_content.is_empty() {
        zshrc_content.push('\n');
    }
    zshrc_content.push_str(&completion_setup);
    zshrc_content.push('\n');
    
    // Write back to .zshrc
    fs::write(&zshrc_path, zshrc_content)?;
    
    println!("✓ Successfully installed zsh completions!");
    println!("\nTo activate completions in your current shell, run:");
    println!("  {}", "source ~/.zshrc".cyan());
    println!("\nOr start a new terminal session.");
    
    Ok(())
}