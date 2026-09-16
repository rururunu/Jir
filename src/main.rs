mod cli;
mod commands;
mod jdk;
mod prompt;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    cli::Cli::parse().run()
}
