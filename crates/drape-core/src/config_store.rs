//! Configuration persistence boundary. Wire parsing, migration orchestration,
//! backups and quarantine policy belong to the Infrastructure implementation.

use crate::domain::{DomainConfig, Repair};
use crate::ports::{Cancellation, CommitDurability, FileSystemError, OperationId};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigOrigin {
    Current,
    Backup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOnlyReason {
    NewerSchema(u32),
    /// Stored values cannot be written back without loss.
    UnsupportedContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigLoad {
    Loaded {
        config: DomainConfig,
        origin: ConfigOrigin,
        /// A corrupt current document was preserved before this result.
        quarantined: bool,
        repairs: Vec<Repair>,
    },
    /// The stored bytes stay untouched. `config` is a display projection, if one exists.
    ReadOnly {
        reason: ReadOnlyReason,
        config: Option<DomainConfig>,
        origin: ConfigOrigin,
        quarantined: bool,
    },
    /// Neither a current document nor a valid backup exists. The caller supplies defaults.
    Empty { quarantined: bool },
}

/// Redacted persistence failures; no variant carries a path or document content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigStoreError {
    /// No successful load describes what is currently stored.
    NotLoaded,
    /// The save derives from a load that a newer load has replaced.
    StaleBaseline,
    /// The gate or the supplied config forbids this save.
    ReadOnly,
    /// This save showed that the stored document cannot be rewritten without loss.
    DocumentReadOnly(ReadOnlyReason),
    /// An earlier indeterminate result latched this store.
    Stopped,
    FileSystem(FileSystemError),
    /// Corrupt input is still in place and nothing may replace it.
    QuarantineFailed(FileSystemError),
    /// The namespace state is unknown, so all evidence must be retained.
    Indeterminate(FileSystemError),
}

impl fmt::Display for ConfigStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotLoaded => "configuration has not been loaded",
            Self::StaleBaseline => "configuration changed since it was loaded",
            Self::ReadOnly | Self::DocumentReadOnly(_) => "configuration is read-only",
            Self::Stopped => "configuration persistence is stopped",
            Self::FileSystem(_) => "configuration file operation failed",
            Self::QuarantineFailed(_) => "corrupt configuration could not be preserved",
            Self::Indeterminate(_) => "configuration state is unknown",
        })
    }
}

impl std::error::Error for ConfigStoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSave {
    /// The new document is published. Later cleanup or pruning cannot undo that.
    Committed {
        durability: CommitDurability,
        cleanup: Option<FileSystemError>,
    },
    /// The stored document and existing backups are unchanged.
    NotCommitted(ConfigStoreError),
    /// Publication is unknown. No retry, pruning or cleanup may follow.
    Indeterminate(FileSystemError),
}

/// Decides whether a store may touch storage, based on the results it has observed.
///
/// Every `ConfigStore` applies the same gate so that implementations and fakes
/// cannot disagree about when overwriting is allowed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SaveGate {
    #[default]
    Unknown,
    /// Only saves derived from this load may proceed.
    Writable(OperationId),
    ReadOnly,
    Stopped,
}

impl SaveGate {
    /// A load may quarantine, so a stopped store refuses it as well.
    pub fn check_load(self) -> Result<(), ConfigStoreError> {
        match self {
            Self::Stopped => Err(ConfigStoreError::Stopped),
            _ => Ok(()),
        }
    }

    pub fn check_save(
        self,
        baseline: OperationId,
        config: &DomainConfig,
    ) -> Result<(), ConfigStoreError> {
        match self {
            Self::Stopped => Err(ConfigStoreError::Stopped),
            _ if config.is_read_only() => Err(ConfigStoreError::ReadOnly),
            Self::ReadOnly => Err(ConfigStoreError::ReadOnly),
            Self::Unknown => Err(ConfigStoreError::NotLoaded),
            Self::Writable(current) if current != baseline => Err(ConfigStoreError::StaleBaseline),
            Self::Writable(_) => Ok(()),
        }
    }

    /// Any failed load drops the earlier baseline; `Stopped` never clears.
    #[must_use]
    pub fn after_load(
        self,
        operation: OperationId,
        result: &Result<ConfigLoad, ConfigStoreError>,
    ) -> Self {
        if self == Self::Stopped {
            return self;
        }
        match result {
            Ok(ConfigLoad::Loaded { config, .. }) if !config.is_read_only() => {
                Self::Writable(operation)
            }
            Ok(ConfigLoad::Loaded { .. } | ConfigLoad::ReadOnly { .. }) => Self::ReadOnly,
            Ok(ConfigLoad::Empty { .. }) => Self::Writable(operation),
            Err(ConfigStoreError::Indeterminate(_) | ConfigStoreError::Stopped) => Self::Stopped,
            Err(_) => Self::Unknown,
        }
    }

    #[must_use]
    pub fn after_save(self, result: &ConfigSave) -> Self {
        if self == Self::Stopped {
            return self;
        }
        match result {
            ConfigSave::Indeterminate(_)
            | ConfigSave::NotCommitted(
                ConfigStoreError::Indeterminate(_) | ConfigStoreError::Stopped,
            ) => Self::Stopped,
            ConfigSave::NotCommitted(ConfigStoreError::DocumentReadOnly(_)) => Self::ReadOnly,
            // The destination differs from what was loaded.
            ConfigSave::NotCommitted(ConfigStoreError::FileSystem(
                FileSystemError::Conflict
                | FileSystemError::NotFound
                | FileSystemError::AlreadyExists,
            )) => Self::Unknown,
            ConfigSave::Committed { .. } | ConfigSave::NotCommitted(_) => self,
        }
    }
}

/// Synchronous persistence on one selected worker, never the UI event loop.
///
/// Implementations call `SaveGate::check_load` or `check_save` before any
/// mutation and feed each result back into the gate. `DomainConfig` is not a
/// lossless document: a store keeps the unknown wire values from its last load
/// and merges a save into them. Cancellation is observed only before mutation;
/// once publication starts, the actual outcome is returned. Debouncing belongs
/// to the caller, and a shutdown flush is a direct `save` call. Neither
/// operation may delay or gate desktop restoration. The caller issues a new
/// `OperationId` for every load and passes the one its config derives from to
/// `save`, so a delayed save cannot overwrite what a later load observed.
pub trait ConfigStore {
    fn load(
        &mut self,
        operation: OperationId,
        cancellation: &Cancellation,
    ) -> Result<ConfigLoad, ConfigStoreError>;

    fn save(
        &mut self,
        baseline: OperationId,
        config: &DomainConfig,
        cancellation: &Cancellation,
    ) -> ConfigSave;
}
