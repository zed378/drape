use drape_core::TimestampMillis;
use drape_core::config_store::{
    ConfigLoad, ConfigOrigin, ConfigSave, ConfigStore, ConfigStoreError, ReadOnlyReason,
};
use drape_core::domain::{
    DomainConfig, DrapeItem, Guid, ItemKind, LabelMode, Page, PathKey, SortMode,
};
use drape_core::ports::{
    Cancellation, CommitDurability, FileSystem, FileSystemError, OperationId, PortError,
};
use drape_infrastructure::config::{GuidSource, JsonConfigStore, MAX_DOCUMENT_BYTES};
use drape_test_support::FakePathKeys;
use drape_test_support::config_fs::{FakeFailure, FakeFileSystem, FileSystemStage};
use serde_json::{Value, json};
use std::num::NonZeroUsize;
use std::rc::Rc;

const LEGACY: &str = include_str!("fixtures/legacy-v1.json");
const EXTENDED: &str = include_str!("fixtures/extended-v1.json");
const LOAD: OperationId = OperationId(1);

struct Counter(u32);
impl GuidSource for Counter {
    fn next_guid(&mut self) -> Result<Guid, PortError> {
        self.0 += 1;
        Ok(id(self.0))
    }
}

fn id(n: u32) -> Guid {
    let mut bytes = [0xAB; 16];
    bytes[12..].copy_from_slice(&n.to_be_bytes());
    Guid::from_bytes(bytes).unwrap()
}

fn fallback() -> Page {
    Page::new(id(900_000), "Main".into(), 0).unwrap()
}

type Store = JsonConfigStore<FakeFileSystem, FakePathKeys, Counter>;

fn store(files: &FakeFileSystem) -> Store {
    JsonConfigStore::new(
        files.clone(),
        FakePathKeys::default(),
        Counter(0),
        fallback(),
    )
}

fn seeded(text: &str) -> (FakeFileSystem, Store) {
    let mut files = FakeFileSystem::new();
    files.seed_current(text.as_bytes().to_vec()).unwrap();
    let store = store(&files);
    (files, store)
}

fn load(store: &mut Store) -> Result<ConfigLoad, ConfigStoreError> {
    store.load(LOAD, &Cancellation::default())
}

fn loaded(store: &mut Store) -> DomainConfig {
    match load(store).unwrap() {
        ConfigLoad::Loaded { config, .. } => config,
        other => panic!("expected a writable load, got {other:?}"),
    }
}

fn reloaded(files: &FakeFileSystem) -> DomainConfig {
    loaded(&mut store(files))
}

fn save(store: &mut Store, config: &DomainConfig) -> ConfigSave {
    store.save(LOAD, config, &Cancellation::default())
}

fn assert_committed(result: ConfigSave) {
    assert_eq!(
        result,
        ConfigSave::Committed {
            durability: CommitDurability::Confirmed,
            cleanup: None
        }
    );
}

fn current(files: &FakeFileSystem) -> Value {
    serde_json::from_slice(&files.peek_current().unwrap()).unwrap()
}

fn backup_count(files: &FakeFileSystem) -> usize {
    files
        .list_backups(NonZeroUsize::new(4096).unwrap(), &Cancellation::default())
        .unwrap()
        .len()
}

fn renamed(config: &DomainConfig, title: &str) -> DomainConfig {
    let drape = config.drapes()[0].rename(title.into()).unwrap();
    config.replace_drape(drape).unwrap()
}

fn titled(title: &str) -> String {
    LEGACY.replace("Laporan \\u672a", title)
}

fn patched(edit: impl FnOnce(&mut Value)) -> String {
    let mut value: Value = serde_json::from_str(LEGACY).unwrap();
    edit(&mut value);
    serde_json::to_string(&value).unwrap()
}

#[test]
fn legacy_writer_shape_loads_without_loss_or_repair() {
    let (_, mut store) = seeded(LEGACY);
    let ConfigLoad::Loaded {
        config,
        origin,
        quarantined,
        repairs,
    } = load(&mut store).unwrap()
    else {
        panic!("legacy file must be writable")
    };
    assert_eq!(origin, ConfigOrigin::Current);
    assert!(!quarantined && repairs.is_empty());
    assert!(!config.settings().quick_hide_enabled);
    assert!(config.settings().autostart);
    let drape = &config.drapes()[0];
    assert_eq!(drape.title(), "Laporan \u{672a}");
    assert_eq!(drape.style.label_mode, LabelMode::OneLine);
    assert_eq!(drape.style.tint, None);
    assert_eq!(drape.state.sort_mode, SortMode::DateAdded);
    assert!(drape.quick_hide_excluded);
    assert_eq!(
        drape.items()[0].added_at(),
        TimestampMillis(1_791_531_780_123)
    );
    assert_eq!(drape.items()[1].kind(), ItemKind::Shortcut);
    assert!(drape.items()[1].missing_since().is_some());
}

#[test]
fn unchanged_save_keeps_every_stored_value_and_the_previous_bytes_as_backup() {
    for fixture in [LEGACY, EXTENDED] {
        let (files, mut store) = seeded(fixture);
        let config = loaded(&mut store);
        assert_committed(save(&mut store, &config));
        let before: Value = serde_json::from_str(fixture).unwrap();
        assert_eq!(current(&files), before);
        assert_eq!(files.peek_backup(id(2)).unwrap(), fixture.as_bytes());
    }
}

#[test]
fn schema_v1_round_trip_preserves_nested_unknown_values_and_numeric_precision() {
    let (files, mut store) = seeded(EXTENDED);
    let config = loaded(&mut store);
    assert_committed(save(&mut store, &renamed(&config, "Renamed")));
    let text = String::from_utf8(files.peek_current().unwrap()).unwrap();
    for literal in [
        "9007199254740993",
        "1.00000000000000000000001",
        "18446744073709551617",
        "40.50",
        "2026-10-09T07:43:00.000000001Z",
        "two-lines",
        "\"Normal\"",
        "futureDelay",
        "futureBounds",
        "futureStyle",
        "futureState",
        "futureItem",
        "futureRule",
    ] {
        assert!(text.contains(literal), "lost {literal}");
    }
    let mut expected: Value = serde_json::from_str(EXTENDED).unwrap();
    expected["drapes"][0]["title"] = json!("Renamed");
    assert_eq!(current(&files), expected);
    let keys: Vec<_> = current(&files)
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(keys[0], "futureRoot");
}

#[test]
fn changed_values_use_written_forms_and_update_each_present_alias() {
    let (files, mut store) = seeded(EXTENDED);
    let config = loaded(&mut store);
    let mut settings = config.settings().clone();
    settings.quick_hide_enabled = false;
    let mut drape = config.drapes()[0].clone();
    drape.style.label_mode = LabelMode::None;
    let changed = config
        .with_settings(settings)
        .unwrap()
        .replace_drape(drape)
        .unwrap();
    assert_committed(save(&mut store, &changed));
    let value = current(&files);
    assert_eq!(value["settings"]["quickHide"]["enabled"], json!(false));
    assert!(value["settings"].get("quickHideEnabled").is_none());
    assert_eq!(value["drapes"][0]["style"]["labelMode"], json!("none"));
    assert_eq!(reloaded(&files), changed);
}

#[test]
fn added_and_moved_items_keep_identity_and_unknown_properties() {
    let (files, mut store) = seeded(EXTENDED);
    let config = loaded(&mut store);
    let moved = config.drapes()[0].items()[0].clone();
    let added = DrapeItem::new(
        id(700),
        ItemKind::Folder,
        "D:\\new".into(),
        PathKey::new("D:\\new".into()).unwrap(),
        TimestampMillis(-1),
    )
    .unwrap();
    let source = config.drapes()[0].remove_item(moved.id());
    let target = config.drapes()[1]
        .add_item(moved)
        .unwrap()
        .add_item(added)
        .unwrap();
    let changed = config
        .replace_drape(source)
        .unwrap()
        .replace_drape(target)
        .unwrap();
    assert_committed(save(&mut store, &changed));
    let value = current(&files);
    assert_eq!(value["drapes"][1]["items"][0]["futureItem"], json!([1, 2]));
    assert_eq!(
        value["drapes"][1]["items"][1]["addedAt"],
        json!("1969-12-31T23:59:59.999Z")
    );
    assert_eq!(reloaded(&files), changed);
}

#[test]
fn missing_valid_config_returns_empty_and_first_save_creates_a_loadable_document() {
    let files = FakeFileSystem::new();
    let mut first = store(&files);
    assert_eq!(
        load(&mut first),
        Ok(ConfigLoad::Empty { quarantined: false })
    );
    let legacy = loaded(&mut seeded(LEGACY).1);
    assert_committed(save(&mut first, &legacy));
    assert_eq!(reloaded(&files), legacy);
    assert_eq!(backup_count(&files), 0);
}

#[test]
fn unknown_page_and_duplicate_membership_are_repaired_reported_and_persisted() {
    let text = patched(|v| {
        let mut second = v["drapes"][0].clone();
        second["id"] = json!("6f1c2b8e-0000-4000-8000-000000000030");
        second["pageId"] = json!("6f1c2b8e-0000-4000-8000-0000000000ff");
        v["drapes"].as_array_mut().unwrap().push(second);
    });
    let (files, mut first) = seeded(&text);
    let ConfigLoad::Loaded {
        config, repairs, ..
    } = load(&mut first).unwrap()
    else {
        panic!("repairable file must load")
    };
    assert_eq!(repairs.len(), 3);
    assert_committed(save(&mut first, &config));
    let value = current(&files);
    assert_eq!(value["drapes"][1]["pageId"], value["pages"][0]["id"]);
    assert_eq!(value["drapes"][1]["items"], json!([]));
    let ConfigLoad::Loaded { repairs, .. } = load(&mut store(&files)).unwrap() else {
        panic!("saved file must load")
    };
    assert!(repairs.is_empty());
}

fn assert_read_only(text: &str, reason: ReadOnlyReason, projected: bool) {
    let (files, mut store) = seeded(text);
    let ConfigLoad::ReadOnly {
        reason: actual,
        config,
        origin,
        quarantined,
    } = load(&mut store).unwrap()
    else {
        panic!("expected read-only for {text}")
    };
    assert_eq!(actual, reason, "{text}");
    assert_eq!(config.is_some(), projected, "{text}");
    assert_eq!((origin, quarantined), (ConfigOrigin::Current, false));
    let other = loaded(&mut seeded(LEGACY).1);
    assert_eq!(
        save(&mut store, &other),
        ConfigSave::NotCommitted(ConfigStoreError::ReadOnly)
    );
    assert_eq!(files.peek_current().unwrap(), text.as_bytes());
    assert!(!files.operations().contains(&FileSystemStage::Quarantine));
}

#[test]
fn future_schema_is_read_only_and_bytes_remain_unchanged() {
    for (text, version) in [
        (r#"{"schemaVersion": 2, "pages": "reshaped"}"#, 2),
        (r#"{"SchemaVersion": 7}"#, 7),
        (r#"{"schemaVersion": 1, "SchemaVersion": 3}"#, 3),
        (r#"{"schemaVersion": 99999999999999999999999}"#, u32::MAX),
        (r#"{"schemaVersion": 2, "x": 1, "x": 2}"#, 2),
    ] {
        assert_read_only(text, ReadOnlyReason::NewerSchema(version), false);
    }
}

#[test]
fn ambiguous_documents_are_read_only_without_projection() {
    for text in [
        LEGACY.replacen(
            "\"theme\": \"dark\",",
            "\"theme\": \"dark\", \"theme\": \"light\",",
            1,
        ),
        titled("lone \\ud800 surrogate"),
        LEGACY.replacen(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1, \"ext\": {\"$serde_json::private::Number\": \"123\"},",
            1,
        ),
        LEGACY.replacen(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1, \"ext\": {\"$serde_json::private::RawValue\": \"x\"},",
            1,
        ),
    ] {
        assert_read_only(&text, ReadOnlyReason::UnsupportedContent, false);
    }
}

type Edit = fn(&mut Value);

#[test]
fn unsupported_values_are_read_only_with_a_projection_and_are_never_quarantined() {
    let edits: [Edit; 19] = [
        |v| v["settings"]["Language"] = json!("x"),
        |v| v["settings"]["quickHide"] = json!({ "enabled": true }),
        |v| v["settings"]["updateCheck"] = json!("hourly"),
        |v| v["settings"]["dropMode"] = json!("move"),
        |v| v["drapes"][0]["kind"] = json!("stack"),
        |v| v["drapes"][0]["items"][0]["kind"] = json!("virtual"),
        |v| v["drapes"][0]["title"] = json!("  "),
        |v| v["pages"][0]["name"] = json!(""),
        |v| v["drapes"][0]["items"][0]["path"] = json!(""),
        |v| v["drapes"][0]["bounds"]["width"] = json!(119),
        |v| v["drapes"][0]["bounds"]["fx"] = json!(1.5),
        |v| v["drapes"][0]["bounds"]["dpi"] = json!(0),
        |v| v["drapes"][0]["style"]["opacity"] = json!(2),
        |v| v["drapes"][0]["items"][0]["order"] = json!(-1),
        |v| {
            v["drapes"][0]["items"][0]
                .as_object_mut()
                .unwrap()
                .remove("addedAt");
        },
        |v| v["drapes"][0]["items"][0]["addedAt"] = json!("yesterday"),
        |v| v["drapes"][0]["items"][0]["addedAt"] = json!("2026-02-30T00:00:00Z"),
        |v| {
            let copy = v["drapes"][0]["items"][0].clone();
            v["drapes"][0]["items"].as_array_mut().unwrap().push(copy);
        },
        |v| {
            let mut copy = v["drapes"][0].clone();
            copy["id"] = json!("6f1c2b8e-0000-4000-8000-000000000030");
            copy["items"][0]["addedAt"] = json!("2026-10-09T14:43:00.1239+07:00");
            v["drapes"].as_array_mut().unwrap().push(copy);
        },
    ];
    for edit in edits {
        assert_read_only(&patched(edit), ReadOnlyReason::UnsupportedContent, true);
    }
    let huge = LEGACY.replacen("\"x\": 40,", "\"x\": 1e999,", 1);
    assert_read_only(&huge, ReadOnlyReason::UnsupportedContent, true);
}

#[test]
fn structural_damage_is_quarantined_byte_for_byte() {
    let broken: Vec<String> = vec![
        "{".into(),
        "[]".into(),
        "{\"pages\": []}".into(),
        "{\"schemaVersion\": 0}".into(),
        "{\"schemaVersion\": \"1\"}".into(),
        patched(|v| v["pages"] = json!(5)),
        patched(|v| v["drapes"][0]["title"] = json!(5)),
        patched(|v| v["drapes"][0]["id"] = json!("nope")),
        patched(|v| {
            v["drapes"][0].as_object_mut().unwrap().remove("bounds");
        }),
        patched(|v| {
            let copy = v["pages"][0].clone();
            v["pages"].as_array_mut().unwrap().push(copy);
        }),
        patched(|v| {
            let mut copy = v["drapes"][0]["items"][0].clone();
            copy["path"] = json!("D:\\other");
            v["drapes"][0]["items"].as_array_mut().unwrap().push(copy);
        }),
    ];
    for text in broken {
        let (files, mut store) = seeded(&text);
        assert_eq!(
            load(&mut store),
            Ok(ConfigLoad::Empty { quarantined: true }),
            "{text}"
        );
        assert_eq!(files.peek_quarantine(id(1)).unwrap(), text.as_bytes());
        assert!(files.peek_current().is_none());
    }
}

#[test]
fn backup_recovery_uses_newest_valid_document() {
    let (mut files, mut store) = seeded("{");
    for (n, bytes) in [
        (501, titled("older").into_bytes()),
        (502, titled("newest valid").into_bytes()),
        (503, b"{".to_vec()),
    ] {
        files.seed_backup(id(n), bytes).unwrap();
    }
    let ConfigLoad::Loaded {
        config,
        origin,
        quarantined,
        ..
    } = load(&mut store).unwrap()
    else {
        panic!("a valid backup exists")
    };
    assert_eq!((origin, quarantined), (ConfigOrigin::Backup, true));
    assert_eq!(config.drapes()[0].title(), "newest valid");
    assert_committed(save(&mut store, &config));
    assert_eq!(current(&files)["drapes"][0]["title"], json!("newest valid"));
    assert_eq!(backup_count(&files), 3);
}

#[test]
fn read_only_backup_is_returned_and_never_skipped_for_an_older_one() {
    let mut files = FakeFileSystem::new();
    files
        .seed_backup(id(501), titled("older").into_bytes())
        .unwrap();
    files
        .seed_backup(id(502), br#"{"schemaVersion": 2}"#.to_vec())
        .unwrap();
    assert_eq!(
        load(&mut store(&files)),
        Ok(ConfigLoad::ReadOnly {
            reason: ReadOnlyReason::NewerSchema(2),
            config: None,
            origin: ConfigOrigin::Backup,
            quarantined: false,
        })
    );
}

#[test]
fn save_after_recovery_never_overwrites_a_document_that_appeared() {
    let mut files = FakeFileSystem::new();
    files
        .seed_backup(id(501), titled("backup").into_bytes())
        .unwrap();
    let mut store = store(&files);
    let config = loaded(&mut store);
    files.seed_current(b"someone else".to_vec()).unwrap();
    assert_eq!(
        save(&mut store, &config),
        ConfigSave::NotCommitted(ConfigStoreError::FileSystem(FileSystemError::AlreadyExists))
    );
    assert_eq!(files.peek_current().unwrap(), b"someone else");
}

#[test]
fn external_change_conflicts_and_requires_a_reload() {
    let (mut files, mut store) = seeded(LEGACY);
    let config = loaded(&mut store);
    files.seed_current(titled("edited").into_bytes()).unwrap();
    assert_eq!(
        save(&mut store, &config),
        ConfigSave::NotCommitted(ConfigStoreError::FileSystem(FileSystemError::Conflict))
    );
    assert_eq!(
        save(&mut store, &config),
        ConfigSave::NotCommitted(ConfigStoreError::NotLoaded)
    );
    assert_eq!(files.peek_current().unwrap(), titled("edited").as_bytes());
    assert!(files.peek_temporary(id(1)).is_none());
}

#[test]
fn consecutive_saves_replace_their_own_result_and_stale_loads_are_refused() {
    let (files, mut store) = seeded(LEGACY);
    let config = loaded(&mut store);
    assert_committed(save(&mut store, &renamed(&config, "one")));
    assert_committed(save(&mut store, &renamed(&config, "two")));
    assert_eq!(current(&files)["drapes"][0]["title"], json!("two"));
    store
        .load(OperationId(2), &Cancellation::default())
        .unwrap();
    assert_eq!(
        save(&mut store, &config),
        ConfigSave::NotCommitted(ConfigStoreError::StaleBaseline)
    );
}

#[test]
fn save_failure_at_each_stage_keeps_previous_config_and_backup() {
    use FileSystemStage::*;
    for stage in [
        AcquireWriter,
        CreateTemporary,
        WriteTemporary,
        SyncTemporary,
        CommitTemporary,
    ] {
        let (mut files, mut store) = seeded(LEGACY);
        files
            .seed_backup(id(501), titled("older").into_bytes())
            .unwrap();
        let config = loaded(&mut store);
        files.inject(FakeFailure::Error(stage, FileSystemError::Unavailable));
        assert_eq!(
            save(&mut store, &renamed(&config, "new")),
            ConfigSave::NotCommitted(ConfigStoreError::FileSystem(FileSystemError::Unavailable)),
            "{stage:?}"
        );
        assert_eq!(files.pending_failures(), 0);
        assert_eq!(files.peek_current().unwrap(), LEGACY.as_bytes());
        assert_eq!(backup_count(&files), 1);
        assert!(files.peek_temporary(id(1)).is_none(), "{stage:?}");
        assert_committed(save(&mut store, &renamed(&config, "retry")));
    }
}

#[test]
fn partial_write_never_reaches_the_destination() {
    let (files, mut store) = seeded(LEGACY);
    let config = loaded(&mut store);
    files.inject(FakeFailure::PartialWrite {
        bytes: 7,
        error: FileSystemError::Unavailable,
    });
    assert!(matches!(
        save(&mut store, &config),
        ConfigSave::NotCommitted(_)
    ));
    assert_eq!(files.peek_current().unwrap(), LEGACY.as_bytes());
    assert!(files.peek_temporary(id(1)).is_none());
}

#[test]
fn indeterminate_commit_stops_the_store_and_keeps_evidence() {
    for publish in [false, true] {
        let (files, mut store) = seeded(LEGACY);
        let config = loaded(&mut store);
        files.inject(FakeFailure::CommitIndeterminate {
            publish,
            error: FileSystemError::Unavailable,
        });
        assert_eq!(
            save(&mut store, &renamed(&config, "new")),
            ConfigSave::Indeterminate(FileSystemError::Unavailable)
        );
        assert!(files.peek_temporary(id(1)).is_some());
        let operations = files.operations();
        assert!(!operations.contains(&FileSystemStage::CleanupTemporary));
        assert!(!operations.contains(&FileSystemStage::RemoveBackup));
        assert_eq!(load(&mut store), Err(ConfigStoreError::Stopped));
    }
}

#[test]
fn cancelled_calls_change_nothing() {
    let (files, mut store) = seeded(LEGACY);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        store.load(LOAD, &cancelled),
        Err(ConfigStoreError::FileSystem(FileSystemError::Cancelled))
    );
    let config = loaded(&mut store);
    assert_eq!(
        store.save(LOAD, &config, &cancelled),
        ConfigSave::NotCommitted(ConfigStoreError::FileSystem(FileSystemError::Cancelled))
    );
    assert_eq!(files.peek_current().unwrap(), LEGACY.as_bytes());
    assert_committed(save(&mut store, &config));
}

#[test]
fn failed_quarantine_never_falls_through_to_overwrite() {
    let (mut files, mut store) = seeded("{");
    files
        .seed_backup(id(501), titled("older").into_bytes())
        .unwrap();
    files.inject(FakeFailure::Error(
        FileSystemStage::Quarantine,
        FileSystemError::AccessDenied,
    ));
    assert_eq!(
        load(&mut store),
        Err(ConfigStoreError::QuarantineFailed(
            FileSystemError::AccessDenied
        ))
    );
    let other = loaded(&mut seeded(LEGACY).1);
    assert_eq!(
        save(&mut store, &other),
        ConfigSave::NotCommitted(ConfigStoreError::NotLoaded)
    );
    assert_eq!(files.peek_current().unwrap(), b"{");
}

#[test]
fn corrupt_config_is_quarantined_without_overwriting_a_prior_entry() {
    let (mut files, mut first) = seeded("{");
    load(&mut first).unwrap();
    files.seed_current(b"[".to_vec()).unwrap();
    assert_eq!(
        load(&mut store(&files)),
        Err(ConfigStoreError::QuarantineFailed(
            FileSystemError::AlreadyExists
        ))
    );
    assert_eq!(files.peek_quarantine(id(1)).unwrap(), b"{");
    assert_eq!(files.peek_current().unwrap(), b"[");
}

#[test]
fn indeterminate_quarantine_stops_recovery() {
    let (files, mut store) = seeded("{");
    files.inject(FakeFailure::QuarantineIndeterminate {
        move_file: true,
        error: FileSystemError::Unavailable,
    });
    assert_eq!(
        load(&mut store),
        Err(ConfigStoreError::Indeterminate(
            FileSystemError::Unavailable
        ))
    );
    assert!(!files.operations().contains(&FileSystemStage::ListBackups));
    assert_eq!(load(&mut store), Err(ConfigStoreError::Stopped));
}

#[test]
fn quarantine_notice_survives_a_failed_recovery_until_a_load_succeeds() {
    let (files, mut store) = seeded("{");
    files.inject(FakeFailure::Error(
        FileSystemStage::ListBackups,
        FileSystemError::Unavailable,
    ));
    assert!(load(&mut store).is_err());
    assert_eq!(
        load(&mut store),
        Ok(ConfigLoad::Empty { quarantined: true })
    );
    assert_eq!(
        load(&mut store),
        Ok(ConfigLoad::Empty { quarantined: false })
    );
}

#[test]
fn read_failures_are_reported_and_move_nothing() {
    for error in [FileSystemError::AccessDenied, FileSystemError::Unavailable] {
        let (files, mut store) = seeded(LEGACY);
        files.inject(FakeFailure::Error(FileSystemStage::Read, error));
        assert_eq!(load(&mut store), Err(ConfigStoreError::FileSystem(error)));
        assert!(!files.operations().contains(&FileSystemStage::Quarantine));
    }
}

#[test]
fn path_key_failure_fails_the_load_without_moving_anything() {
    let mut files = FakeFileSystem::new();
    files.seed_current(LEGACY.as_bytes().to_vec()).unwrap();
    let mut store = JsonConfigStore::new(
        files.clone(),
        FakePathKeys::failing(PortError::Unavailable),
        Counter(0),
        fallback(),
    );
    assert_eq!(
        store.load(LOAD, &Cancellation::default()),
        Err(ConfigStoreError::FileSystem(FileSystemError::Unavailable))
    );
    assert!(!files.operations().contains(&FileSystemStage::Quarantine));
}

#[test]
fn oversized_document_is_preserved_and_never_replaced() {
    let deep = format!(
        "{{\"schemaVersion\":1,\"x\":{}{}}}",
        "[".repeat(65),
        "]".repeat(65)
    );
    let long = format!(
        "{{\"schemaVersion\":1,\"x\":\"{}\"}}",
        "a".repeat(256 * 1024 + 1)
    );
    let many = format!("{{\"schemaVersion\":1,\"x\":[{}0]}}", "0,".repeat(200_000));
    let big = format!("{}{}", LEGACY, " ".repeat(MAX_DOCUMENT_BYTES));
    for text in [deep, long, many, big] {
        let (files, mut store) = seeded(&text);
        assert_eq!(
            load(&mut store),
            Err(ConfigStoreError::FileSystem(FileSystemError::TooLarge))
        );
        assert_eq!(files.peek_current().unwrap(), text.as_bytes());
        assert!(!files.operations().contains(&FileSystemStage::Quarantine));
    }
}

fn count(value: &Value) -> usize {
    1 + match value {
        Value::Array(items) => items.iter().map(count).sum(),
        Value::Object(map) => map.values().map(count).sum(),
        _ => 0,
    }
}

#[test]
fn legacy_scale_profile_stays_under_a_quarter_of_the_limits() {
    let text = patched(|v| {
        let drape = v["drapes"][0].clone();
        let item = drape["items"][0].clone();
        let drapes: Vec<Value> = (0..50)
            .map(|d| {
                let mut drape = drape.clone();
                drape["id"] = json!(id(1000 + d).to_string());
                drape["items"] = (0..50)
                    .map(|i| {
                        let mut item = item.clone();
                        item["id"] = json!(id(10_000 + d * 50 + i).to_string());
                        item["order"] = json!(i);
                        item["path"] = json!(format!("C:\\Users\\example\\Desktop\\{d}-{i}.txt"));
                        item
                    })
                    .collect();
                drape
            })
            .collect();
        v["drapes"] = Value::Array(drapes);
    });
    let (files, mut store) = seeded(&text);
    let config = loaded(&mut store);
    assert_committed(save(&mut store, &renamed(&config, "profile")));
    let bytes = files.peek_current().unwrap();
    assert!(bytes.len() < MAX_DOCUMENT_BYTES / 4, "{}", bytes.len());
    assert!(count(&current(&files)) < 200_000 / 4);
}

fn with_backups(valid: u32) -> (FakeFileSystem, Store, DomainConfig) {
    let (mut files, mut store) = seeded(LEGACY);
    for n in 0..valid {
        files
            .seed_backup(id(600 + n), titled(&format!("backup {n}")).into_bytes())
            .unwrap();
    }
    let config = loaded(&mut store);
    (files, store, config)
}

#[test]
fn successful_replace_prunes_only_after_commit() {
    let (files, mut store, config) = with_backups(12);
    files.inject(FakeFailure::Error(
        FileSystemStage::CommitTemporary,
        FileSystemError::Unavailable,
    ));
    save(&mut store, &config);
    assert_eq!(backup_count(&files), 12);
    assert_committed(save(&mut store, &config));
    assert_eq!(backup_count(&files), 10);
    assert!(files.peek_backup(id(600)).is_none());
    assert!(files.peek_backup(id(611)).is_some());
}

#[test]
fn unconfirmed_durability_prunes_nothing() {
    let (files, mut store, config) = with_backups(12);
    files.inject(FakeFailure::CommitDurability(FileSystemError::Unavailable));
    assert_eq!(
        save(&mut store, &config),
        ConfigSave::Committed {
            durability: CommitDurability::Unconfirmed(FileSystemError::Unavailable),
            cleanup: None
        }
    );
    assert_eq!(backup_count(&files), 13);
}

#[test]
fn corrupt_backups_never_displace_valid_ones() {
    let (mut files, mut store, config) = with_backups(10);
    for n in 0..10 {
        files.seed_backup(id(700 + n), b"{".to_vec()).unwrap();
    }
    assert_committed(save(&mut store, &config));
    // The replaced current document is the newest valid backup, so one old one leaves.
    assert!(files.peek_backup(id(600)).is_none());
    assert!(files.peek_backup(id(601)).is_some());
    assert!(files.peek_backup(id(700)).is_some());
    assert_eq!(backup_count(&files), 20);
}

#[test]
fn pruning_keeps_read_only_backups_and_removes_corrupt_ones_past_the_tenth_valid() {
    let (mut files, mut store) = seeded(LEGACY);
    files.seed_backup(id(800), b"{".to_vec()).unwrap();
    files
        .seed_backup(id(801), br#"{"schemaVersion": 2}"#.to_vec())
        .unwrap();
    for n in 0..10 {
        files
            .seed_backup(id(600 + n), titled(&format!("b{n}")).into_bytes())
            .unwrap();
    }
    let config = loaded(&mut store);
    assert_committed(save(&mut store, &config));
    assert!(files.peek_backup(id(800)).is_none());
    assert!(files.peek_backup(id(801)).is_some());
}

#[test]
fn fewer_than_ten_valid_backups_removes_nothing() {
    let (mut files, mut store, config) = with_backups(3);
    for n in 0..12 {
        files.seed_backup(id(700 + n), b"{".to_vec()).unwrap();
    }
    assert_committed(save(&mut store, &config));
    assert_eq!(backup_count(&files), 16);
}

#[test]
fn prune_failure_is_reported_as_cleanup_of_a_committed_save() {
    let (files, mut store, config) = with_backups(12);
    files.inject(FakeFailure::Error(
        FileSystemStage::RemoveBackup,
        FileSystemError::AccessDenied,
    ));
    assert_eq!(
        save(&mut store, &config),
        ConfigSave::Committed {
            durability: CommitDurability::Confirmed,
            cleanup: Some(FileSystemError::AccessDenied)
        }
    );
}

#[cfg(windows)]
#[test]
fn unrepresentable_path_is_refused_before_any_mutation() {
    use std::os::windows::ffi::OsStringExt;
    let (files, mut store) = seeded(LEGACY);
    let config = loaded(&mut store);
    let item = DrapeItem::new(
        id(700),
        ItemKind::File,
        std::ffi::OsString::from_wide(&[0x43, 0x3a, 0x5c, 0xd800]),
        PathKey::new("opaque".into()).unwrap(),
        TimestampMillis(0),
    )
    .unwrap();
    let drape = config.drapes()[0].add_item(item).unwrap();
    assert_eq!(
        save(&mut store, &config.replace_drape(drape).unwrap()),
        ConfigSave::NotCommitted(ConfigStoreError::ReadOnly)
    );
    assert_eq!(files.peek_current().unwrap(), LEGACY.as_bytes());
    assert!(!files.operations().contains(&FileSystemStage::AcquireWriter));
}

#[test]
fn unchanged_null_optional_values_stay_stored() {
    let text = patched(|v| {
        let item = &mut v["drapes"][0]["items"][0];
        item["missingSince"] = Value::Null;
        item["fileId"] = Value::Null;
    });
    let (files, mut store) = seeded(&text);
    let config = loaded(&mut store);
    assert_committed(save(&mut store, &config));
    assert_eq!(
        current(&files),
        serde_json::from_str::<Value>(&text).unwrap()
    );
}

#[test]
fn truncated_json_with_a_duplicate_property_is_corrupt() {
    let (files, mut store) = seeded(r#"{"schemaVersion":1,"pages":[],"x":0,"x":1"#);
    assert_eq!(
        load(&mut store),
        Ok(ConfigLoad::Empty { quarantined: true })
    );
    assert!(files.peek_current().is_none());
}

fn projection(text: &str) -> DomainConfig {
    let (files, mut store) = seeded(text);
    let ConfigLoad::ReadOnly {
        config: Some(config),
        ..
    } = load(&mut store).unwrap()
    else {
        panic!("expected a read-only projection for {text}")
    };
    assert_eq!(files.peek_current().unwrap(), text.as_bytes());
    config
}

#[test]
fn ambiguous_quick_hide_forms_project_the_default() {
    let conflicting = patched(|v| v["settings"]["quickHide"] = json!({ "enabled": true }));
    assert!(projection(&conflicting).settings().quick_hide_enabled);
    let respelled = LEGACY.replacen("\"quickHideEnabled\"", "\"QuickHideEnabled\"", 1);
    assert!(projection(&respelled).settings().quick_hide_enabled);
}

#[test]
fn differently_cased_properties_are_read_only_and_never_quarantined() {
    let required = LEGACY.replacen("\"pages\"", "\"Pages\"", 1);
    assert_eq!(projection(&required).pages().len(), 1);
    let bounds = LEGACY.replacen("\"monitorId\"", "\"MonitorID\"", 1);
    assert_eq!(projection(&bounds).drapes().len(), 1);
    let optional = patched(|v| v["drapes"][0]["Title"] = json!(5));
    assert_eq!(
        projection(&optional).drapes()[0].title(),
        "Laporan \u{672a}"
    );
}

#[test]
fn a_listed_backup_that_cannot_be_read_stops_recovery() {
    let mut files = FakeFileSystem::new();
    for n in [501, 502] {
        files
            .seed_backup(id(n), titled("backup").into_bytes())
            .unwrap();
    }
    for _ in 0..2 {
        files.inject(FakeFailure::Error(
            FileSystemStage::Read,
            FileSystemError::NotFound,
        ));
    }
    assert_eq!(
        load(&mut store(&files)),
        Err(ConfigStoreError::FileSystem(FileSystemError::NotFound))
    );
}

#[test]
fn cancellation_during_writer_acquisition_prevents_quarantine() {
    let (files, mut store) = seeded("{");
    let cancellation = Rc::new(Cancellation::default());
    files.cancel_on_acquire(Rc::clone(&cancellation));
    assert_eq!(
        store.load(LOAD, &cancellation),
        Err(ConfigStoreError::FileSystem(FileSystemError::Cancelled))
    );
    assert_eq!(files.peek_current().unwrap(), b"{");
    assert!(!files.operations().contains(&FileSystemStage::Quarantine));
    assert_eq!(
        load(&mut store),
        Ok(ConfigLoad::Empty { quarantined: true })
    );
}

#[test]
fn output_that_would_not_read_back_as_saved_is_refused() {
    let (files, mut store) = seeded(LEGACY);
    let config = loaded(&mut store);
    let item = DrapeItem::new(
        id(700),
        ItemKind::File,
        r"D:\foreign".into(),
        PathKey::new("key from another provider".into()).unwrap(),
        TimestampMillis(0),
    )
    .unwrap();
    let drape = config.drapes()[0].add_item(item).unwrap();
    assert_eq!(
        save(&mut store, &config.replace_drape(drape).unwrap()),
        ConfigSave::NotCommitted(ConfigStoreError::DocumentReadOnly(
            ReadOnlyReason::UnsupportedContent
        ))
    );
    assert_eq!(files.peek_current().unwrap(), LEGACY.as_bytes());
    assert!(!files.operations().contains(&FileSystemStage::AcquireWriter));
}
