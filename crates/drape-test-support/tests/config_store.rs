use drape_core::config_store::{ConfigLoad, ConfigSave, ConfigStore, ConfigStoreError};
use drape_core::domain::{DomainConfig, Guid, Page};
use drape_core::ports::{Cancellation, CommitDurability, FileSystemError, OperationId};
use drape_test_support::config_store::FakeConfigStore;

const FIRST: OperationId = OperationId(1);
const SECOND: OperationId = OperationId(2);

fn config() -> DomainConfig {
    let id = Guid::parse("6f1c2b8e-0000-4000-8000-000000000001").unwrap();
    let page = Page::new(id, "Main".into(), 0).unwrap();
    DomainConfig::new(1, vec![page], vec![]).unwrap()
}

fn empty() -> Result<ConfigLoad, ConfigStoreError> {
    Ok(ConfigLoad::Empty { quarantined: false })
}

fn committed() -> ConfigSave {
    ConfigSave::Committed {
        durability: CommitDurability::Confirmed,
        cleanup: None,
    }
}

#[test]
fn fake_refuses_save_before_load_without_consuming_the_script() {
    let mut store = FakeConfigStore::new(vec![empty()], vec![committed()]);
    let run = Cancellation::default();
    assert_eq!(
        store.save(FIRST, &config(), &run),
        ConfigSave::NotCommitted(ConfigStoreError::NotLoaded)
    );
    assert_eq!(store.load(FIRST, &run), empty());
    assert_eq!(store.save(FIRST, &config(), &run), committed());
}

#[test]
fn fake_records_only_committed_configs() {
    let failed =
        ConfigSave::NotCommitted(ConfigStoreError::FileSystem(FileSystemError::Unavailable));
    let mut store = FakeConfigStore::new(vec![empty()], vec![failed.clone(), committed()]);
    let run = Cancellation::default();
    store.load(FIRST, &run).unwrap();
    assert_eq!(store.save(FIRST, &config(), &run), failed);
    assert!(store.committed().is_empty());
    assert_eq!(store.save(FIRST, &config(), &run), committed());
    assert_eq!(store.committed(), [config()]);
}

#[test]
fn cancelled_calls_do_not_consume_scripts() {
    let mut store = FakeConfigStore::new(vec![empty()], vec![committed()]);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    let error = ConfigStoreError::FileSystem(FileSystemError::Cancelled);
    assert_eq!(store.load(FIRST, &cancelled), Err(error));
    let run = Cancellation::default();
    assert_eq!(store.load(FIRST, &run), empty());
    assert_eq!(
        store.save(FIRST, &config(), &cancelled),
        ConfigSave::NotCommitted(error)
    );
    assert_eq!(store.save(FIRST, &config(), &run), committed());
}

#[test]
fn exhausted_scripts_fail_closed() {
    let run = Cancellation::default();
    let unavailable = ConfigStoreError::FileSystem(FileSystemError::Unavailable);
    let mut store = FakeConfigStore::new(vec![], vec![]);
    assert_eq!(store.load(FIRST, &run), Err(unavailable));
    assert_eq!(
        store.save(FIRST, &config(), &run),
        ConfigSave::NotCommitted(ConfigStoreError::NotLoaded)
    );
    let mut store = FakeConfigStore::new(vec![empty()], vec![]);
    store.load(FIRST, &run).unwrap();
    assert_eq!(
        store.save(FIRST, &config(), &run),
        ConfigSave::NotCommitted(unavailable)
    );
    assert!(store.committed().is_empty());
}

#[test]
fn indeterminate_save_blocks_later_load_and_save() {
    let unknown = ConfigSave::Indeterminate(FileSystemError::Unavailable);
    let mut store =
        FakeConfigStore::new(vec![empty(), empty()], vec![unknown.clone(), committed()]);
    let run = Cancellation::default();
    store.load(FIRST, &run).unwrap();
    assert_eq!(store.save(FIRST, &config(), &run), unknown);
    assert_eq!(store.load(FIRST, &run), Err(ConfigStoreError::Stopped));
    assert_eq!(
        store.save(FIRST, &config(), &run),
        ConfigSave::NotCommitted(ConfigStoreError::Stopped)
    );
    assert!(store.committed().is_empty());
}

#[test]
fn fake_refuses_a_save_from_a_superseded_load_without_consuming_the_script() {
    let mut store = FakeConfigStore::new(vec![empty(), empty()], vec![committed()]);
    let run = Cancellation::default();
    store.load(FIRST, &run).unwrap();
    store.load(SECOND, &run).unwrap();
    assert_eq!(
        store.save(FIRST, &config(), &run),
        ConfigSave::NotCommitted(ConfigStoreError::StaleBaseline)
    );
    assert!(store.committed().is_empty());
    assert_eq!(store.save(SECOND, &config(), &run), committed());
}
