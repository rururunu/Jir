mod cli;
mod commands;
mod help;
mod jdk;
mod prompt;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if should_show_root_help(&args) {
        help::print_root_help();
        return Ok(());
    }

    let cli = cli::Cli::parse();
    cli.run()
}

fn should_show_root_help(args: &[String]) -> bool {
    matches!(
        args.get(1).map(String::as_str),
        None | Some("-h") | Some("--help") | Some("-help") | Some("help")
    )
}
