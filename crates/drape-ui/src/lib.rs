//! Compiled Slint baseline; native handles never cross this public boundary.
#![deny(unsafe_code)]

// ADR-028: only compiler-generated toolkit glue may use unsafe internals.
#[allow(unsafe_code)]
mod generated {
    slint::include_modules!();
}

mod presentation;
pub use presentation::run;
