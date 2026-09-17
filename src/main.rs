mod cli;
mod commands;
mod jdk;
mod prompt;
mod theme;

use anyhow::Result;
use clap::Parser;

/// True when `-h`/`--help` was given to the root command itself.
///
/// clap can only switch its help flag off as a *global* setting, which every
/// subcommand inherits — that is what silently broke `jir ls -h`. clap keeps
/// its flag for the subcommands, so the root screen is drawn here instead.
fn asks_for_root_help() -> bool {
    std::env::args()
        .skip(1)
        .take_while(|arg| arg.starts_with('-'))
        .any(|arg| arg == "-h" || arg == "--help")
}

fn main() -> Result<()> {
    if asks_for_root_help() {
        theme::print_help();
        return Ok(());
    }
    cli::Cli::parse().run()
}
