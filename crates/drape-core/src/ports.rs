//! Synchronous adapter boundaries. No blanket Send/Sync requirement is imposed.
//!
//! Composition chooses an adapter's documented thread/apartment; calls must never
//! run on the Slint event loop when they can block. Providers enforce caller work
//! limits, return owned data, and report unsupported capabilities explicitly.

use crate::domain::{Guid, IconBitmap, ItemKind, PathKey};
use std::cell::Cell;
use std::ffi::OsString;
use std::fmt;

/// Caller-issued session/request identity. Consumers must discard stale responses.
///
/// This is not an authentication token or native recovery ownership generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId(pub u64);

/// Cooperative cancellation on the adapter's call thread.
///
/// Cancellation means the result is no longer wanted, not that native execution
/// stopped. Native owners/resources must remain alive until the call returns.
/// An asynchronous dispatcher must marshal cancellation to that same thread.
#[derive(Debug, Default)]
pub struct Cancellation(Cell<bool>);

impl Cancellation {
    pub fn cancel(&self) {
        self.0.set(true);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.get()
    }
    pub fn check(&self) -> Result<(), PortError> {
        if self.is_cancelled() {
            Err(PortError::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// Redacted failures; platform diagnostics belong to the adapter, not user path text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortError {
    Unsupported,
    Unavailable,
    Cancelled,
    LimitExceeded,
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unsupported => "operation unsupported",
            Self::Unavailable => "adapter unavailable",
            Self::Cancelled => "request cancelled",
            Self::LimitExceeded => "work limit exceeded",
        })
    }
}
impl std::error::Error for PortError {}

/// Known Folder locations, including redirected/private and public desktop roots.
///
/// Thread/apartment: chosen adapter thread, never an implicit background executor.
/// Returns at most two owned literal roots; no directory traversal or hydration.
/// Unsupported platforms return Unsupported. No host/icon-visibility permission
/// is implied. The synchronous bounded lookup completes before adapter shutdown.
pub trait DesktopLocations {
    fn desktop_locations(&self) -> Result<Vec<OsString>, PortError>;
}

/// Lossless literal filesystem observation; keys originate in the platform adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    pub path: OsString,
    pub key: PathKey,
    pub kind: ItemKind,
    /// An opaque tracker only; Core never merges entries solely by this value.
    pub file_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSnapshot {
    pub operation: OperationId,
    pub entries: Vec<DesktopEntry>,
}

/// Read-only bounded enumeration through a caller-selected adapter/worker thread.
///
/// No native-icon hiding/host capability is required or granted. Providers return
/// Unsupported if enumeration is unavailable. Enforce entry_limit before growing
/// the result and return LimitExceeded instead of silently truncating. Observe
/// cancellation between bounded batches; never request cloud-file hydration.
/// Shutdown cancels work, waits for completion and then releases native resources.
/// Consumers match operation with their current request before applying results.
pub trait DesktopItemSource {
    fn snapshot(
        &self,
        operation: OperationId,
        entry_limit: usize,
        cancellation: &Cancellation,
    ) -> Result<DesktopSnapshot, PortError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconRequest {
    pub operation: OperationId,
    pub item: Guid,
    pub path: OsString,
    pub pixel_size: u32,
    pub byte_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconResponse {
    pub operation: OperationId,
    pub item: Guid,
    pub bitmap: IconBitmap,
}

/// Own pixels across the native/UI boundary, never handles, COM objects or UI images.
///
/// Thread/apartment: the provider's documented icon worker apartment, never Slint.
/// Capability: return Unsupported when icon extraction is unavailable; this port
/// grants no desktop mutation. Validate nonzero pixel size and byte_limit before
/// decoding; call IconBitmap::required_bytes before allocation and honor the
/// supplied total bytes. Do not hydrate cloud files. Check cancellation before and
/// after native work; cancellation does not authorize releasing live COM owners.
/// Shutdown waits for native completion; callers reject stale operation/item pairs
/// and convert pixels to UI image data only on the UI event-loop thread.
pub trait IconProvider {
    fn load_icon(
        &self,
        request: &IconRequest,
        cancellation: &Cancellation,
    ) -> Result<IconResponse, PortError>;
}
