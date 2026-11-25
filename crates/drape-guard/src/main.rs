//! Deliberately cannot announce readiness until the R0-05 protocol exists.
#![forbid(unsafe_code)]

fn main() -> std::process::ExitCode {
    eprintln!(
        "Drape Guard is unavailable until R0-05; no readiness acknowledged and no desktop state changed"
    );
    std::process::ExitCode::from(2)
}
