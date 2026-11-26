//! Scripted configuration store behind the Core save gate. No parsing or I/O.

use drape_core::config_store::{ConfigLoad, ConfigSave, ConfigStore, ConfigStoreError, SaveGate};
use drape_core::domain::DomainConfig;
use drape_core::ports::{Cancellation, FileSystemError, OperationId};
use std::collections::VecDeque;

/// Refused and cancelled calls leave the scripts untouched; exhausted scripts fail.
#[derive(Debug, Default)]
pub struct FakeConfigStore {
    loads: VecDeque<Result<ConfigLoad, ConfigStoreError>>,
    saves: VecDeque<ConfigSave>,
    gate: SaveGate,
    committed: Vec<DomainConfig>,
}

impl FakeConfigStore {
    pub fn new(loads: Vec<Result<ConfigLoad, ConfigStoreError>>, saves: Vec<ConfigSave>) -> Self {
        Self {
            loads: loads.into(),
            saves: saves.into(),
            ..Self::default()
        }
    }

    /// Configs whose scripted save reported publication, oldest first.
    pub fn committed(&self) -> &[DomainConfig] {
        &self.committed
    }

    fn unavailable() -> ConfigStoreError {
        ConfigStoreError::FileSystem(FileSystemError::Unavailable)
    }

    fn cancelled(cancellation: &Cancellation) -> Result<(), ConfigStoreError> {
        if cancellation.is_cancelled() {
            Err(ConfigStoreError::FileSystem(FileSystemError::Cancelled))
        } else {
            Ok(())
        }
    }
}

impl ConfigStore for FakeConfigStore {
    fn load(
        &mut self,
        operation: OperationId,
        cancellation: &Cancellation,
    ) -> Result<ConfigLoad, ConfigStoreError> {
        self.gate.check_load()?;
        let result = Self::cancelled(cancellation)
            .and_then(|()| self.loads.pop_front().unwrap_or(Err(Self::unavailable())));
        self.gate = self.gate.after_load(operation, &result);
        result
    }

    fn save(
        &mut self,
        baseline: OperationId,
        config: &DomainConfig,
        cancellation: &Cancellation,
    ) -> ConfigSave {
        if let Err(error) = self
            .gate
            .check_save(baseline, config)
            .and_then(|()| Self::cancelled(cancellation))
        {
            return ConfigSave::NotCommitted(error);
        }
        let result = self
            .saves
            .pop_front()
            .unwrap_or(ConfigSave::NotCommitted(Self::unavailable()));
        if matches!(result, ConfigSave::Committed { .. }) {
            self.committed.push(config.clone());
        }
        self.gate = self.gate.after_save(&result);
        result
    }
}
