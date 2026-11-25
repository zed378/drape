//! Isolated R0-04 diagnostic. Only compiler-generated glue may contain unsafe.
#![deny(unsafe_code)]

#[allow(unsafe_code)]
mod generated {
    slint::include_modules!();
}

mod probe;

fn main() -> std::process::ExitCode {
    probe::main()
}
