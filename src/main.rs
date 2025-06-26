use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod cli;
mod commands;
mod completions;
mod config;
mod git;
mod hooks;
mod shell_integration;
mod utils;

use cli::{Cli, Commands, CompletionAction};
use commands::{add, init, list, remove};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { repo_url } => {
            init::run(&repo_url)?;
        }
        Commands::Add { branch_name, print_path } => {
            add::run(&branch_name, print_path)?;
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
        Commands::ShellInit { shell } => {
            println!("{}", shell_integration::generate_shell_integration(shell));
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
            // Output the pre-generated completion to stdout
            println!("{}", completions::get_completion_content(shell));
        }
        Some(CompletionAction::Install { shell, with_integration }) => {
            let shell = shell.unwrap_or_else(|| {
                completions::detect_shell().unwrap_or(clap_complete::Shell::Bash)
            });
            completions::install_completions_for_shell(shell)?;
            
            if with_integration {
                install_shell_integration(shell)?;
            }
        }
    }
    Ok(())
}

fn check_completions_status() -> Result<()> {
    let shell = completions::detect_shell()?;
    println!("Detected shell: {}", shell.to_string().green());

    let installed = completions::check_completions_installed(shell)?;

    if installed {
        println!("✓ Completions appear to be installed");
        println!(
            "\nTo reinstall or update, run: {}",
            "gwt completions install".cyan()
        );
    } else {
        println!("✗ Completions not installed");
        println!(
            "\nTo install completions, run: {}",
            "gwt completions install".cyan()
        );
    }

    println!("\nTo generate completions for a specific shell:");
    println!("  {}", "gwt completions generate <shell>".cyan());
    println!("\nSupported shells: bash, zsh, fish, powershell, elvish");

    Ok(())
}

fn install_shell_integration(shell: clap_complete::Shell) -> Result<()> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    
    let home = env::var("HOME").map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
    
    let (config_file, install_instructions) = match shell {
        clap_complete::Shell::Bash => {
            let bashrc = PathBuf::from(&home).join(".bashrc");
            (bashrc, "source ~/.bashrc")
        }
        clap_complete::Shell::Zsh => {
            let zshrc = PathBuf::from(&home).join(".zshrc");
            (zshrc, "source ~/.zshrc")
        }
        clap_complete::Shell::Fish => {
            let fish_config = PathBuf::from(&home).join(".config/fish/config.fish");
            // Create fish config directory if it doesn't exist
            if let Some(parent) = fish_config.parent() {
                fs::create_dir_all(parent)?;
            }
            (fish_config, "restart fish or run 'source ~/.config/fish/config.fish'")
        }
        _ => {
            println!("{}: Shell integration for {} is not yet implemented", "Note".yellow(), shell);
            println!("You can generate the integration code manually:");
            println!("  {}", format!("gwt shell-init {}", shell).cyan());
            return Ok(());
        }
    };
    
    // Get the integration code
    let integration_code = shell_integration::generate_shell_integration(shell);
    
    // Read current config or create empty
    let mut current_content = if config_file.exists() {
        fs::read_to_string(&config_file)?
    } else {
        String::new()
    };
    
    // Check if integration is already installed
    if current_content.contains("gwt shell integration") {
        println!("{}: Shell integration already appears to be installed", "Note".yellow());
        println!("To reinstall, remove the existing 'gwt shell integration' section from {}", config_file.display());
        return Ok(());
    }
    
    // Add integration code
    if !current_content.ends_with('\n') && !current_content.is_empty() {
        current_content.push('\n');
    }
    current_content.push_str(integration_code);
    
    // Write back to config file
    fs::write(&config_file, current_content)?;
    
    println!("✓ Installed shell integration to: {}", config_file.display().to_string().cyan());
    println!("\nTo activate the integration, run:");
    println!("  {}", install_instructions.cyan());
    println!("\nUsage:");
    println!("  {} - Create worktree and auto-navigate", "gwt_add feature/branch".green());
    println!("  {} - Generate integration for other shells", "gwt shell-init <shell>".cyan());
    
    Ok(())
}
