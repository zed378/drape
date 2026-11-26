use drape_core::config_store::{
    ConfigLoad, ConfigOrigin, ConfigSave, ConfigStoreError, ReadOnlyReason, SaveGate,
};
use drape_core::domain::{DomainConfig, Guid, Page};
use drape_core::ports::{CommitDurability, FileSystemError, OperationId};

fn config(schema: u32) -> DomainConfig {
    let id = Guid::parse("6f1c2b8e-0000-4000-8000-000000000001").unwrap();
    let page = Page::new(id, "Main".into(), 0).unwrap();
    DomainConfig::new(schema, vec![page], vec![]).unwrap()
}

fn loaded(schema: u32) -> Result<ConfigLoad, ConfigStoreError> {
    Ok(ConfigLoad::Loaded {
        config: config(schema),
        origin: ConfigOrigin::Current,
        quarantined: false,
        repairs: vec![],
    })
}

fn committed(durability: CommitDurability) -> ConfigSave {
    ConfigSave::Committed {
        durability,
        cleanup: None,
    }
}

const FIRST: OperationId = OperationId(1);
const SECOND: OperationId = OperationId(2);

fn writable() -> SaveGate {
    SaveGate::default().after_load(FIRST, &loaded(1))
}

#[test]
fn save_is_refused_until_a_load_establishes_the_baseline() {
    let gate = SaveGate::default();
    assert_eq!(gate.check_load(), Ok(()));
    assert_eq!(
        gate.check_save(FIRST, &config(1)),
        Err(ConfigStoreError::NotLoaded)
    );
}

#[test]
fn writable_baseline_comes_only_from_loaded_or_empty_results() {
    for result in [
        loaded(1),
        Ok(ConfigLoad::Empty { quarantined: false }),
        Ok(ConfigLoad::Empty { quarantined: true }),
    ] {
        let gate = SaveGate::default().after_load(FIRST, &result);
        assert_eq!(gate.check_save(FIRST, &config(1)), Ok(()));
    }
    for result in [
        loaded(2),
        Ok(ConfigLoad::ReadOnly {
            reason: ReadOnlyReason::NewerSchema(2),
            origin: ConfigOrigin::Current,
            quarantined: false,
            config: None,
        }),
        Ok(ConfigLoad::ReadOnly {
            reason: ReadOnlyReason::UnsupportedContent,
            origin: ConfigOrigin::Backup,
            quarantined: true,
            config: Some(config(1)),
        }),
    ] {
        let gate = SaveGate::default().after_load(FIRST, &result);
        assert_eq!(
            gate.check_save(FIRST, &config(1)),
            Err(ConfigStoreError::ReadOnly)
        );
    }
}

#[test]
fn newer_schema_config_is_refused_even_with_a_writable_baseline() {
    assert_eq!(
        writable().check_save(FIRST, &config(2)),
        Err(ConfigStoreError::ReadOnly)
    );
}

#[test]
fn failed_load_revokes_an_earlier_writable_baseline() {
    for error in [
        ConfigStoreError::FileSystem(FileSystemError::Cancelled),
        ConfigStoreError::FileSystem(FileSystemError::TooLarge),
        ConfigStoreError::FileSystem(FileSystemError::AccessDenied),
        ConfigStoreError::QuarantineFailed(FileSystemError::Unavailable),
    ] {
        let gate = writable().after_load(FIRST, &Err(error));
        assert_eq!(
            gate.check_save(FIRST, &config(1)),
            Err(ConfigStoreError::NotLoaded)
        );
        assert_eq!(gate.check_load(), Ok(()));
    }
}

#[test]
fn indeterminate_results_latch_the_gate_against_later_success() {
    let unknown = FileSystemError::Unavailable;
    for gate in [
        writable().after_load(FIRST, &Err(ConfigStoreError::Indeterminate(unknown))),
        writable().after_save(&ConfigSave::Indeterminate(unknown)),
        writable().after_save(&ConfigSave::NotCommitted(ConfigStoreError::Indeterminate(
            unknown,
        ))),
    ] {
        let later = gate
            .after_load(FIRST, &loaded(1))
            .after_save(&committed(CommitDurability::Confirmed));
        assert_eq!(later.check_load(), Err(ConfigStoreError::Stopped));
        assert_eq!(
            later.check_save(FIRST, &config(1)),
            Err(ConfigStoreError::Stopped)
        );
    }
}

#[test]
fn stale_baseline_save_failures_require_a_reload() {
    for error in [
        FileSystemError::Conflict,
        FileSystemError::NotFound,
        FileSystemError::AlreadyExists,
    ] {
        let failed = ConfigSave::NotCommitted(ConfigStoreError::FileSystem(error));
        let gate = writable().after_save(&failed);
        assert_eq!(
            gate.check_save(FIRST, &config(1)),
            Err(ConfigStoreError::NotLoaded)
        );
        assert_eq!(
            gate.after_load(FIRST, &loaded(1))
                .check_save(FIRST, &config(1)),
            Ok(())
        );
    }
}

#[test]
fn retryable_and_committed_saves_keep_the_gate_writable() {
    for result in [
        ConfigSave::NotCommitted(ConfigStoreError::FileSystem(FileSystemError::Cancelled)),
        ConfigSave::NotCommitted(ConfigStoreError::FileSystem(FileSystemError::Unavailable)),
        ConfigSave::NotCommitted(ConfigStoreError::ReadOnly),
        committed(CommitDurability::Confirmed),
        committed(CommitDurability::Unconfirmed(FileSystemError::Unavailable)),
    ] {
        assert_eq!(
            writable().after_save(&result).check_save(FIRST, &config(1)),
            Ok(())
        );
    }
}

#[test]
fn errors_describe_the_failure_without_payload_details() {
    let error = ConfigStoreError::QuarantineFailed(FileSystemError::AccessDenied);
    assert_eq!(
        error.to_string(),
        "corrupt configuration could not be preserved"
    );
}

#[test]
fn save_derived_from_a_superseded_load_is_refused() {
    let reloaded = writable().after_load(SECOND, &loaded(1));
    assert_eq!(
        reloaded.check_save(FIRST, &config(1)),
        Err(ConfigStoreError::StaleBaseline)
    );
    assert_eq!(reloaded.check_save(SECOND, &config(1)), Ok(()));
    let stale = ConfigSave::NotCommitted(ConfigStoreError::StaleBaseline);
    assert_eq!(reloaded.after_save(&stale), reloaded);
}

#[test]
fn document_found_unsavable_during_save_becomes_read_only() {
    let refused = ConfigSave::NotCommitted(ConfigStoreError::DocumentReadOnly(
        ReadOnlyReason::UnsupportedContent,
    ));
    let gate = writable().after_save(&refused);
    assert_eq!(
        gate.check_save(FIRST, &config(1)),
        Err(ConfigStoreError::ReadOnly)
    );
}
