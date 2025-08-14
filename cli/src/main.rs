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
    /// manage users
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
    /// edit a user
    Edit(UsersEditArgs),
    /// list all users currently in the database
    List {},
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(arg_required_else_help = true)]
struct UsersEditArgs {
    /// the user id (see hitster-cli users list)
    id: String,
    /// make the user an admin (grand them all permissions)
    #[arg(short, long, group = "perm", action = clap::ArgAction::SetTrue)]
    admin: Option<bool>,
    /// give the user specific permissions (see hitster-cli users list)
    #[arg(short, long, group = "perm")]
    permissions: Option<u32>,
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
            match &args.command {
                UsersCommands::Edit(args) => {
                    let success = users::edit(
                        &db,
                        &args.id,
                        users::EditArgs {
                            admin: args.admin.unwrap_or(false),
                            permissions: args.permissions.unwrap_or(0),
                        },
                    )
                    .await;
                    if !success {
                        return Ok(ExitCode::from(1));
                    }
                }
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
