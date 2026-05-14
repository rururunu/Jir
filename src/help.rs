use colored::Colorize;

pub fn print_root_help() {
    println!();
    println!("  {}", "jir".bold().cyan());
    println!("  {}", "Java Install & Runtime manager".dimmed());
    println!();
    println!("  {}", "Usage".bold());
    println!("    {}", "jir <command> [options]".green());
    println!();
    println!("  {}", "Commands".bold());
    command("ls, list", "List installed JDKs");
    command("ls -i, list -i", "Browse installable JDKs");
    command("i, install <version[:distro]>", "Download and install a JDK");
    command("use <version[:distro]>", "Activate an installed JDK");
    command("uni, uninstall <version[:distro]>", "Remove an installed JDK");
    command("current, cur", "Show the active JDK");
    command("-h, -help, --help", "Show this help page");
    println!();
    println!("  {}", "Examples".bold());
    example("jir ls -i", "show downloadable versions");
    example("jir i 21", "pick a vendor and install Java 21");
    example("jir i 21:temurin", "install Temurin 21 directly");
    example("jir use 21", "pick an installed Java 21 vendor and activate it");
    example("jir current", "show active JAVA_HOME");
    println!();
    println!("  {}", "Notes".bold());
    println!(
        "    {}",
        "Active JDK is exposed through home/occupy. Point JAVA_HOME there once.".dimmed()
    );
    println!();
}

fn command(name: &str, description: &str) {
    println!("    {:<38} {}", name.green(), description);
}

fn example(command: &str, description: &str) {
    println!("    {:<38} {}", command.cyan(), description.dimmed());
}
