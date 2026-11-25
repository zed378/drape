//! Read-only observations. Handles never escape the observation's UI-thread scope.
#![forbid(unsafe_code)]

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::ComponentHandle;
use std::{ffi::OsString, process::ExitCode};

use crate::generated::HandleProbe;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Interactive,
    Smoke,
    Help,
}

fn parse(args: &[OsString]) -> Result<Command, &'static str> {
    match args {
        [] => Ok(Command::Interactive),
        [arg] if arg == "--smoke" => Ok(Command::Smoke),
        [arg] if arg == "--help" || arg == "-h" => Ok(Command::Help),
        _ => Err("unsupported arguments; use --help"),
    }
}

fn observe(window: &slint::Window, phase: &str) -> String {
    // The provider can retain a backend owner. Keep both it and its borrowed
    // handle local; do not infer validity across hide/recreation/destruction.
    let provider = window.window_handle();
    let result = match provider.window_handle() {
        Ok(handle) => match handle.as_raw() {
            RawWindowHandle::Win32(native) => {
                format!("Win32 borrowed HWND=0x{:x}", native.hwnd.get())
            }
            _ => "non-Win32 handle; Windows host evidence unavailable".to_owned(),
        },
        Err(error) => format!("handle unavailable: {error}"),
    };
    let observation = format!(
        "{phase}: {result}; visible={}; scale={}",
        window.is_visible(),
        window.scale_factor()
    );
    println!("{observation}");
    observation
}

fn run(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()?;
    println!("R0-04 diagnostic: Slint 1.13.1; requested winit/software; no desktop attachment");
    let ui = HandleProbe::new()?;
    observe(ui.window(), "before-show");
    ui.show()?;
    ui.set_observation(observe(ui.window(), "after-show").into());

    if command == Command::Interactive {
        let weak = ui.as_weak();
        ui.on_sample(move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_observation(observe(ui.window(), "ui-sample").into());
            }
        });
        ui.on_finish(|| {
            if let Err(error) = slint::quit_event_loop() {
                eprintln!("could not request event-loop exit: {error}");
            }
        });
        slint::run_event_loop()?;
    }

    // No native registration exists. Observations acquire new local borrows.
    // Explicit hide precedes dropping the component and all backend owners.
    ui.hide()?;
    observe(ui.window(), "after-hide");
    drop(ui);
    println!("component dropped; no raw handle retained or probed after destruction");
    Ok(())
}

pub fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match parse(&args) {
        Ok(Command::Help) => {
            println!(
                "R0-04 Slint borrowed-handle diagnostic\nUsage: drape-r0-04-handle-probe [--help | --smoke]\nDefault: interactive normal top-level diagnostic window.\n--smoke: briefly show/sample/hide a real GUI window; interactive test account required.\nNo host strategy, icon mutation or Explorer manipulation is implemented."
            );
            ExitCode::SUCCESS
        }
        Ok(command) => match run(command) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("handle probe failed: {error}");
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

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn help_and_smoke_are_explicit_before_backend_startup() {
        assert_eq!(parse(&args(&[])), Ok(Command::Interactive));
        assert_eq!(parse(&args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["-h"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["--smoke"])), Ok(Command::Smoke));
    }

    #[test]
    fn native_mutation_flags_and_combined_arguments_are_rejected() {
        for values in [
            vec!["--parent"],
            vec!["--hide-icons"],
            vec!["--smoke", "--help"],
        ] {
            assert!(parse(&args(&values)).is_err());
        }
    }
}
