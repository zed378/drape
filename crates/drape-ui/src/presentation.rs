//! Hand-written presentation code retains an unconditional unsafe prohibition.
#![forbid(unsafe_code)]

use slint::ComponentHandle;

use crate::generated::BaselineWindow;

/// Runs the normal top-level foundation window, without desktop integration.
///
/// # Errors
/// Returns renderer/event-loop initialization or execution failures.
pub fn run() -> Result<(), slint::PlatformError> {
    BaselineWindow::new()?.run()
}
