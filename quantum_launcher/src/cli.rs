use clap::Command;
use colored::Colorize;
use ql_core::{
    err_no_log,
    json::{instance_config::InstanceConfigJson, version::VersionDetails},
    IntoStringError, LAUNCHER_DIR, LAUNCHER_VERSION_NAME,
};
use std::io::Write;

use crate::{
    menu_renderer::{DISCORD, GITHUB},
    state::get_entries,
};

fn command() -> Command {
    Command::new(if cfg!(target_os = "windows") {
        ".\\quantum_launcher.exe"
    } else {
        "./quantum_launcher"
    })
    .arg_required_else_help(false)
    .author("Mrmayman")
    .version(LAUNCHER_VERSION_NAME)
    .long_about(long_about())
    .subcommand(
        get_list_instance_subcommands("list-instances")
            .short_flag('l')
            .about("Lists all installed Minecraft instances")
            .long_about("Lists all installed Minecraft instances. Can be paired with hyphen-separated-flags like name-loader, name-version, loader-name-version"),
    )
    .subcommand(
        get_list_instance_subcommands("list-servers")
            .short_flag('s')
            .about("Lists all installed Minecraft servers")
            .long_about("Lists all installed Minecraft servers. Can be paired with hyphen-separated-flags like name-loader, name-version, loader-name-version"),
    )
    .subcommand(Command::new("list-available-versions").short_flag('a').about("Lists all downloadable versions, downloading a list from Mojang/Omniarchive"))
    .subcommand(Command::new("--no-sandbox").hide(true)) // This one doesn't do anything, but on Windows i686 it's automatically passed?
}

fn get_list_instance_subcommands(name: &'static str) -> Command {
    Command::new(name)
        // May god forgive me for what I'm about to do
        .subcommand(Command::new("name"))
        .subcommand(Command::new("version"))
        .subcommand(Command::new("loader"))
        .subcommand(Command::new("name-version"))
        .subcommand(Command::new("name-loader"))
        .subcommand(Command::new("version-name"))
        .subcommand(Command::new("version-loader"))
        .subcommand(Command::new("loader-name"))
        .subcommand(Command::new("loader-version"))
        .subcommand(Command::new("name-version-loader"))
        .subcommand(Command::new("name-loader-version"))
        .subcommand(Command::new("version-name-loader"))
        .subcommand(Command::new("version-loader-name"))
        .subcommand(Command::new("loader-name-version"))
        .subcommand(Command::new("loader-version-name"))
}

fn cmd_list_available_versions() {
    eprintln!("Listing downloadable versions...");
    let versions = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(ql_instances::list_versions())
        .strerr()
    {
        Ok(n) => n,
        Err(err) => {
            panic!("Could not list versions!\n{err}");
        }
    };

    let mut stdout = std::io::stdout().lock();
    for version in versions {
        writeln!(stdout, "{version}").unwrap();
    }
}

fn long_about() -> String {
    format!(
        r"
QuantumLauncher: A simple, powerful Minecraft launcher by Mrmayman

Website: https://mrmayman.github.io/quantumlauncher
Github : {GITHUB}
Discord: {DISCORD}
"
    )
}

enum PrintCmd {
    Name,
    Version,
    Loader,
}

fn cmd_list_instances(cmds: &[PrintCmd], dirname: &str) {
    let instances = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_entries(dirname.to_owned(), false))
        .strerr()
    {
        Ok(n) => n.0,
        Err(err) => {
            panic!("Could not list instances: {err}");
        }
    };

    for instance in instances {
        let mut has_printed = false;
        for cmd in cmds {
            match cmd {
                PrintCmd::Name => {
                    if has_printed {
                        print!("\t");
                    }
                    print!("{instance}");
                }
                PrintCmd::Version => {
                    if has_printed {
                        print!("\t");
                    }
                    let instance_dir = LAUNCHER_DIR.join(dirname).join(&instance);

                    let json = std::fs::read_to_string(instance_dir.join("details.json")).unwrap();
                    let json: VersionDetails = serde_json::from_str(&json).unwrap();

                    print!("{}", json.id);
                }
                PrintCmd::Loader => {
                    if has_printed {
                        print!("\t");
                    }
                    let instance_dir = LAUNCHER_DIR.join(dirname).join(&instance);
                    let config_json =
                        std::fs::read_to_string(instance_dir.join("config.json")).unwrap();
                    let config_json: InstanceConfigJson =
                        serde_json::from_str(&config_json).unwrap();

                    print!("{}", config_json.mod_type);
                }
            }
            has_printed = true;
        }
        if has_printed {
            println!();
        }
    }
}

/// Prints the "intro" to the screen
/// consisting of the **ASCII art logo**, as well as
/// **stylised text saying `QuantumLauncher`**
///
/// The actual data is `include_str!()`ed from
/// - `assets/ascii/icon.txt` for the ASCII art
/// - `assets/ascii/text.txt` for the text logo
///
/// The other files in `assets/ascii` are unused.
fn print_intro() {
    /// Helper function to pad lines to a fixed width
    fn pad_line(line: Option<&str>, width: usize) -> String {
        let line = line.unwrap_or_default();
        if line.len() < width {
            format!("{line:<width$}")
        } else {
            line.to_owned()
        }
    }

    const TEXT_WIDTH: u16 = 39;

    const LOGO: &str = include_str!("../../assets/ascii/icon.txt");
    const LOGO_WIDTH: u16 = 30;

    if cfg!(target_os = "windows") {
        return;
    }

    let (text, text_len_old) = get_side_text();

    let logo_len: usize = LOGO.lines().count();

    let Some((terminal_size::Width(width), _)) = terminal_size::terminal_size() else {
        return;
    };

    let mut stdout = std::io::stdout().lock();

    // Ok, this code is uncomfortably ugly but bear with me...
    if width > TEXT_WIDTH + LOGO_WIDTH {
        // Screen large enough for Text and Logo
        // to fit side-by-side
        let lines_len = std::cmp::max(text.lines().count(), LOGO.lines().count());
        for i in 0..lines_len {
            let text_line = pad_line(text.lines().nth(i), TEXT_WIDTH as usize);
            let logo_line = pad_line(LOGO.lines().nth(i), LOGO_WIDTH as usize);
            if i >= logo_len {
                _ = write!(stdout, "{logo_line} ");
            } else {
                _ = write!(stdout, "{} ", logo_line.purple().bold());
            }
            if i >= text_len_old {
                _ = write!(stdout, "{text_line}");
            } else {
                _ = write!(stdout, "{}", text_line.bold());
            }
            _ = writeln!(stdout);
        }
    } else if width >= TEXT_WIDTH {
        // Screen only large enough for
        // Text and Logo to fit one after another
        // vertically
        _ = writeln!(stdout, "{}\n{}", LOGO.purple().bold(), text.bold());
    } else if width >= LOGO_WIDTH {
        // Screen only large enough for Logo,
        // not text
        _ = writeln!(stdout, "{}", LOGO.purple().bold());
    } else {
        // Screen is too tiny
        _ = writeln!(stdout, "Quantum Launcher {LAUNCHER_VERSION_NAME}");
    }
    _ = writeln!(stdout);
}

fn get_side_text() -> (String, usize) {
    let mut text = include_str!("../../assets/ascii/text.txt").to_owned();
    let text_len_old = text.lines().count();

    let mut message = if cfg!(target_os = "windows") {
        "\n A simple, powerful Minecraft launcher".to_owned()
    } else {
        format!(
            "\n {}",
            "A simple, powerful Minecraft launcher".green().bold(),
        )
    };

    message.push_str("\n This window just shows debug info so\n feel free to ignore it\n\n ");

    let list_of_commands = if cfg!(target_os = "windows") {
        "For a list of commands type 'quantum_launcher.exe --help'".to_owned()
    } else {
        format!(
            "For a list of commands type\n {} {}",
            "./quantum_launcher".yellow().bold(),
            "--help".yellow()
        )
    };
    message.push_str(&list_of_commands);

    text.push_str(&message);

    (text, text_len_old)
}

pub fn start_cli(is_dir_err: bool) {
    let command = command();
    let matches = command.clone().get_matches();

    if let Some(subcommand) = matches.subcommand() {
        if is_dir_err {
            std::process::exit(1);
        }
        match subcommand.0 {
            "list-instances" => {
                let command = get_list_instance_subcommand(subcommand);
                cmd_list_instances(&command, "instances");
                std::process::exit(0);
            }
            "list-servers" => {
                let command = get_list_instance_subcommand(subcommand);
                cmd_list_instances(&command, "servers");
                std::process::exit(0);
            }
            "list-available-versions" => {
                cmd_list_available_versions();
                std::process::exit(0);
            }
            "--no-sandbox" => {
                err_no_log!("Unknown command --no-sandbox, ignoring...");
            }
            err => panic!("Unimplemented command! {err}"),
        }
    } else {
        print_intro();
    }
}

fn get_list_instance_subcommand(subcommand: (&str, &clap::ArgMatches)) -> Vec<PrintCmd> {
    if let Some((cmd, _)) = subcommand.1.subcommand() {
        let mut cmds = Vec::new();
        for cmd in cmd.split('-') {
            match cmd {
                "name" => cmds.push(PrintCmd::Name),
                "version" => cmds.push(PrintCmd::Version),
                "loader" => cmds.push(PrintCmd::Loader),
                invalid => {
                    panic!("Invalid subcommand {invalid}! Use any combination of name, version and loader separated by hyphen '-'");
                }
            }
        }
        cmds
    } else {
        vec![PrintCmd::Name]
    }
}
