//! Windows adapter boundary. Native mutations remain disabled until the safety spike.

/// Only experimentally verified support may be reported by this adapter.
pub fn capabilities() -> drape_core::PlatformCapabilities {
    drape_core::PlatformCapabilities::default()
}

/// Minimal renderer-independent entry point reserved for R0-05.
#[cfg(feature = "recovery")]
pub mod recovery {
    /// No readiness, mutation or recovery is claimed by the baseline.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RecoveryUnavailable;

    impl std::fmt::Display for RecoveryUnavailable {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(
                "native recovery is not implemented (R0-05); no desktop state was changed",
            )
        }
    }

    impl std::error::Error for RecoveryUnavailable {}

    /// Fails explicitly until coordinated ownership and verified restoration exist.
    ///
    /// # Errors
    /// Always returns `RecoveryUnavailable` in the foundation baseline.
    pub fn restore_desktop() -> Result<(), RecoveryUnavailable> {
        Err(RecoveryUnavailable)
    }
}
