mod migrate;
mod users;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use dotenvy::dotenv;
use std::{env, error::Error, path::PathBuf, process::ExitCode};

/// hitster-cli - a tool for managing everything that needs to happen
///               behind the scenes of the hitster project

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// migrate from hitster list format v1 (csv) to v2 (yaml)
    Migrate {
        /// the path to the csv file
        file: PathBuf,
    },
    /// manage user permissions
    Users(UsersArgs),
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
#[command(arg_required_else_help = true)]
struct UsersArgs {
    #[command(subcommand)]
    command: UsersCommands,
}

#[derive(Subcommand)]
enum UsersCommands {
    /// list all users currently in the database
    List {},
}

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn Error>> {
    let _ = dotenv();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Migrate { file } => {
            let success = migrate::migrate(file.clone());
            if !success {
                return Ok(ExitCode::from(1));
            }
        }
        Commands::Users(args) => {
            let db = env::var("DATABASE_URL").expect("DATABASEURL environment variable not found");
            match args.command {
                UsersCommands::List {} => {
                    let success = users::list(&db).await;
                    if !success {
                        return Ok(ExitCode::from(1));
                    }
                }
            }
        }
    }

    Ok(ExitCode::from(0))
}
