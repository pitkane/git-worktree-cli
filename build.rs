use std::env;
use std::fs;
use std::path::Path;

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};

#[path = "src/cli.rs"]
mod cli;
use cli::Cli;

fn main() -> std::io::Result<()> {
    let outdir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let completions_dir = Path::new(&outdir).join("completions");
    fs::create_dir_all(&completions_dir)?;

    let mut cmd = Cli::command();

    // Generate completions for all supported shells
    for shell in Shell::value_variants() {
        let path = generate_to(*shell, &mut cmd, "gwt", &completions_dir)?;

        println!("Generated {} completions: {:?}", shell, path);
    }

    // Tell Cargo to rerun this script if cli.rs changes
    println!("cargo:rerun-if-changed=src/cli.rs");

    Ok(())
}
