use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Parser)]
#[command(
    name = "jir",
    version,
    about = "Java Install & Runtime manager",
    long_about = None,
    arg_required_else_help = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List installed versions, or add -i to browse all downloadable versions
    #[command(alias = "ls")]
    List {
        /// Show all installable versions
        #[arg(short = 'i')]
        installable: bool,
    },

    /// Download and install a JDK  e.g. jir install 21:temurin
    #[command(alias = "i")]
    Install {
        /// version:distro  e.g.  21:temurin  17:corretto
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        spec: Vec<String>,
    },

    /// Activate an installed JDK  e.g. jir use 21:temurin
    #[command(name = "use", alias = "u")]
    Use {
        /// version:distro  e.g.  21:temurin
        spec: String,
    },

    /// Remove an installed JDK  e.g. jir uninstall 21:temurin
    #[command(alias = "uni")]
    Uninstall {
        /// version:distro  e.g.  21:temurin
        spec: String,
    },

    /// Show the currently active JDK
    #[command(alias = "cur")]
    Current,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::List { installable } => commands::list::run(installable),
            Commands::Install { spec } => commands::install::run(&spec.join(" ")),
            Commands::Use { spec } => commands::switch::run(&spec),
            Commands::Uninstall { spec } => commands::uninstall::run(&spec),
            Commands::Current => commands::current::run(),
        }
    }
}
