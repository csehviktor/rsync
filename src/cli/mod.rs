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
    /// Initialize configuration and state files in the current directory
    #[command(alias = "init")]
    Initialize {
        /// Optional bucket name to prefill in the config
        bucket: Option<String>,
    },

    /// Compare local files against the bucket and show sync status
    #[command(alias = "ls")]
    List,

    /// Download remote changes
    Pull,

    /// Upload local changes
    Push,

    /// Verify credentials and test bucket reachability
    Validate,
}
