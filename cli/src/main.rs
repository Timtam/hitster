use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::ExitCode;

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
    Migrate {},
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Migrate {}) => {
            return Ok(ExitCode::from(0));
        }
        _ => {}
    }

    Ok(ExitCode::from(0))
}
