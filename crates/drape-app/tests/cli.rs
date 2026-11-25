//! Run only renderer-independent routes; never touch actual desktop state.

use std::process::Command;

#[test]
fn restore_reports_unavailable_without_starting_ui() {
    let result = Command::new(env!("CARGO_BIN_EXE_drape-app"))
        // Renderer initialization must not occur on this route.
        .env("SLINT_BACKEND", "drape-invalid-backend")
        .arg("--restore-desktop")
        .output()
        .expect("baseline executable starts");
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("native recovery is not implemented"));
    assert!(!String::from_utf8_lossy(&result.stderr).contains("UI could not start"));
    assert!(result.stdout.is_empty());
}

#[test]
fn help_exits_successfully_and_unknown_argument_fails() {
    let help = Command::new(env!("CARGO_BIN_EXE_drape-app"))
        .env("SLINT_BACKEND", "drape-invalid-backend")
        .arg("--help")
        .output()
        .expect("help starts");
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--restore-desktop"));
    let unknown = Command::new(env!("CARGO_BIN_EXE_drape-app"))
        .arg("--unknown")
        .output()
        .expect("argument validation starts");
    assert_eq!(unknown.status.code(), Some(2));
}
