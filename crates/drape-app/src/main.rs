//! Foundation composition root. CLI routing precedes UI/renderer initialization.
#![forbid(unsafe_code)]

use std::process::ExitCode;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Run,
    Help,
    Restore,
}

fn parse_command(args: &[std::ffi::OsString]) -> Result<Command, &'static str> {
    match args {
        [] => Ok(Command::Run),
        [arg] if arg == "--help" || arg == "-h" => Ok(Command::Help),
        [arg] if arg == "--restore-desktop" => Ok(Command::Restore),
        _ => Err("unsupported arguments; use --help"),
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match parse_command(&args) {
        Ok(Command::Help) => {
            println!(
                "Drape Rust foundation\nUsage: drape-app [--help | --restore-desktop]\nNative recovery is unavailable until R0-05."
            );
            ExitCode::SUCCESS
        }
        Ok(Command::Restore) => match drape_shell_windows::recovery::restore_desktop() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(2)
            }
        },
        Ok(Command::Run) => match drape_ui::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Drape UI could not start: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, &'static str> {
        parse_command(
            &args
                .iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn restore_is_selected_before_ui_startup() {
        assert_eq!(parse(&["--restore-desktop"]), Ok(Command::Restore));
    }

    #[test]
    fn help_and_default_route_are_explicit() {
        assert_eq!(parse(&["--help"]), Ok(Command::Help));
        assert_eq!(parse(&["-h"]), Ok(Command::Help));
        assert_eq!(parse(&[]), Ok(Command::Run));
    }

    #[test]
    fn unknown_and_combined_arguments_fail_closed() {
        assert!(parse(&["--unknown"]).is_err());
        assert!(parse(&["--restore-desktop", "--help"]).is_err());
    }
}
