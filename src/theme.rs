//! Terminal theme mirroring the graphical installer.
//!
//! Colours are lifted from `packaging/windows/JirSetup.cs` so the CLI and the
//! Windows installer read as one product. The installer paints on a white
//! surface while terminals are frequently dark, so the palette carries the
//! wordmark, the subtitle and the field labels, and values keep the terminal's
//! own foreground colour.
//!
//! On top of that shared palette each command owns an accent hue ([`Accent`]),
//! and the help screen spends those hues on its command table: `jir -h` names
//! every verb in the colour that verb belongs to.
//!
//! The hues stop at the help screen. A command's own output keeps the terminal
//! conventions — the active JDK is blue, success is green, a warning is yellow —
//! because those carry state rather than identity.

use std::fmt::Write as _;
use std::io::IsTerminal;

use colored::{ColoredString, Colorize};

/// `titleLabel.ForeColor` and `installButton.BackColor` — the jir wordmark.
const BRAND: (u8, u8, u8) = (206, 32, 41);
/// `subtitleLabel.ForeColor` — the line printed under the wordmark.
const MUTED: (u8, u8, u8) = (130, 120, 120);
/// `statusLabel.ForeColor` — secondary text, used here for field labels.
const LABEL: (u8, u8, u8) = (88, 97, 115);

/// `subtitleLabel.Text` — the line printed under the wordmark.
const SUBTITLE: &str = "Manage Java runtimes fast.";

/// Value column of a `label  value` row, excluding the two-space indent.
const LABEL_WIDTH: usize = 10;

/// Name column of a command or flag table — wide enough for the longest
/// `name, alias` pair, `uninstall, uni`.
const ENTRY_WIDTH: usize = 16;

/// `titleLabel` as block letters — the terminal stand-in for the installer's
/// 34pt bold "jir". Rows are padded to a common width so the letters cannot
/// drift; `wordmark_rows_align` guards that.
const WORDMARK: [&str; 6] = [
    "     ██╗ ██╗ ██████╗ ",
    "     ██║ ██║ ██╔══██╗",
    "     ██║ ██║ ██████╔╝",
    "██   ██║ ██║ ██╔══██╗",
    "╚█████╔╝ ██║ ██║  ██║",
    " ╚════╝  ╚═╝ ╚═╝  ╚═╝",
];

fn tint(text: &str, rgb: (u8, u8, u8)) -> ColoredString {
    text.truecolor(rgb.0, rgb.1, rgb.2)
}

/// The accent hue a subcommand is known by in the help screen.
///
/// Hues are spread around the brand red rather than picked at random, and each
/// one is legible on both a white and a black terminal — `accents_stay_legible`
/// holds that line.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Accent {
    /// `list` — browsing the index
    List,
    /// `install` — adding a JDK
    Install,
    /// `use` — activating one
    Switch,
    /// `current` — reporting the active JDK
    Current,
    /// `uninstall` — removing one
    Uninstall,
    /// `update` — refreshing jir itself
    Update,
}

impl Accent {
    /// The accent a subcommand owns, by its canonical clap name. `None` for
    /// verbs that have no identity of their own — `help` prints this screen and
    /// falls back to the brand colour.
    fn from_name(name: &str) -> Option<Accent> {
        Some(match name {
            "list" => Accent::List,
            "install" => Accent::Install,
            "use" => Accent::Switch,
            "current" => Accent::Current,
            "uninstall" => Accent::Uninstall,
            "update" => Accent::Update,
            _ => return None,
        })
    }

    fn rgb(self) -> (u8, u8, u8) {
        match self {
            Accent::List => (70, 130, 195),      // azure
            Accent::Install => (32, 158, 140),   // teal
            Accent::Switch => BRAND,             // the signature action
            Accent::Current => (198, 74, 128),   // rose
            Accent::Uninstall => (214, 108, 32), // ember
            Accent::Update => (140, 110, 214),   // violet
        }
    }
}

/// Pads before colouring. `colored` measures the escape sequences it inserts,
/// so padding a `ColoredString` would push every column out of line.
fn column(text: &str, width: usize) -> String {
    let used = text.chars().count();
    if used >= width {
        format!("{text} ")
    } else {
        format!("{text}{}", " ".repeat(width - used))
    }
}

/// Wordmark and subtitle — the opening every themed screen shares. Each screen
/// passes its own subtitle: the installer tagline for `-v`, the command summary
/// for help.
fn push_header(out: &mut String, subtitle: &str) {
    let _ = writeln!(out);
    // block glyphs are already heavy; layering bold on top smears the strokes
    for row in WORDMARK {
        let _ = writeln!(out, "  {}", tint(row, BRAND));
    }
    let _ = writeln!(out, "  {}", tint(subtitle, MUTED));
}

/// Section title above a table, e.g. `Commands`.
fn push_heading(out: &mut String, text: &str) {
    let _ = writeln!(out);
    let _ = writeln!(out, "  {}", tint(text, MUTED).bold());
}

/// A `label  value` row, matching the labelled fields of the installer form.
fn push_field(out: &mut String, label: &str, value: impl std::fmt::Display) {
    let _ = writeln!(out, "  {}{}", tint(&column(label, LABEL_WIDTH), LABEL), value);
}

/// A `name  description` row of a table. The name carries the hue of the
/// command it names, so the column of verbs is also a colour key; rows with no
/// command behind them — the flags — keep the brand colour.
fn push_entry(out: &mut String, name: &str, accent: Option<Accent>, description: &str) {
    let rgb = accent.map_or(BRAND, |accent| accent.rgb());
    let _ = writeln!(out, "  {}{}", tint(&column(name, ENTRY_WIDTH), rgb), description);
}

/// Path of the running executable — what the installer calls the install
/// location.
fn location() -> String {
    std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "(unknown)".to_string())
}

/// Card printed by `jir -v`, styled after the installer window.
///
/// When stdout is not a terminal the plain `jir <version>` line is printed
/// instead so the flag stays usable in scripts. `CLICOLOR_FORCE` forces the
/// card, mirroring what the colour layer itself honours.
pub fn print_version(version: &str) {
    if !std::io::stdout().is_terminal() && std::env::var_os("CLICOLOR_FORCE").is_none() {
        println!("jir {}", version);
        return;
    }

    let mut out = String::new();
    push_header(&mut out, SUBTITLE);
    let _ = writeln!(out);
    push_field(&mut out, "version", tint(version, BRAND).bold());
    push_field(&mut out, "location", location());
    push_field(&mut out, "home", crate::jdk::jdks_base().display());
    let _ = writeln!(out);
    print!("{out}");
}

/// The help screen, styled after the installer window.
///
/// The command and flag tables are reflected out of clap rather than written
/// out by hand, so a new subcommand cannot leave this screen stale. Unlike
/// `-v` there is no single-line fallback: help is read by people even when it
/// is redirected to a file, and the colour layer already drops ANSI codes when
/// stdout is not a terminal.
fn help_text() -> String {
    let cmd = <crate::cli::Cli as clap::CommandFactory>::command();
    let mut out = String::new();

    let about = plain(cmd.get_about());
    push_header(&mut out, &about);

    push_heading(&mut out, "Usage");
    push_field(&mut out, "jir", "<command> [options]");

    push_heading(&mut out, "Commands");
    for sub in cmd.get_subcommands() {
        push_entry(
            &mut out,
            &with_aliases(sub.get_name(), sub.get_aliases()),
            accent_of(&cmd, sub.get_name()),
            &plain(sub.get_about()),
        );
    }

    push_heading(&mut out, "Options");
    for arg in cmd.get_arguments() {
        let mut names = Vec::new();
        if let Some(short) = arg.get_short() {
            names.push(format!("-{short}"));
        }
        if let Some(long) = arg.get_long() {
            names.push(format!("--{long}"));
        }
        if names.is_empty() {
            continue;
        }
        push_entry(&mut out, &names.join(", "), None, &plain(arg.get_help()));
    }

    if let Some(after) = cmd.get_after_help() {
        let _ = writeln!(out);
        for line in after.to_string().lines() {
            if line.trim().is_empty() {
                let _ = writeln!(out);
            } else {
                let _ = writeln!(out, "  {}", accent_example(&cmd, line));
            }
        }
    }

    out
}

/// Prints the themed help screen.
pub fn print_help() {
    print!("{}", help_text());
}

/// clap hands its text back as a `StyledStr`; the derive attributes carry no
/// markup, so this is the plain string.
fn plain(text: Option<&clap::builder::StyledStr>) -> String {
    text.map(|text| text.to_string()).unwrap_or_default()
}

/// `list, ls` — the name followed by every alias, so the table documents the
/// short forms that clap otherwise keeps to itself.
fn with_aliases<'a>(name: &str, aliases: impl Iterator<Item = &'a str>) -> String {
    let aliases: Vec<&str> = aliases.collect();
    if aliases.is_empty() {
        name.to_string()
    } else {
        format!("{name}, {}", aliases.join(", "))
    }
}

/// The accent of whatever command `token` names, whether it is spelled out or
/// abbreviated. Looking the aliases up in clap rather than repeating them here
/// keeps `ls` and `list` from drifting apart in two places.
fn accent_of(cmd: &clap::Command, token: &str) -> Option<Accent> {
    let sub = cmd
        .get_subcommands()
        .find(|sub| sub.get_name() == token || sub.get_aliases().any(|alias| alias == token))?;
    Accent::from_name(sub.get_name())
}

/// Paints the invocation at the head of an example line — `jir ls` — with the
/// hue of the command it runs, so the examples agree with the table above.
/// Captions and prose lines come back unchanged.
fn accent_example(cmd: &clap::Command, line: &str) -> String {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let Some(rest) = trimmed.strip_prefix("jir ") else {
        return line.to_string();
    };
    let token = rest.split_whitespace().next().unwrap_or_default();
    match accent_of(cmd, token) {
        Some(accent) => format!(
            "{indent}{}{}",
            tint(&format!("jir {token}"), accent.rgb()),
            &rest[token.len()..]
        ),
        None => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Block art breaks the moment one row is a column short, and the damage is
    /// invisible in code review — so state the invariant explicitly.
    #[test]
    fn wordmark_rows_align() {
        let width = WORDMARK[0].chars().count();
        for row in WORDMARK {
            assert_eq!(
                row.chars().count(),
                width,
                "{row:?} is not {width} columns wide"
            );
        }
    }

    /// Guards the glyph set: if a character ever falls outside the box-drawing
    /// and block ranges the art stops lining up in a terminal.
    #[test]
    fn wordmark_uses_block_glyphs_only() {
        for row in WORDMARK {
            for glyph in row.chars().filter(|g| !g.is_whitespace()) {
                let ok = matches!(glyph,
                    '\u{2580}'..='\u{259F}'   // block elements
                    | '\u{2500}'..='\u{257F}' // box drawing
                );
                assert!(ok, "{glyph:?} is not a block or box-drawing glyph");
            }
        }
    }

    const ALL_ACCENTS: [Accent; 6] = [
        Accent::List,
        Accent::Install,
        Accent::Switch,
        Accent::Current,
        Accent::Uninstall,
        Accent::Update,
    ];

    /// The palette has to reach the terminal as 24-bit escapes — a mistyped
    /// triple would otherwise only show up on someone else's screen.
    #[test]
    fn accents_reach_the_terminal_as_truecolor() {
        // `colored` disables itself when stdout is not a terminal, which is the
        // case under `cargo test`
        colored::control::set_override(true);

        let expected = [
            (Accent::List, "38;2;70;130;195"),
            (Accent::Install, "38;2;32;158;140"),
            (Accent::Switch, "38;2;206;32;41"),
            (Accent::Current, "38;2;198;74;128"),
            (Accent::Uninstall, "38;2;214;108;32"),
            (Accent::Update, "38;2;140;110;214"),
        ];
        for (accent, escape) in expected {
            let rendered = tint("x", accent.rgb()).to_string();
            assert!(
                rendered.contains(escape),
                "{accent:?} rendered as {rendered:?}, expected {escape}"
            );
        }
    }

    /// Two commands sharing a hue would defeat the point of colouring by
    /// command at all.
    #[test]
    fn every_command_has_its_own_accent() {
        for (index, one) in ALL_ACCENTS.iter().enumerate() {
            for other in &ALL_ACCENTS[index + 1..] {
                assert_ne!(one.rgb(), other.rgb(), "{one:?} and {other:?} share a hue");
            }
        }
    }

    /// Terminals come in both flavours, so no accent may be so pale that it
    /// washes out on white or so dark that it vanishes on black.
    #[test]
    fn accents_stay_legible_on_light_and_dark_terminals() {
        fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
            fn channel(value: u8) -> f64 {
                let value = value as f64 / 255.0;
                if value <= 0.03928 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            }
            0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
        }

        fn contrast(one: f64, other: f64) -> f64 {
            let (high, low) = if one > other { (one, other) } else { (other, one) };
            (high + 0.05) / (low + 0.05)
        }

        for accent in ALL_ACCENTS {
            let own = luminance(accent.rgb());
            for (name, background) in [("white", 1.0), ("black", 0.0)] {
                let ratio = contrast(own, background);
                assert!(
                    ratio >= 3.0,
                    "{accent:?} on {name} is only {ratio:.2}:1"
                );
            }
        }
    }

    /// The help screen has to open the same way as `-v`, or the two themed
    /// screens stop reading as one product.
    #[test]
    fn help_keeps_the_wordmark() {
        let help = help_text();
        for row in WORDMARK {
            assert!(help.contains(row), "help screen dropped {row:?}");
        }
        // `-v` follows the art with the installer tagline, help with the
        // command summary
        assert!(
            help.lines()
                .any(|line| line.contains("Java Install & Runtime manager")),
            "no subtitle line under the wordmark"
        );
    }

    /// Reflected over clap rather than compared against a literal list, so a
    /// subcommand added later cannot quietly miss the help screen.
    #[test]
    fn help_lists_every_subcommand_and_alias() {
        let help = help_text();
        let cmd = <crate::cli::Cli as clap::CommandFactory>::command();
        let mut aliases = 0;
        for sub in cmd.get_subcommands() {
            assert!(help.contains(sub.get_name()), "help omits `{}`", sub.get_name());
            for alias in sub.get_aliases() {
                assert!(help.contains(alias), "help omits alias `{alias}`");
                aliases += 1;
            }
        }
        assert!(aliases >= 6, "expected six short aliases, saw {aliases}");
    }

    /// `-h` and `-v` are both declared by hand, so neither would appear if the
    /// flag table were written out by hand.
    #[test]
    fn help_lists_every_flag() {
        let help = help_text();
        let cmd = <crate::cli::Cli as clap::CommandFactory>::command();
        for arg in cmd.get_arguments() {
            if let Some(short) = arg.get_short() {
                assert!(help.contains(&format!("-{short}")), "help omits `-{short}`");
            }
            if let Some(long) = arg.get_long() {
                assert!(help.contains(&format!("--{long}")), "help omits `--{long}`");
            }
        }
    }

    /// A subcommand renamed in `cli.rs` would quietly drop back to the brand
    /// colour — the one thing this table must not do.
    #[test]
    fn every_subcommand_resolves_to_an_accent() {
        let cmd = <crate::cli::Cli as clap::CommandFactory>::command();
        for sub in cmd.get_subcommands() {
            let name = sub.get_name();
            if name == "help" {
                assert!(
                    accent_of(&cmd, name).is_none(),
                    "`help` prints this screen; it has no hue of its own"
                );
            } else {
                assert!(accent_of(&cmd, name).is_some(), "`{name}` has no accent");
            }
        }
    }

    /// The point of the screen: six verbs, six hues, each one the colour that
    /// verb prints in.
    #[test]
    fn help_paints_each_command_with_its_own_accent() {
        // `colored` switches itself off when stdout is not a terminal, which is
        // what it sees under `cargo test`
        colored::control::set_override(true);

        let cmd = <crate::cli::Cli as clap::CommandFactory>::command();
        let help = help_text();
        let mut painted = 0;
        for sub in cmd.get_subcommands() {
            let Some(accent) = accent_of(&cmd, sub.get_name()) else {
                continue;
            };
            let (r, g, b) = accent.rgb();
            let escaped = format!("\u{1b}[38;2;{r};{g};{b}m{}", sub.get_name());
            assert!(
                help.contains(&escaped),
                "`{}` is not painted {:?}",
                sub.get_name(),
                accent
            );
            painted += 1;
        }
        assert_eq!(painted, ALL_ACCENTS.len(), "not every accent reached the table");
    }

    /// An alias stands for its command, so it wears the same colour — in the
    /// table and in the examples underneath it.
    #[test]
    fn aliases_and_examples_borrow_the_command_hue() {
        colored::control::set_override(true);

        let help = help_text();
        let (r, g, b) = Accent::List.rgb();
        let azure = format!("\u{1b}[38;2;{r};{g};{b}m");
        assert!(
            help.contains(&format!("{azure}list, ls")),
            "`ls` does not share the hue of `list`"
        );
        assert!(
            help.contains(&format!("{azure}jir ls")),
            "the `jir ls` example is not painted like `list`"
        );
    }
}
