use clap::{Parser, Subcommand};

pub mod initialize;
pub mod list;
pub mod pull;
pub mod push;
pub mod validate;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Initialize { bucket: Option<String> },
    List,
    Pull,
    Push,
    Validate,
}
