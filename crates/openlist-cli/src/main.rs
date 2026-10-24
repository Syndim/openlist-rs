mod cli;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    commands::run(cli::Cli::parse()).await
}
