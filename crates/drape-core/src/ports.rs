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

/// Redacted persistence failures. Missing, inaccessible and oversized inputs must
/// not all become an empty configuration. No variant contains a user pathname.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemError {
    NotFound,
    AccessDenied,
    TooLarge,
    AlreadyExists,
    Conflict,
    InvalidToken,
    Unsupported,
    Unavailable,
    Cancelled,
}

/// Owned bytes from one stable, bounded observation, paired with an opaque
/// revision. The revision detects replacement/change; it is not just a timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRead<Revision> {
    pub bytes: Vec<u8>,
    pub revision: Revision,
}

/// Select initial creation explicitly; absence never authorizes overwriting an
/// unexpected destination. Replacement preserves the expected previous bytes in
/// a newly, exclusively named backup before reporting success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigCommit<Revision> {
    CreateNew,
    Replace {
        expected_revision: Revision,
        backup_id: Guid,
    },
}

/// Namespace replacement and storage durability are distinct observations.
/// Unconfirmed durability never means that an already committed save was undone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitDurability {
    Confirmed,
    Unconfirmed(FileSystemError),
}

/// Successful namespace commit. Cleanup/pruning is a separate subsequent step;
/// its failure must not be returned as failure to commit these bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigCommitReceipt<File, Revision> {
    pub revision: Revision,
    pub backup: Option<File>,
    pub durability: CommitDurability,
}

/// A platform error alone does not establish whether replacement occurred.
/// Indeterminate results retain all evidence and prohibit pruning, automatic
/// retry and a clean-success claim until the adapter reconciles the namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigCommitOutcome<File, Revision> {
    Committed(ConfigCommitReceipt<File, Revision>),
    /// Destination and existing backups are unchanged; owned temp may remain.
    NotCommitted(FileSystemError),
    /// No outcome is inferred and no artefact may be discarded automatically.
    Indeterminate(FileSystemError),
}

/// Quarantine is also a namespace mutation; an error cannot imply that moving
/// the source failed. Unknown outcomes stop recovery and preserve all evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuarantineOutcome<File> {
    Moved(File),
    /// Current config and existing quarantine entries are unchanged.
    NotMoved(FileSystemError),
    /// No backup recovery, overwrite or automatic retry until reconciliation.
    Indeterminate(FileSystemError),
}

/// Cleanup applies only to the caller's exclusively created temporary artefact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporaryCleanup {
    Removed,
    AlreadyAbsent,
}

/// Root-confined configuration storage, selected and injected by App.
///
/// This port cannot address Desktop items, recovery journals or arbitrary paths.
/// File tokens refer only to current config and recognized app-managed backups /
/// quarantine entries. Tokens and revisions are opaque, root/adapter-bound and
/// validated on every call; native handles must never be exposed through them.
/// Existing legacy backup filenames remain discoverable without renaming them.
/// Symlinks/reparse escapes and foreign, stale or wrong-kind tokens are rejected.
///
/// All calls are synchronous on a selected persistence worker, never Slint's
/// event-loop thread. No blanket Send/Sync bound is imposed. A writer excludes
/// cooperating Drape processes using the same config root; it is not an atomic
/// compare-and-swap against arbitrary external writers. Adapters must document
/// external-mutation residual windows, detect conflicts where possible, and
/// reject unsupported stronger guarantees before mutation. The application still
/// coordinates its own single-instance lifetime. Shutdown stops admission, cancels reads and
/// precommit work, waits for in-flight native calls, resolves commit outcome,
/// cleans only owned temporary files, then releases the writer. None of this may
/// block the independent desktop-restoration path.
///
/// Cancellation is cooperative before mutation. Commit has no cancellation
/// parameter: once begun it must report its actual outcome even if the request
/// becomes stale. Cancellation after Committed never reports NotCommitted.
/// Dropping resources is best effort, not a process-kill durability guarantee.
/// Numeric byte/list limits are supplied explicitly, not silently defaulted here.
pub trait FileSystem {
    type File: Clone;
    type Revision: Clone;
    type Writer;
    type Temporary;

    /// A logical token for config.json; obtaining it does not create a file.
    fn current_config(&self) -> Self::File;

    /// Non-blocking exclusive writer acquisition; contention returns Conflict.
    fn acquire_writer(&mut self) -> Result<Self::Writer, FileSystemError>;

    /// Return all bytes from a stable revision without exceeding byte_limit.
    /// Check the bound while reading even if metadata advertised a smaller size.
    /// An oversize result preserves the file and is never a partial success.
    fn read_config(
        &self,
        file: &Self::File,
        byte_limit: std::num::NonZeroUsize,
        cancellation: &Cancellation,
    ) -> Result<ConfigRead<Self::Revision>, FileSystemError>;

    /// Enumerate only recognized managed backups, newest first with a stable
    /// adapter-defined tie-break. Bound scanning/retention before allocation;
    /// TooLarge reports incomplete enumeration, never a truncated successful list.
    fn list_backups(
        &self,
        entry_limit: std::num::NonZeroUsize,
        cancellation: &Cancellation,
    ) -> Result<Vec<Self::File>, FileSystemError>;

    /// Exclusively create an empty same-directory temporary file. A collision is
    /// AlreadyExists, never reuse/truncate. The caller supplies the unique ID.
    fn create_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary_id: Guid,
    ) -> Result<Self::Temporary, FileSystemError>;

    /// Write the complete byte slice. Failure may leave a partial owned temp but
    /// cannot touch the destination/backups. Only a complete successful write,
    /// followed by successful sync_temporary, makes a temp eligible for commit.
    /// The adapter enforces this state, including after failed/repeated writes.
    fn write_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
        bytes: &[u8],
        cancellation: &Cancellation,
    ) -> Result<(), FileSystemError>;

    /// Flush application buffers and sync the owned file. A separate failure
    /// point from write, preserving the previously committed destination.
    fn sync_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
        cancellation: &Cancellation,
    ) -> Result<(), FileSystemError>;

    /// Atomically publish a fully written/synced temp, checking writer ownership
    /// and expected revision under cooperating-writer exclusion. A separate
    /// revision check is not atomic CAS against arbitrary external editors.
    /// Replacement is allowed
    /// only after the coordinator validated the previous document. Backup-name
    /// collisions never overwrite existing backups. A successful commit consumes
    /// the temp's eligibility; replay is rejected. Unsupported guarantees must
    /// fail before mutation. Filesystem power-loss limits remain explicit.
    fn commit_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
        mode: ConfigCommit<Self::Revision>,
    ) -> ConfigCommitOutcome<Self::File, Self::Revision>;

    /// Preserve only the expected current config under a new quarantine identity,
    /// without overwriting another entry. NotMoved guarantees unchanged source;
    /// Indeterminate preserves evidence and stops recovery until reconciliation.
    /// Backup recovery cannot fall through either fault into replacing config.
    fn quarantine_current(
        &mut self,
        writer: &Self::Writer,
        expected_revision: &Self::Revision,
        quarantine_id: Guid,
    ) -> QuarantineOutcome<Self::File>;

    /// Remove only a recognized backup whose revision still matches. The
    /// coordinator invokes this solely after a confirmed successful commit and
    /// successful bounded validation/listing of the backups it will retain.
    fn remove_backup(
        &mut self,
        writer: &Self::Writer,
        backup: &Self::File,
        expected_revision: &Self::Revision,
    ) -> Result<(), FileSystemError>;

    /// Idempotent cleanup of this writer's temp only; foreign replacements must
    /// never be removed. Failed cleanup remains visible without undoing commit.
    fn cleanup_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
    ) -> Result<TemporaryCleanup, FileSystemError>;

    /// Release writer ownership after all calls/cleanup finish. Failure is
    /// reported separately from commit; adapters retain best-effort Drop cleanup.
    fn release_writer(&mut self, writer: Self::Writer) -> Result<(), FileSystemError>;
}
