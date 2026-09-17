use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Parser)]
#[command(
    name = "jir",
    version,
    // the flag is declared by hand below so `-v` works, not just `-V`
    disable_version_flag = true,
    // Deliberately NOT `disable_help_flag`: clap keeps that as a *global*
    // setting and ORs it into every subcommand (`_propagate_subcommand`), which
    // left `jir ls -h` with no help flag at all. The root screen is intercepted
    // in `main` instead, so subcommands keep clap's own flag.
    disable_help_subcommand = true,
    about = "Java Install & Runtime manager",
    long_about = None,
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
    /// Print the installed jir version
    #[arg(short = 'v', short_alias = 'V', long = "version")]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
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

    /// Update jir itself to the newest release
    #[command(alias = "up")]
    Update {
        /// Reinstall even when this is already the newest release
        #[arg(short = 'f', long = "force")]
        force: bool,
    },

    /// Print this help
    Help,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        if self.version {
            crate::theme::print_version(env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        // A bare `jir` used to be answered by clap's own help text. It is ours
        // now, and it keeps clap's exit code: 2, the invocation asked for
        // nothing.
        let Some(command) = self.command else {
            crate::theme::print_help();
            std::process::exit(2);
        };

        match command {
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
            Commands::Update { force } => commands::update::run(force),
            Commands::Help => {
                crate::theme::print_help();
                Ok(())
            }
        }
    }
}
