use clap::Parser;

use crate::cli::*;

pub mod cli;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Initialize { bucket } => initialize::run(&bucket),
        Command::List => list::run(),
        Command::Pull => pull::run(),
        Command::Push => push::run(),
        Command::Validate => validate::run(),
    }
}
