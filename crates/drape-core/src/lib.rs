//! Platform-free domain data and adapter contracts.
#![forbid(unsafe_code)]

pub mod domain;
pub mod layout;
pub mod ports;

/// Measured platform support. Unsupported is the safe initial state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub desktop_host: bool,
    pub native_icon_visibility: bool,
    pub global_mouse_hook: bool,
    pub blur: bool,
    pub global_hotkeys: bool,
}

/// Signed milliseconds since the Unix epoch, supplied by an external adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimestampMillis(pub i64);

/// Time injection boundary; Core must never obtain wall-clock time directly.
///
/// Implementers define their thread affinity; the port does not require Send/Sync.
pub trait Clock {
    fn now(&self) -> TimestampMillis;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unverified_platform_has_no_enabled_capabilities() {
        let value = PlatformCapabilities::default();
        assert!(!value.desktop_host);
        assert!(!value.native_icon_visibility);
        assert!(!value.global_mouse_hook);
        assert!(!value.blur);
        assert!(!value.global_hotkeys);
    }
}
