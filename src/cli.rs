use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Nvim(NvimArgs),
}

#[derive(Args, Debug)]
pub struct NvimArgs {
    #[arg(long)]
    pub socket: PathBuf,
}
