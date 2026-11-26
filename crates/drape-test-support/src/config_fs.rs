//! In-memory configuration filesystem with deterministic operation-level faults.
//!
//! This fake proves consumer behavior against the neutral contract. It does not
//! emulate native replacement races, reparse points, hydration or power loss.
//! Clones share a root and cooperating-writer lease; independently constructed
//! instances have distinct token provenance without global IDs or environment I/O.
use drape_core::domain::Guid;
use drape_core::ports::{
    Cancellation, CommitDurability, ConfigCommit, ConfigCommitOutcome, ConfigCommitReceipt,
    ConfigRead, FileSystem, FileSystemError, QuarantineOutcome, TemporaryCleanup,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::num::NonZeroUsize;
use std::rc::{Rc, Weak};

/// Fault/observation points contain no paths or bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemStage {
    AcquireWriter,
    Read,
    ListBackups,
    CreateTemporary,
    WriteTemporary,
    SyncTemporary,
    CommitTemporary,
    Quarantine,
    RemoveBackup,
    CleanupTemporary,
    ReleaseWriter,
}

/// One-shot scripted fault. The front fault waits for its matching operation;
/// unrelated operations do not consume it. Indeterminate variants choose the
/// hidden namespace result while the consumer receives only uncertainty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeFailure {
    Error(FileSystemStage, FileSystemError),
    PartialWrite {
        bytes: usize,
        error: FileSystemError,
    },
    CommitIndeterminate {
        publish: bool,
        error: FileSystemError,
    },
    QuarantineIndeterminate {
        move_file: bool,
        error: FileSystemError,
    },
    CommitDurability(FileSystemError),
}

impl FakeFailure {
    fn stage(&self) -> FileSystemStage {
        match self {
            Self::Error(stage, _) => *stage,
            Self::PartialWrite { .. } => FileSystemStage::WriteTemporary,
            Self::CommitIndeterminate { .. } | Self::CommitDurability(_) => {
                FileSystemStage::CommitTemporary
            }
            Self::QuarantineIndeterminate { .. } => FileSystemStage::Quarantine,
        }
    }
}

#[derive(Clone)]
struct Root(Rc<()>);

impl fmt::Debug for Root {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FakeRoot")
    }
}
impl PartialEq for Root {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Root {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileKind {
    Current,
    Backup(Guid),
    Quarantine(Guid),
}

/// Opaque root-bound managed-file identity; fields cannot be forged by clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeFile {
    root: Root,
    kind: FileKind,
    identity: Option<u64>,
}

/// Opaque observation revision; equality compares root identity and generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeRevision {
    root: Root,
    generation: u64,
}

/// Non-cloneable cooperating-writer lease. Drop is best effort and never deletes
/// outstanding temporary evidence or claims process-kill recovery.
pub struct FakeWriter {
    root: Root,
    generation: u64,
    state: Weak<RefCell<State>>,
}

impl fmt::Debug for FakeWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeWriter")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}
impl Drop for FakeWriter {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade()
            && let Ok(mut state) = state.try_borrow_mut()
            && state.writer == Some(self.generation)
        {
            state.writer = None;
        }
    }
}

/// Non-cloneable reference to one exclusive temporary artefact. Reusing its
/// logical name creates a different identity, so an old token cannot delete it.
#[derive(Debug)]
pub struct FakeTemporary {
    root: Root,
    writer: u64,
    id: Guid,
    identity: u64,
}

#[derive(Clone)]
struct StoredFile {
    bytes: Vec<u8>,
    revision: FakeRevision,
    order: u64,
}

struct StoredTemporary {
    writer: u64,
    identity: u64,
    bytes: Vec<u8>,
    present: bool,
    complete: bool,
    synced: bool,
}

#[derive(Default)]
struct State {
    sequence: u64,
    writer: Option<u64>,
    current: Option<StoredFile>,
    backups: BTreeMap<Guid, StoredFile>,
    quarantine: BTreeMap<Guid, StoredFile>,
    temporaries: BTreeMap<Guid, StoredTemporary>,
    stopped: bool,
    failures: VecDeque<FakeFailure>,
    operations: Vec<FileSystemStage>,
    cancel_on_acquire: Option<Rc<Cancellation>>,
}

impl State {
    fn next_generation(&mut self) -> Result<u64, FileSystemError> {
        let next = self
            .sequence
            .checked_add(1)
            .ok_or(FileSystemError::Unavailable)?;
        self.sequence = next;
        Ok(next)
    }
}

/// Shared-root fake. There is intentionally no filesystem/path/native API access.
#[derive(Clone)]
pub struct FakeFileSystem {
    root: Root,
    state: Rc<RefCell<State>>,
}

impl fmt::Debug for FakeFileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeFileSystem").finish_non_exhaustive()
    }
}

impl Default for FakeFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeFileSystem {
    pub fn new() -> Self {
        Self {
            root: Root(Rc::new(())),
            state: Rc::new(RefCell::new(State::default())),
        }
    }

    /// Fixture control, also simulating an unrelated external editor. This
    /// deliberately bypasses the cooperating lease, changes revision and does
    /// not pretend that check-then-replace implements native compare-and-swap.
    pub fn seed_current(&mut self, bytes: Vec<u8>) -> Result<FakeRevision, FileSystemError> {
        let mut state = self.state.borrow_mut();
        let generation = state.next_generation()?;
        let revision = FakeRevision {
            root: self.root.clone(),
            generation,
        };
        state.current = Some(StoredFile {
            bytes,
            revision: revision.clone(),
            order: generation,
        });
        Ok(revision)
    }

    /// Fixture control: a managed backup newer than everything seeded before it.
    pub fn seed_backup(
        &mut self,
        id: Guid,
        bytes: Vec<u8>,
    ) -> Result<FakeRevision, FileSystemError> {
        let mut state = self.state.borrow_mut();
        let generation = state.next_generation()?;
        let revision = FakeRevision {
            root: self.root.clone(),
            generation,
        };
        state.backups.insert(
            id,
            StoredFile {
                bytes,
                revision: revision.clone(),
                order: generation,
            },
        );
        Ok(revision)
    }

    /// Cancels the given request while the next writer acquisition succeeds.
    pub fn cancel_on_acquire(&self, cancellation: Rc<Cancellation>) {
        self.state.borrow_mut().cancel_on_acquire = Some(cancellation);
    }

    pub fn inject(&self, failure: FakeFailure) {
        self.state.borrow_mut().failures.push_back(failure);
    }

    /// Evidence inspection for assertions only; production consumers use ports.
    pub fn peek_current(&self) -> Option<Vec<u8>> {
        self.state
            .borrow()
            .current
            .as_ref()
            .map(|file| file.bytes.clone())
    }
    pub fn peek_backup(&self, id: Guid) -> Option<Vec<u8>> {
        self.state
            .borrow()
            .backups
            .get(&id)
            .map(|file| file.bytes.clone())
    }
    pub fn peek_quarantine(&self, id: Guid) -> Option<Vec<u8>> {
        self.state
            .borrow()
            .quarantine
            .get(&id)
            .map(|file| file.bytes.clone())
    }
    pub fn peek_temporary(&self, id: Guid) -> Option<Vec<u8>> {
        self.state
            .borrow()
            .temporaries
            .get(&id)
            .filter(|temporary| temporary.present)
            .map(|temporary| temporary.bytes.clone())
    }
    pub fn is_stopped(&self) -> bool {
        self.state.borrow().stopped
    }
    pub fn operations(&self) -> Vec<FileSystemStage> {
        self.state.borrow().operations.clone()
    }
    pub fn pending_failures(&self) -> usize {
        self.state.borrow().failures.len()
    }

    fn file(&self, kind: FileKind) -> FakeFile {
        FakeFile {
            root: self.root.clone(),
            kind,
            identity: None,
        }
    }
    fn observed_file(&self, kind: FileKind, generation: u64) -> FakeFile {
        FakeFile {
            root: self.root.clone(),
            kind,
            identity: Some(generation),
        }
    }
    fn operation(&self, stage: FileSystemStage) -> Option<FakeFailure> {
        let mut state = self.state.borrow_mut();
        state.operations.push(stage);
        if state
            .failures
            .front()
            .is_some_and(|failure| failure.stage() == stage)
        {
            state.failures.pop_front()
        } else {
            None
        }
    }
    fn error_at(&self, stage: FileSystemStage) -> Result<(), FileSystemError> {
        match self.operation(stage) {
            None => Ok(()),
            Some(FakeFailure::Error(_, error)) => Err(error),
            Some(_) => Err(FileSystemError::Unsupported),
        }
    }
    fn check_writer(
        &self,
        writer: &FakeWriter,
        permit_stopped: bool,
    ) -> Result<(), FileSystemError> {
        if writer.root != self.root || !Weak::ptr_eq(&writer.state, &Rc::downgrade(&self.state)) {
            return Err(FileSystemError::InvalidToken);
        }
        let state = self.state.borrow();
        if state.writer != Some(writer.generation) {
            return Err(FileSystemError::InvalidToken);
        }
        if state.stopped && !permit_stopped {
            return Err(FileSystemError::Conflict);
        }
        Ok(())
    }
    fn check_temporary(
        &self,
        writer: &FakeWriter,
        temporary: &FakeTemporary,
    ) -> Result<(), FileSystemError> {
        self.check_writer(writer, false)?;
        if temporary.root != self.root || temporary.writer != writer.generation {
            return Err(FileSystemError::InvalidToken);
        }
        let state = self.state.borrow();
        let stored = state
            .temporaries
            .get(&temporary.id)
            .ok_or(FileSystemError::InvalidToken)?;
        if stored.identity != temporary.identity || stored.writer != writer.generation {
            return Err(FileSystemError::InvalidToken);
        }
        Ok(())
    }
    fn check_revision(&self, revision: &FakeRevision) -> Result<(), FileSystemError> {
        if revision.root == self.root {
            Ok(())
        } else {
            Err(FileSystemError::InvalidToken)
        }
    }
    fn cancellation(cancellation: &Cancellation) -> Result<(), FileSystemError> {
        if cancellation.is_cancelled() {
            Err(FileSystemError::Cancelled)
        } else {
            Ok(())
        }
    }

    fn publish(
        &self,
        temporary: &FakeTemporary,
        mode: &ConfigCommit<FakeRevision>,
        preserve_temporary: bool,
        durability: CommitDurability,
    ) -> Result<ConfigCommitReceipt<FakeFile, FakeRevision>, FileSystemError> {
        let mut state = self.state.borrow_mut();
        let bytes = state
            .temporaries
            .get(&temporary.id)
            .ok_or(FileSystemError::InvalidToken)?
            .bytes
            .clone();
        let generation = state.next_generation()?;
        let revision = FakeRevision {
            root: self.root.clone(),
            generation,
        };
        let backup = match mode {
            ConfigCommit::CreateNew => None,
            ConfigCommit::Replace { backup_id, .. } => {
                let mut previous = state.current.take().ok_or(FileSystemError::NotFound)?;
                previous.order = generation;
                let previous_identity = previous.revision.generation;
                state.backups.insert(*backup_id, previous);
                Some(self.observed_file(FileKind::Backup(*backup_id), previous_identity))
            }
        };
        state.current = Some(StoredFile {
            bytes,
            revision: revision.clone(),
            order: generation,
        });
        // The token was checked immediately before this single-threaded operation.
        if let Some(stored) = state.temporaries.get_mut(&temporary.id) {
            stored.complete = false;
            stored.synced = false;
            if !preserve_temporary {
                stored.present = false;
                stored.bytes.clear();
            }
        }
        Ok(ConfigCommitReceipt {
            revision,
            backup,
            durability,
        })
    }

    fn commit_checked(
        &self,
        writer: &FakeWriter,
        temporary: &FakeTemporary,
        mode: ConfigCommit<FakeRevision>,
    ) -> Result<ConfigCommitOutcome<FakeFile, FakeRevision>, FileSystemError> {
        self.check_temporary(writer, temporary)?;
        {
            let state = self.state.borrow();
            let stored = state
                .temporaries
                .get(&temporary.id)
                .ok_or(FileSystemError::InvalidToken)?;
            if !stored.present || !stored.complete || !stored.synced {
                return Err(FileSystemError::Conflict);
            }
            match &mode {
                ConfigCommit::CreateNew if state.current.is_some() => {
                    return Err(FileSystemError::AlreadyExists);
                }
                ConfigCommit::CreateNew => {}
                ConfigCommit::Replace {
                    expected_revision,
                    backup_id,
                } => {
                    self.check_revision(expected_revision)?;
                    let previous = state.current.as_ref().ok_or(FileSystemError::NotFound)?;
                    if &previous.revision != expected_revision {
                        return Err(FileSystemError::Conflict);
                    }
                    if state.backups.contains_key(backup_id) {
                        return Err(FileSystemError::AlreadyExists);
                    }
                }
            }
        }
        match self.operation(FileSystemStage::CommitTemporary) {
            Some(FakeFailure::Error(_, error)) => Err(error),
            Some(FakeFailure::CommitIndeterminate { publish, error }) => {
                if publish {
                    self.publish(temporary, &mode, true, CommitDurability::Unconfirmed(error))?;
                }
                self.state.borrow_mut().stopped = true;
                Ok(ConfigCommitOutcome::Indeterminate(error))
            }
            Some(FakeFailure::CommitDurability(error)) => {
                Ok(ConfigCommitOutcome::Committed(self.publish(
                    temporary,
                    &mode,
                    false,
                    CommitDurability::Unconfirmed(error),
                )?))
            }
            None => Ok(ConfigCommitOutcome::Committed(self.publish(
                temporary,
                &mode,
                false,
                CommitDurability::Confirmed,
            )?)),
            Some(_) => Err(FileSystemError::Unsupported),
        }
    }

    fn quarantine_checked(
        &self,
        writer: &FakeWriter,
        expected_revision: &FakeRevision,
        quarantine_id: Guid,
    ) -> Result<QuarantineOutcome<FakeFile>, FileSystemError> {
        self.check_writer(writer, false)?;
        self.check_revision(expected_revision)?;
        {
            let state = self.state.borrow();
            let current = state.current.as_ref().ok_or(FileSystemError::NotFound)?;
            if &current.revision != expected_revision {
                return Err(FileSystemError::Conflict);
            }
            if state.quarantine.contains_key(&quarantine_id) {
                return Err(FileSystemError::AlreadyExists);
            }
        }
        let failure = self.operation(FileSystemStage::Quarantine);
        let (move_file, indeterminate) = match failure {
            Some(FakeFailure::Error(_, error)) => return Err(error),
            Some(FakeFailure::QuarantineIndeterminate { move_file, error }) => {
                (move_file, Some(error))
            }
            None => (true, None),
            Some(_) => return Err(FileSystemError::Unsupported),
        };
        let mut state = self.state.borrow_mut();
        if move_file {
            let current = state.current.take().ok_or(FileSystemError::NotFound)?;
            state.quarantine.insert(quarantine_id, current);
        }
        if let Some(error) = indeterminate {
            state.stopped = true;
            Ok(QuarantineOutcome::Indeterminate(error))
        } else {
            Ok(QuarantineOutcome::Moved(self.observed_file(
                FileKind::Quarantine(quarantine_id),
                expected_revision.generation,
            )))
        }
    }
}

impl FileSystem for FakeFileSystem {
    type File = FakeFile;
    type Revision = FakeRevision;
    type Writer = FakeWriter;
    type Temporary = FakeTemporary;

    fn current_config(&self) -> Self::File {
        self.file(FileKind::Current)
    }
    fn acquire_writer(&mut self) -> Result<Self::Writer, FileSystemError> {
        {
            let state = self.state.borrow();
            if state.writer.is_some() || state.stopped {
                return Err(FileSystemError::Conflict);
            }
        }
        self.error_at(FileSystemStage::AcquireWriter)?;
        let mut state = self.state.borrow_mut();
        let generation = state.next_generation()?;
        state.writer = Some(generation);
        if let Some(cancellation) = state.cancel_on_acquire.take() {
            cancellation.cancel();
        }
        Ok(FakeWriter {
            root: self.root.clone(),
            generation,
            state: Rc::downgrade(&self.state),
        })
    }
    fn read_config(
        &self,
        file: &Self::File,
        byte_limit: NonZeroUsize,
        cancellation: &Cancellation,
    ) -> Result<ConfigRead<Self::Revision>, FileSystemError> {
        Self::cancellation(cancellation)?;
        if file.root != self.root {
            return Err(FileSystemError::InvalidToken);
        }
        self.error_at(FileSystemStage::Read)?;
        let state = self.state.borrow();
        let stored = match file.kind {
            FileKind::Current => state.current.as_ref(),
            FileKind::Backup(id) => state.backups.get(&id),
            FileKind::Quarantine(id) => state.quarantine.get(&id),
        }
        .ok_or(FileSystemError::NotFound)?;
        if file.kind != FileKind::Current && file.identity != Some(stored.revision.generation) {
            return Err(FileSystemError::InvalidToken);
        }
        if stored.bytes.len() > byte_limit.get() {
            return Err(FileSystemError::TooLarge);
        }
        Ok(ConfigRead {
            bytes: stored.bytes.clone(),
            revision: stored.revision.clone(),
        })
    }
    fn list_backups(
        &self,
        entry_limit: NonZeroUsize,
        cancellation: &Cancellation,
    ) -> Result<Vec<Self::File>, FileSystemError> {
        Self::cancellation(cancellation)?;
        self.error_at(FileSystemStage::ListBackups)?;
        let state = self.state.borrow();
        if state.backups.len() > entry_limit.get() {
            return Err(FileSystemError::TooLarge);
        }
        let mut entries: Vec<_> = state.backups.iter().collect();
        entries.sort_by(|(left_id, left), (right_id, right)| {
            right
                .order
                .cmp(&left.order)
                .then_with(|| left_id.cmp(right_id))
        });
        Ok(entries
            .into_iter()
            .map(|(id, stored)| {
                self.observed_file(FileKind::Backup(*id), stored.revision.generation)
            })
            .collect())
    }
    fn create_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary_id: Guid,
    ) -> Result<Self::Temporary, FileSystemError> {
        self.check_writer(writer, false)?;
        if self
            .state
            .borrow()
            .temporaries
            .get(&temporary_id)
            .is_some_and(|stored| stored.present)
        {
            return Err(FileSystemError::AlreadyExists);
        }
        self.error_at(FileSystemStage::CreateTemporary)?;
        let mut state = self.state.borrow_mut();
        let identity = state.next_generation()?;
        state.temporaries.insert(
            temporary_id,
            StoredTemporary {
                writer: writer.generation,
                identity,
                bytes: Vec::new(),
                present: true,
                complete: false,
                synced: false,
            },
        );
        Ok(FakeTemporary {
            root: self.root.clone(),
            writer: writer.generation,
            id: temporary_id,
            identity,
        })
    }
    fn write_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
        bytes: &[u8],
        cancellation: &Cancellation,
    ) -> Result<(), FileSystemError> {
        Self::cancellation(cancellation)?;
        self.check_temporary(writer, temporary)?;
        {
            let mut state = self.state.borrow_mut();
            let stored = state
                .temporaries
                .get_mut(&temporary.id)
                .ok_or(FileSystemError::InvalidToken)?;
            if !stored.present {
                return Err(FileSystemError::Conflict);
            }
            stored.complete = false;
            stored.synced = false;
        }
        let failure = self.operation(FileSystemStage::WriteTemporary);
        match failure {
            Some(FakeFailure::Error(_, error)) => Err(error),
            Some(FakeFailure::PartialWrite {
                bytes: count,
                error,
            }) => {
                let mut state = self.state.borrow_mut();
                let stored = state
                    .temporaries
                    .get_mut(&temporary.id)
                    .ok_or(FileSystemError::InvalidToken)?;
                stored.bytes = bytes[..count.min(bytes.len())].to_vec();
                Err(error)
            }
            None => {
                let mut state = self.state.borrow_mut();
                let stored = state
                    .temporaries
                    .get_mut(&temporary.id)
                    .ok_or(FileSystemError::InvalidToken)?;
                stored.bytes = bytes.to_vec();
                stored.complete = true;
                Ok(())
            }
            Some(_) => Err(FileSystemError::Unsupported),
        }
    }
    fn sync_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
        cancellation: &Cancellation,
    ) -> Result<(), FileSystemError> {
        Self::cancellation(cancellation)?;
        self.check_temporary(writer, temporary)?;
        {
            let mut state = self.state.borrow_mut();
            let stored = state
                .temporaries
                .get_mut(&temporary.id)
                .ok_or(FileSystemError::InvalidToken)?;
            stored.synced = false;
            if !stored.present || !stored.complete {
                return Err(FileSystemError::Conflict);
            }
        }
        self.error_at(FileSystemStage::SyncTemporary)?;
        let mut state = self.state.borrow_mut();
        let stored = state
            .temporaries
            .get_mut(&temporary.id)
            .ok_or(FileSystemError::InvalidToken)?;
        stored.synced = true;
        Ok(())
    }
    fn commit_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
        mode: ConfigCommit<Self::Revision>,
    ) -> ConfigCommitOutcome<Self::File, Self::Revision> {
        match self.commit_checked(writer, temporary, mode) {
            Ok(outcome) => outcome,
            Err(error) => ConfigCommitOutcome::NotCommitted(error),
        }
    }
    fn quarantine_current(
        &mut self,
        writer: &Self::Writer,
        expected_revision: &Self::Revision,
        quarantine_id: Guid,
    ) -> QuarantineOutcome<Self::File> {
        match self.quarantine_checked(writer, expected_revision, quarantine_id) {
            Ok(outcome) => outcome,
            Err(error) => QuarantineOutcome::NotMoved(error),
        }
    }
    fn remove_backup(
        &mut self,
        writer: &Self::Writer,
        backup: &Self::File,
        expected_revision: &Self::Revision,
    ) -> Result<(), FileSystemError> {
        self.check_writer(writer, false)?;
        self.check_revision(expected_revision)?;
        if backup.root != self.root {
            return Err(FileSystemError::InvalidToken);
        }
        let FileKind::Backup(id) = backup.kind else {
            return Err(FileSystemError::InvalidToken);
        };
        {
            let state = self.state.borrow();
            let stored = state.backups.get(&id).ok_or(FileSystemError::NotFound)?;
            if backup.identity != Some(stored.revision.generation) {
                return Err(FileSystemError::InvalidToken);
            }
            if &stored.revision != expected_revision {
                return Err(FileSystemError::Conflict);
            }
        }
        self.error_at(FileSystemStage::RemoveBackup)?;
        self.state.borrow_mut().backups.remove(&id);
        Ok(())
    }
    fn cleanup_temporary(
        &mut self,
        writer: &Self::Writer,
        temporary: &mut Self::Temporary,
    ) -> Result<TemporaryCleanup, FileSystemError> {
        self.check_temporary(writer, temporary)?;
        self.error_at(FileSystemStage::CleanupTemporary)?;
        let mut state = self.state.borrow_mut();
        let stored = state
            .temporaries
            .get_mut(&temporary.id)
            .ok_or(FileSystemError::InvalidToken)?;
        if !stored.present {
            return Ok(TemporaryCleanup::AlreadyAbsent);
        }
        stored.present = false;
        stored.complete = false;
        stored.synced = false;
        stored.bytes.clear();
        Ok(TemporaryCleanup::Removed)
    }
    fn release_writer(&mut self, writer: Self::Writer) -> Result<(), FileSystemError> {
        self.check_writer(&writer, true)?;
        // Even when explicit release fails, the consumed token's Drop attempts
        // release; this is separate from whether namespace commitment succeeded.
        self.error_at(FileSystemStage::ReleaseWriter)?;
        self.state.borrow_mut().writer = None;
        Ok(())
    }
}
