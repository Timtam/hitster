mod migrate;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, process::ExitCode};

/// hitster-cli - a tool for managing everything that needs to happen
///               behind the scenes of the hitster project

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// migrate from hitster list format v1 (csv) to v2 (yaml)
    Migrate {
        /// the path to the csv file
        file: PathBuf,
    },
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Migrate { file }) => {
            let success = migrate::migrate(file.clone());
            if success {
                return Ok(ExitCode::from(0));
            } else {
                return Ok(ExitCode::from(1));
            }
        }
        _ => {}
    }

    Ok(ExitCode::from(0))
}
