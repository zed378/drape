use super::{Class, Document, MAX_BACKUPS, MAX_DOCUMENT_BYTES, RETAINED_BACKUPS, read, write};
use drape_core::config_store::{
    ConfigLoad, ConfigOrigin, ConfigSave, ConfigStore, ConfigStoreError, ReadOnlyReason, SaveGate,
};
use drape_core::domain::{DomainConfig, Guid, Page};
use drape_core::ports::{
    Cancellation, CommitDurability, ConfigCommit, ConfigCommitOutcome, FileSystem, FileSystemError,
    OperationId, PathKeys, PortError, QuarantineOutcome,
};
use serde_json::Value;
use std::num::NonZeroUsize;

pub trait GuidSource {
    fn next_guid(&mut self) -> Result<Guid, PortError>;
}

pub struct JsonConfigStore<F: FileSystem, K, G> {
    files: F,
    keys: K,
    ids: G,
    fallback_page: Page,
    gate: SaveGate,
    baseline: Option<(Value, Option<F::Revision>)>,
    quarantine_notice: bool,
}

impl<F: FileSystem, K, G> JsonConfigStore<F, K, G> {
    pub fn new(files: F, keys: K, ids: G, fallback_page: Page) -> Self {
        Self {
            files,
            keys,
            ids,
            fallback_page,
            gate: SaveGate::default(),
            baseline: None,
            quarantine_notice: false,
        }
    }
}

fn limit(value: usize) -> Result<NonZeroUsize, FileSystemError> {
    NonZeroUsize::new(value).ok_or(FileSystemError::TooLarge)
}

fn check_cancellation(cancellation: &Cancellation) -> Result<(), FileSystemError> {
    if cancellation.is_cancelled() {
        Err(FileSystemError::Cancelled)
    } else {
        Ok(())
    }
}

impl<F: FileSystem, K: PathKeys, G: GuidSource> JsonConfigStore<F, K, G> {
    fn classify(&self, bytes: &[u8]) -> Result<Class, FileSystemError> {
        read::classify(bytes, &self.keys, &self.fallback_page)
            .map_err(|_| FileSystemError::Unavailable)
    }

    fn next_id(&mut self) -> Result<Guid, ConfigStoreError> {
        self.ids
            .next_guid()
            .map_err(|_| ConfigStoreError::FileSystem(FileSystemError::Unavailable))
    }

    fn accept(
        &mut self,
        class: Class,
        revision: Option<F::Revision>,
        origin: ConfigOrigin,
    ) -> Result<Option<ConfigLoad>, ConfigStoreError> {
        let quarantined = self.quarantine_notice;
        Ok(Some(match class {
            Class::Valid(document) => {
                let Document {
                    value,
                    config,
                    repairs,
                } = *document;
                self.baseline = Some((value, revision));
                ConfigLoad::Loaded {
                    config,
                    origin,
                    quarantined,
                    repairs,
                }
            }
            Class::Unsupported(config) => ConfigLoad::ReadOnly {
                reason: ReadOnlyReason::UnsupportedContent,
                config: config.map(|config| *config),
                origin,
                quarantined,
            },
            Class::Newer(version) => ConfigLoad::ReadOnly {
                reason: ReadOnlyReason::NewerSchema(version),
                config: None,
                origin,
                quarantined,
            },
            Class::Corrupt => return Ok(None),
            Class::TooLarge => {
                return Err(ConfigStoreError::FileSystem(FileSystemError::TooLarge));
            }
        }))
    }

    fn recover(&mut self, cancellation: &Cancellation) -> Result<ConfigLoad, ConfigStoreError> {
        let backups = self
            .files
            .list_backups(
                limit(MAX_BACKUPS).map_err(ConfigStoreError::FileSystem)?,
                cancellation,
            )
            .map_err(ConfigStoreError::FileSystem)?;
        let byte_limit = limit(MAX_DOCUMENT_BYTES).map_err(ConfigStoreError::FileSystem)?;
        for backup in backups {
            let document = self
                .files
                .read_config(&backup, byte_limit, cancellation)
                .map_err(ConfigStoreError::FileSystem)?;
            let class = self
                .classify(&document.bytes)
                .map_err(ConfigStoreError::FileSystem)?;
            if let Some(loaded) = self.accept(class, None, ConfigOrigin::Backup)? {
                return Ok(loaded);
            }
        }
        self.baseline = Some((Value::Object(serde_json::Map::new()), None));
        Ok(ConfigLoad::Empty {
            quarantined: self.quarantine_notice,
        })
    }

    fn run_load(&mut self, cancellation: &Cancellation) -> Result<ConfigLoad, ConfigStoreError> {
        check_cancellation(cancellation).map_err(ConfigStoreError::FileSystem)?;
        let document = match self.files.read_config(
            &self.files.current_config(),
            limit(MAX_DOCUMENT_BYTES).map_err(ConfigStoreError::FileSystem)?,
            cancellation,
        ) {
            Ok(document) => document,
            Err(FileSystemError::NotFound) => return self.recover(cancellation),
            Err(error) => return Err(ConfigStoreError::FileSystem(error)),
        };
        let class = self
            .classify(&document.bytes)
            .map_err(ConfigStoreError::FileSystem)?;
        if let Some(loaded) = self.accept(
            class,
            Some(document.revision.clone()),
            ConfigOrigin::Current,
        )? {
            return Ok(loaded);
        }
        check_cancellation(cancellation).map_err(ConfigStoreError::FileSystem)?;
        let id = self.next_id()?;
        let writer = self
            .files
            .acquire_writer()
            .map_err(ConfigStoreError::QuarantineFailed)?;
        if let Err(error) = check_cancellation(cancellation) {
            let _ = self.files.release_writer(writer);
            return Err(ConfigStoreError::FileSystem(error));
        }
        let outcome = self
            .files
            .quarantine_current(&writer, &document.revision, id);
        let release = self.files.release_writer(writer);
        match outcome {
            QuarantineOutcome::Moved(_) => {
                self.quarantine_notice = true;
                release.map_err(ConfigStoreError::QuarantineFailed)?;
                self.recover(cancellation)
            }
            QuarantineOutcome::NotMoved(error) => Err(ConfigStoreError::QuarantineFailed(error)),
            QuarantineOutcome::Indeterminate(error) => Err(ConfigStoreError::Indeterminate(error)),
        }
    }

    fn publish(
        &mut self,
        writer: &F::Writer,
        temporary: &mut F::Temporary,
        bytes: &[u8],
        mode: ConfigCommit<F::Revision>,
        cancellation: &Cancellation,
    ) -> ConfigCommitOutcome<F::File, F::Revision> {
        let prepared = self
            .files
            .write_temporary(writer, temporary, bytes, cancellation)
            .and_then(|()| self.files.sync_temporary(writer, temporary, cancellation))
            .and_then(|()| check_cancellation(cancellation));
        match prepared {
            Ok(()) => self.files.commit_temporary(writer, temporary, mode),
            Err(error) => ConfigCommitOutcome::NotCommitted(error),
        }
    }

    fn run_save(
        &mut self,
        config: &DomainConfig,
        cancellation: &Cancellation,
    ) -> Result<ConfigSave, ConfigStoreError> {
        check_cancellation(cancellation).map_err(ConfigStoreError::FileSystem)?;
        let (base, revision) = self.baseline.as_ref().ok_or(ConfigStoreError::NotLoaded)?;
        let value = write::merge(base, config).map_err(|_| ConfigStoreError::ReadOnly)?;
        let unsupported = ConfigStoreError::DocumentReadOnly(ReadOnlyReason::UnsupportedContent);
        let bytes = serde_json::to_vec_pretty(&value).map_err(|_| unsupported)?;
        let verified = match self
            .classify(&bytes)
            .map_err(ConfigStoreError::FileSystem)?
        {
            Class::Valid(document) if document.config == *config && document.repairs.is_empty() => {
                document.value
            }
            Class::TooLarge => {
                return Err(ConfigStoreError::FileSystem(FileSystemError::TooLarge));
            }
            _ => return Err(unsupported),
        };
        let revision = revision.clone();
        let temporary_id = self.next_id()?;
        let backup_id = self.next_id()?;
        let mode = match revision {
            Some(expected_revision) => ConfigCommit::Replace {
                expected_revision,
                backup_id,
            },
            None => ConfigCommit::CreateNew,
        };
        let writer = self
            .files
            .acquire_writer()
            .map_err(ConfigStoreError::FileSystem)?;
        let mut outcome = (|| {
            let mut temporary = match self.files.create_temporary(&writer, temporary_id) {
                Ok(temporary) => temporary,
                Err(error) => {
                    return ConfigSave::NotCommitted(ConfigStoreError::FileSystem(error));
                }
            };
            match self.publish(&writer, &mut temporary, &bytes, mode, cancellation) {
                ConfigCommitOutcome::NotCommitted(error) => {
                    let _ = self.files.cleanup_temporary(&writer, &mut temporary);
                    ConfigSave::NotCommitted(ConfigStoreError::FileSystem(error))
                }
                ConfigCommitOutcome::Indeterminate(error) => ConfigSave::Indeterminate(error),
                ConfigCommitOutcome::Committed(receipt) => {
                    self.baseline = Some((verified, Some(receipt.revision)));
                    let cleanup = if receipt.durability == CommitDurability::Confirmed {
                        self.prune(&writer).err()
                    } else {
                        None
                    };
                    ConfigSave::Committed {
                        durability: receipt.durability,
                        cleanup,
                    }
                }
            }
        })();
        let release_error = self.files.release_writer(writer).err();
        if let ConfigSave::Committed { cleanup, .. } = &mut outcome {
            *cleanup = cleanup.or(release_error);
        }
        Ok(outcome)
    }

    fn prune(&mut self, writer: &F::Writer) -> Result<(), FileSystemError> {
        let cancellation = Cancellation::default();
        let backups = self
            .files
            .list_backups(limit(MAX_BACKUPS)?, &cancellation)?;
        let byte_limit = limit(MAX_DOCUMENT_BYTES)?;
        let retained = limit(RETAINED_BACKUPS)?.get();
        let mut valid = 0;
        let mut removals = Vec::new();
        for backup in backups {
            let document = self.files.read_config(&backup, byte_limit, &cancellation)?;
            let remove = match self.classify(&document.bytes)? {
                Class::Valid(_) => {
                    valid += 1;
                    valid > retained
                }
                Class::Corrupt => valid >= retained,
                Class::Unsupported(_) | Class::Newer(_) => false,
                Class::TooLarge => return Err(FileSystemError::TooLarge),
            };
            if remove {
                removals.push((backup, document.revision));
            }
        }
        for (backup, revision) in removals {
            self.files.remove_backup(writer, &backup, &revision)?;
        }
        Ok(())
    }
}

impl<F: FileSystem, K: PathKeys, G: GuidSource> ConfigStore for JsonConfigStore<F, K, G> {
    fn load(
        &mut self,
        operation: OperationId,
        cancellation: &Cancellation,
    ) -> Result<ConfigLoad, ConfigStoreError> {
        self.gate.check_load()?;
        self.baseline = None;
        let result = self.run_load(cancellation);
        self.gate = self.gate.after_load(operation, &result);
        if result.is_ok() {
            self.quarantine_notice = false;
        } else {
            self.baseline = None;
        }
        result
    }

    fn save(
        &mut self,
        baseline: OperationId,
        config: &DomainConfig,
        cancellation: &Cancellation,
    ) -> ConfigSave {
        if let Err(error) = self.gate.check_save(baseline, config) {
            return ConfigSave::NotCommitted(error);
        }
        let outcome = self
            .run_save(config, cancellation)
            .unwrap_or_else(ConfigSave::NotCommitted);
        self.gate = self.gate.after_save(&outcome);
        outcome
    }
}
