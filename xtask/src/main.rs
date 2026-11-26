//! Developer commands, run as `cargo xtask <command>`.
//!
//! Keeping these in Rust means one tool chain on every platform and no
//! shell scripts to maintain.

use std::env;
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <command>

Commands:
  run     Start the session in a window: lock screen, then home screen
  check   Run the checks CI runs: fmt, clippy and tests
  help    Show this message
";

/// A command the user asked for.
#[derive(Debug, PartialEq, Eq)]
enum Task {
    Run,
    Check,
    Help,
}

/// Reads the task from the arguments after the program name.
fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Task, String> {
    let mut args = args.into_iter();
    let task = match args.next().as_deref() {
        Some("run") => Task::Run,
        Some("check") => Task::Check,
        None | Some("help" | "--help" | "-h") => Task::Help,
        Some(other) => return Err(format!("unknown command `{other}`")),
    };
    match args.next() {
        Some(extra) => Err(format!("unexpected argument `{extra}`")),
        None => Ok(task),
    }
}

/// Runs `cargo` with `args` and reports whether it succeeded.
fn cargo(args: &[&str]) -> bool {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    println!("$ cargo {}", args.join(" "));
    match Command::new(cargo).args(args).status() {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("could not start cargo: {error}");
            false
        }
    }
}

fn run() -> bool {
    // The compositor starts the shell, and the shell starts apps, from the
    // directory the compositor itself is in. So all three are built first.
    cargo(&[
        "build",
        "--package",
        "compositor",
        "--package",
        "shell",
        "--package",
        "about",
    ]) && cargo(&["run", "--quiet", "--package", "compositor"])
}

fn check() -> bool {
    cargo(&["fmt", "--all", "--check"])
        && cargo(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ])
        && cargo(&["test", "--workspace", "--locked"])
}

fn main() -> ExitCode {
    let ok = match parse(env::args().skip(1)) {
        Ok(Task::Run) => run(),
        Ok(Task::Check) => check(),
        Ok(Task::Help) => {
            print!("{USAGE}");
            true
        }
        Err(message) => {
            eprintln!("error: {message}\n\n{USAGE}");
            false
        }
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn known_commands_parse() {
        assert_eq!(parse(args(&["run"])), Ok(Task::Run));
        assert_eq!(parse(args(&["check"])), Ok(Task::Check));
    }

    #[test]
    fn no_command_shows_help() {
        assert_eq!(parse(args(&[])), Ok(Task::Help));
        assert_eq!(parse(args(&["--help"])), Ok(Task::Help));
    }

    #[test]
    fn unknown_commands_are_errors() {
        assert!(parse(args(&["flash"])).is_err());
        assert!(parse(args(&["run", "extra"])).is_err());
    }
}
