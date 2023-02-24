use std::io;

use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(about = "An archive utility", long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show supported formats
    ListFormats,
    /// Check archive integrity
    Check(commands::CheckArgs),
    /// List files
    List(commands::ListArgs),
    /// Extract files
    Extract(commands::ExtractArgs),
    /// Create archive
    Create(commands::CreateArgs),
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let verbose = args.verbose;

    match args.command {
        Commands::ListFormats => {
            commands::list_formats();
        }

        Commands::Check(args) => {
            commands::check(args, verbose)?;
        }

        Commands::List(args) => {
            commands::list(args)?;
        }

        Commands::Extract(args) => {
            commands::extract(args, verbose)?;
        }

        Commands::Create(args) => {
            commands::create(args)?;
        }
    }

    Ok(())
}
