use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Nvim(NvimArgs),
}

#[derive(Args)]
pub struct NvimArgs {
    #[arg(long)]
    pub socket: PathBuf,
}
