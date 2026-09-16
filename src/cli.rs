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
    after_help = "Examples:\n  \
        jir ls -i            show downloadable versions\n  \
        jir i 21             pick a vendor and install Java 21\n  \
        jir i 21:temurin     install Temurin 21 directly\n  \
        jir use 21           activate an installed Java 21\n  \
        jir use              pick from every installed JDK\n  \
        jir current          show active JAVA_HOME\n\n\
        Notes:\n  \
        The active JDK is exposed through home/occupy — point JAVA_HOME there once."
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

        /// With -i: only show this Java feature version  e.g. jir ls -i 21
        filter: Option<String>,
    },

    /// Download and install one or more JDKs  e.g. jir install 21:temurin 17:corretto
    #[command(alias = "i")]
    Install {
        /// version[:distro] ...  e.g.  21  or  21:temurin 17:corretto
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        spec: Vec<String>,
    },

    /// Activate an installed JDK  e.g. jir use 21:temurin
    #[command(name = "use", alias = "u")]
    Use {
        /// version:distro  e.g.  21:temurin   (omit to pick from installed)
        spec: Option<String>,
    },

    /// Remove an installed JDK  e.g. jir uninstall 21:temurin
    #[command(alias = "uni")]
    Uninstall {
        /// version:distro  e.g.  21:temurin
        spec: String,

        /// Skip the confirmation prompt
        #[arg(short = 'y', long = "force")]
        force: bool,
    },

    /// Show the currently active JDK
    #[command(alias = "cur")]
    Current,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::List { installable, filter } => commands::list::run(installable, filter.as_deref()),
            Commands::Install { spec } => {
                anyhow::ensure!(
                    !spec.is_empty(),
                    "expected  version[:distro]  e.g.  jir i 21:temurin"
                );
                for spec in &spec {
                    commands::install::run(spec)?;
                }
                Ok(())
            }
            Commands::Use { spec } => commands::switch::run(spec.as_deref()),
            Commands::Uninstall { spec, force } => commands::uninstall::run(&spec, force),
            Commands::Current => commands::current::run(),
        }
    }
}
