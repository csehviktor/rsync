use std::process::ExitCode;

use clap::Parser;

use crate::cli::*;

pub mod cli;
pub mod error;
pub mod persistance;
pub mod r2;
pub mod walk;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Initialize { bucket } => initialize::run(&bucket),
        Command::List => list::run(),
        Command::Pull => pull::run(),
        Command::Push => push::run(),
        Command::Validate => validate::run(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
