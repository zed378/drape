//! Deterministic test adapters; no access to real desktop state.
#![forbid(unsafe_code)]

pub mod config_fs;
pub mod config_store;

use drape_core::PlatformCapabilities;
use drape_core::domain::{IconBitmap, PathKey};
use drape_core::ports::{
    Cancellation, DesktopEntry, DesktopItemSource, DesktopLocations, DesktopSnapshot, IconProvider,
    IconRequest, IconResponse, OperationId, PathKeys, PortError,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};

/// Explicit capability data for tests; the default keeps every feature disabled.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FakePlatformCapabilities(PlatformCapabilities);

impl FakePlatformCapabilities {
    pub fn new(capabilities: PlatformCapabilities) -> Self {
        Self(capabilities)
    }

    pub fn capabilities(&self) -> PlatformCapabilities {
        self.0
    }
}

/// Keys equal to the literal path, so different spellings stay distinct.
#[derive(Debug, Default, Clone, Copy)]
pub struct FakePathKeys {
    failure: Option<PortError>,
}

impl FakePathKeys {
    pub fn failing(error: PortError) -> Self {
        Self {
            failure: Some(error),
        }
    }
}

impl PathKeys for FakePathKeys {
    fn path_key(&self, path: &OsStr) -> Result<PathKey, PortError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let literal = path
            .to_str()
            .map_or_else(|| format!("{path:?}"), str::to_owned);
        PathKey::new(literal).map_err(|_| PortError::Unavailable)
    }
}

/// Caller-controlled clock, including pre-epoch and repeated timestamps.
pub struct FixedClock(pub drape_core::TimestampMillis);

impl drape_core::Clock for FixedClock {
    fn now(&self) -> drape_core::TimestampMillis {
        self.0
    }
}

/// Finite scripted location lookups; exhausted scripts fail instead of using the OS.
pub struct FakeDesktopLocations(RefCell<VecDeque<Result<Vec<OsString>, PortError>>>);

impl FakeDesktopLocations {
    pub fn new(results: Vec<Result<Vec<OsString>, PortError>>) -> Self {
        Self(RefCell::new(results.into()))
    }
}

impl DesktopLocations for FakeDesktopLocations {
    fn desktop_locations(&self) -> Result<Vec<OsString>, PortError> {
        let result = self
            .0
            .borrow_mut()
            .pop_front()
            .unwrap_or(Err(PortError::Unavailable))?;
        if result.len() > 2 {
            return Err(PortError::LimitExceeded);
        }
        Ok(result)
    }
}

/// Finite owned snapshots with caller-controlled errors, bounds and cancellation.
pub struct FakeDesktopItemSource(RefCell<VecDeque<Result<Vec<DesktopEntry>, PortError>>>);

impl FakeDesktopItemSource {
    pub fn new(results: Vec<Result<Vec<DesktopEntry>, PortError>>) -> Self {
        Self(RefCell::new(results.into()))
    }
}

impl DesktopItemSource for FakeDesktopItemSource {
    fn snapshot(
        &self,
        operation: OperationId,
        entry_limit: usize,
        cancellation: &Cancellation,
    ) -> Result<DesktopSnapshot, PortError> {
        cancellation.check()?;
        let entries = self
            .0
            .borrow_mut()
            .pop_front()
            .unwrap_or(Err(PortError::Unavailable))?;
        if entries.len() > entry_limit {
            return Err(PortError::LimitExceeded);
        }
        Ok(DesktopSnapshot { operation, entries })
    }
}

/// Scripted owned bitmaps. No decode/native/UI API or hidden thread is involved.
pub struct FakeIconProvider(RefCell<VecDeque<Result<IconBitmap, PortError>>>);

impl FakeIconProvider {
    pub fn new(results: Vec<Result<IconBitmap, PortError>>) -> Self {
        Self(RefCell::new(results.into()))
    }
}

impl IconProvider for FakeIconProvider {
    fn load_icon(
        &self,
        request: &IconRequest,
        cancellation: &Cancellation,
    ) -> Result<IconResponse, PortError> {
        cancellation.check()?;
        if request.pixel_size == 0 || request.byte_limit == 0 || request.path.is_empty() {
            return Err(PortError::LimitExceeded);
        }
        let bitmap = self
            .0
            .borrow_mut()
            .pop_front()
            .unwrap_or(Err(PortError::Unavailable))?;
        if bitmap.pixels().len() > request.byte_limit {
            return Err(PortError::LimitExceeded);
        }
        Ok(IconResponse {
            operation: request.operation,
            item: request.item,
            bitmap,
        })
    }
}

#[cfg(test)]
mod tests {
    use drape_core::{Clock, TimestampMillis};

    #[test]
    fn clock_returns_supplied_pre_epoch_time_without_advancing() {
        let clock = super::FixedClock(TimestampMillis(-1));
        assert_eq!(clock.now(), TimestampMillis(-1));
        assert_eq!(clock.now(), TimestampMillis(-1));
    }
}
