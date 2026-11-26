use drape_core::domain::Guid;
use drape_core::ports::{
    Cancellation, CommitDurability, ConfigCommit, ConfigCommitOutcome, FileSystem, FileSystemError,
    QuarantineOutcome, TemporaryCleanup,
};
use drape_test_support::config_fs::{
    FakeFailure, FakeFileSystem, FakeTemporary, FakeWriter, FileSystemStage,
};
use std::num::NonZeroUsize;

#[test]
fn old_backup_token_cannot_read_or_prune_reused_managed_name() {
    let mut fs = FakeFileSystem::new();
    let revision = fs.seed_current(b"zero".to_vec()).unwrap();
    let writer = fs.acquire_writer().unwrap();
    let mut first = synced(&mut fs, &writer, 1, b"one");
    let old = match fs.commit_temporary(
        &writer,
        &mut first,
        ConfigCommit::Replace {
            expected_revision: revision,
            backup_id: id(10),
        },
    ) {
        ConfigCommitOutcome::Committed(receipt) => receipt.backup.unwrap(),
        other => panic!("unexpected commit: {other:?}"),
    };
    let old_revision = fs
        .read_config(&old, limit(100), &Cancellation::default())
        .unwrap()
        .revision;
    fs.remove_backup(&writer, &old, &old_revision).unwrap();
    let revision = fs
        .read_config(&fs.current_config(), limit(100), &Cancellation::default())
        .unwrap()
        .revision;
    let mut second = synced(&mut fs, &writer, 2, b"two");
    let new = match fs.commit_temporary(
        &writer,
        &mut second,
        ConfigCommit::Replace {
            expected_revision: revision,
            backup_id: id(10),
        },
    ) {
        ConfigCommitOutcome::Committed(receipt) => receipt.backup.unwrap(),
        other => panic!("unexpected commit: {other:?}"),
    };
    assert_eq!(
        fs.read_config(&old, limit(100), &Cancellation::default()),
        Err(FileSystemError::InvalidToken)
    );
    let new_revision = fs
        .read_config(&new, limit(100), &Cancellation::default())
        .unwrap()
        .revision;
    assert_eq!(
        fs.remove_backup(&writer, &old, &new_revision),
        Err(FileSystemError::InvalidToken)
    );
    assert_eq!(fs.peek_backup(id(10)), Some(b"one".to_vec()));
}

fn id(value: u8) -> Guid {
    let mut bytes = [0; 16];
    bytes[15] = value;
    Guid::from_bytes(bytes).unwrap()
}

#[test]
fn noncommit_stage_failures_are_distinct_and_preserve_managed_evidence() {
    let mut fs = FakeFileSystem::new();
    fs.inject(FakeFailure::Error(
        FileSystemStage::AcquireWriter,
        FileSystemError::AccessDenied,
    ));
    assert_eq!(
        fs.acquire_writer().unwrap_err(),
        FileSystemError::AccessDenied
    );
    let revision = fs.seed_current(b"old".to_vec()).unwrap();
    let writer = fs.acquire_writer().unwrap();
    fs.inject(FakeFailure::Error(
        FileSystemStage::ListBackups,
        FileSystemError::Unavailable,
    ));
    assert_eq!(
        fs.list_backups(limit(10), &Cancellation::default()),
        Err(FileSystemError::Unavailable)
    );
    fs.inject(FakeFailure::Error(
        FileSystemStage::Quarantine,
        FileSystemError::AccessDenied,
    ));
    assert_eq!(
        fs.quarantine_current(&writer, &revision, id(20)),
        QuarantineOutcome::NotMoved(FileSystemError::AccessDenied)
    );
    assert_eq!(current(&fs), b"old");
    let mut temporary = synced(&mut fs, &writer, 1, b"new");
    let backup = match fs.commit_temporary(
        &writer,
        &mut temporary,
        ConfigCommit::Replace {
            expected_revision: revision,
            backup_id: id(10),
        },
    ) {
        ConfigCommitOutcome::Committed(receipt) => receipt.backup.unwrap(),
        other => panic!("unexpected commit result: {other:?}"),
    };
    let backup_revision = fs
        .read_config(&backup, limit(100), &Cancellation::default())
        .unwrap()
        .revision;
    fs.inject(FakeFailure::Error(
        FileSystemStage::RemoveBackup,
        FileSystemError::AccessDenied,
    ));
    assert_eq!(
        fs.remove_backup(&writer, &backup, &backup_revision),
        Err(FileSystemError::AccessDenied)
    );
    assert_eq!(fs.peek_backup(id(10)), Some(b"old".to_vec()));
    fs.inject(FakeFailure::Error(
        FileSystemStage::ReleaseWriter,
        FileSystemError::Unavailable,
    ));
    assert_eq!(fs.release_writer(writer), Err(FileSystemError::Unavailable));
    // Failed explicit release still gets the writer token's best-effort Drop.
    let writer = fs.acquire_writer().unwrap();
    fs.release_writer(writer).unwrap();
    assert_eq!(fs.pending_failures(), 0);
}

#[test]
fn writer_drop_releases_lease_but_old_temporary_cannot_cross_generations() {
    let mut fs = FakeFileSystem::new();
    let writer = fs.acquire_writer().unwrap();
    let mut old_temporary = fs.create_temporary(&writer, id(1)).unwrap();
    drop(writer);
    let replacement_writer = fs.acquire_writer().unwrap();
    assert_eq!(
        fs.write_temporary(
            &replacement_writer,
            &mut old_temporary,
            b"bad",
            &Cancellation::default()
        ),
        Err(FileSystemError::InvalidToken)
    );
    assert_eq!(
        fs.cleanup_temporary(&replacement_writer, &mut old_temporary),
        Err(FileSystemError::InvalidToken)
    );
    assert_eq!(fs.peek_temporary(id(1)), Some(Vec::new()));
}

#[test]
fn cancelled_precommit_calls_do_not_consume_scripted_faults_or_change_bytes() {
    let mut fs = FakeFileSystem::new();
    fs.seed_current(b"committed".to_vec()).unwrap();
    let writer = fs.acquire_writer().unwrap();
    let mut temporary = fs.create_temporary(&writer, id(1)).unwrap();
    fs.inject(FakeFailure::Error(
        FileSystemStage::WriteTemporary,
        FileSystemError::AccessDenied,
    ));
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        fs.write_temporary(&writer, &mut temporary, b"new", &cancelled),
        Err(FileSystemError::Cancelled)
    );
    assert_eq!(
        fs.sync_temporary(&writer, &mut temporary, &cancelled),
        Err(FileSystemError::Cancelled)
    );
    assert_eq!(
        fs.list_backups(limit(10), &cancelled),
        Err(FileSystemError::Cancelled)
    );
    assert_eq!(fs.pending_failures(), 1);
    assert_eq!(fs.peek_temporary(id(1)), Some(Vec::new()));
    assert_eq!(current(&fs), b"committed");
    assert_eq!(
        fs.write_temporary(&writer, &mut temporary, b"new", &Cancellation::default()),
        Err(FileSystemError::AccessDenied)
    );
    assert_eq!(fs.pending_failures(), 0);
}

#[test]
fn failed_resync_invalidates_previously_synced_temporary_eligibility() {
    let mut fs = FakeFileSystem::new();
    let writer = fs.acquire_writer().unwrap();
    let mut temporary = synced(&mut fs, &writer, 1, b"candidate");
    fs.inject(FakeFailure::Error(
        FileSystemStage::SyncTemporary,
        FileSystemError::Unavailable,
    ));
    assert_eq!(
        fs.sync_temporary(&writer, &mut temporary, &Cancellation::default()),
        Err(FileSystemError::Unavailable)
    );
    assert_eq!(
        fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
        ConfigCommitOutcome::NotCommitted(FileSystemError::Conflict)
    );
    assert_eq!(fs.peek_current(), None);
    fs.sync_temporary(&writer, &mut temporary, &Cancellation::default())
        .unwrap();
    assert!(matches!(
        fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
        ConfigCommitOutcome::Committed(_)
    ));
}

fn limit(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}
fn synced(fs: &mut FakeFileSystem, writer: &FakeWriter, key: u8, bytes: &[u8]) -> FakeTemporary {
    let mut temporary = fs.create_temporary(writer, id(key)).unwrap();
    fs.write_temporary(writer, &mut temporary, bytes, &Cancellation::default())
        .unwrap();
    fs.sync_temporary(writer, &mut temporary, &Cancellation::default())
        .unwrap();
    temporary
}
fn current(fs: &FakeFileSystem) -> Vec<u8> {
    fs.read_config(&fs.current_config(), limit(1024), &Cancellation::default())
        .unwrap()
        .bytes
}

#[test]
fn bounded_reads_distinguish_missing_access_oversize_and_cancellation() {
    let mut fs = FakeFileSystem::new();
    let file = fs.current_config();
    assert_eq!(
        fs.read_config(&file, limit(10), &Cancellation::default()),
        Err(FileSystemError::NotFound)
    );
    fs.seed_current(b"existing".to_vec()).unwrap();
    assert_eq!(
        fs.read_config(&file, limit(2), &Cancellation::default()),
        Err(FileSystemError::TooLarge)
    );
    fs.inject(FakeFailure::Error(
        FileSystemStage::Read,
        FileSystemError::AccessDenied,
    ));
    assert_eq!(
        fs.read_config(&file, limit(10), &Cancellation::default()),
        Err(FileSystemError::AccessDenied)
    );
    let cancellation = Cancellation::default();
    cancellation.cancel();
    assert_eq!(
        fs.read_config(&file, limit(10), &cancellation),
        Err(FileSystemError::Cancelled)
    );
    assert_eq!(current(&fs), b"existing");
}

#[test]
fn root_provenance_rejects_foreign_files_writers_temporaries_and_revisions() {
    let mut first = FakeFileSystem::new();
    let mut second = FakeFileSystem::new();
    let first_revision = first.seed_current(b"first".to_vec()).unwrap();
    second.seed_current(b"second".to_vec()).unwrap();
    let first_writer = first.acquire_writer().unwrap();
    let second_writer = second.acquire_writer().unwrap();
    let mut temporary = first.create_temporary(&first_writer, id(1)).unwrap();
    assert_eq!(
        second.read_config(
            &first.current_config(),
            limit(100),
            &Cancellation::default()
        ),
        Err(FileSystemError::InvalidToken)
    );
    assert_eq!(
        second.create_temporary(&first_writer, id(2)).unwrap_err(),
        FileSystemError::InvalidToken
    );
    assert_eq!(
        second.write_temporary(
            &second_writer,
            &mut temporary,
            b"x",
            &Cancellation::default()
        ),
        Err(FileSystemError::InvalidToken)
    );
    let mut second_temp = synced(&mut second, &second_writer, 3, b"new");
    assert_eq!(
        second.commit_temporary(
            &second_writer,
            &mut second_temp,
            ConfigCommit::Replace {
                expected_revision: first_revision,
                backup_id: id(4),
            }
        ),
        ConfigCommitOutcome::NotCommitted(FileSystemError::InvalidToken)
    );
    assert_eq!(current(&first), b"first");
    assert_eq!(current(&second), b"second");
}

#[test]
fn shared_writer_exclusion_and_owned_temp_cleanup_reject_stale_identity() {
    let mut fs = FakeFileSystem::new();
    let mut same_root = fs.clone();
    let writer = fs.acquire_writer().unwrap();
    assert_eq!(
        same_root.acquire_writer().unwrap_err(),
        FileSystemError::Conflict
    );
    let mut temporary = fs.create_temporary(&writer, id(1)).unwrap();
    assert_eq!(
        same_root.create_temporary(&writer, id(1)).unwrap_err(),
        FileSystemError::AlreadyExists
    );
    assert_eq!(
        fs.cleanup_temporary(&writer, &mut temporary),
        Ok(TemporaryCleanup::Removed)
    );
    assert_eq!(
        fs.cleanup_temporary(&writer, &mut temporary),
        Ok(TemporaryCleanup::AlreadyAbsent)
    );
    let mut replacement = same_root.create_temporary(&writer, id(1)).unwrap();
    assert_eq!(
        fs.cleanup_temporary(&writer, &mut temporary),
        Err(FileSystemError::InvalidToken)
    );
    assert_eq!(same_root.peek_temporary(id(1)), Some(Vec::new()));
    same_root
        .cleanup_temporary(&writer, &mut replacement)
        .unwrap();
    fs.release_writer(writer).unwrap();
    let next = same_root.acquire_writer().unwrap();
    same_root.release_writer(next).unwrap();
}

#[test]
fn write_sync_eligibility_is_invalidated_by_partial_rewrite_and_commit_replay() {
    let mut fs = FakeFileSystem::new();
    let writer = fs.acquire_writer().unwrap();
    let mut temporary = fs.create_temporary(&writer, id(1)).unwrap();
    fs.write_temporary(
        &writer,
        &mut temporary,
        b"complete",
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(
        fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
        ConfigCommitOutcome::NotCommitted(FileSystemError::Conflict)
    );
    fs.sync_temporary(&writer, &mut temporary, &Cancellation::default())
        .unwrap();
    fs.inject(FakeFailure::PartialWrite {
        bytes: 2,
        error: FileSystemError::Unavailable,
    });
    assert_eq!(
        fs.write_temporary(
            &writer,
            &mut temporary,
            b"rewrite",
            &Cancellation::default()
        ),
        Err(FileSystemError::Unavailable)
    );
    assert_eq!(fs.peek_temporary(id(1)), Some(b"re".to_vec()));
    assert_eq!(
        fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
        ConfigCommitOutcome::NotCommitted(FileSystemError::Conflict)
    );
    fs.write_temporary(&writer, &mut temporary, b"final", &Cancellation::default())
        .unwrap();
    fs.sync_temporary(&writer, &mut temporary, &Cancellation::default())
        .unwrap();
    assert!(matches!(
        fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
        ConfigCommitOutcome::Committed(_)
    ));
    assert_eq!(
        fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
        ConfigCommitOutcome::NotCommitted(FileSystemError::Conflict)
    );
    assert_eq!(current(&fs), b"final");
}

#[test]
fn replacement_keeps_exact_previous_bytes_and_first_create_never_overwrites() {
    let mut fs = FakeFileSystem::new();
    let old_revision = fs.seed_current(b"old-exact-bytes".to_vec()).unwrap();
    let writer = fs.acquire_writer().unwrap();
    let mut temporary = synced(&mut fs, &writer, 1, b"new");
    assert_eq!(
        fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
        ConfigCommitOutcome::NotCommitted(FileSystemError::AlreadyExists)
    );
    let receipt = match fs.commit_temporary(
        &writer,
        &mut temporary,
        ConfigCommit::Replace {
            expected_revision: old_revision.clone(),
            backup_id: id(10),
        },
    ) {
        ConfigCommitOutcome::Committed(receipt) => receipt,
        other => panic!("unexpected commit outcome: {other:?}"),
    };
    assert_eq!(receipt.durability, CommitDurability::Confirmed);
    assert_ne!(receipt.revision, old_revision);
    let backup = receipt.backup.unwrap();
    assert_eq!(
        fs.read_config(&backup, limit(100), &Cancellation::default())
            .unwrap()
            .bytes,
        b"old-exact-bytes"
    );
    assert_eq!(current(&fs), b"new");
}

#[test]
fn backup_collision_and_external_revision_change_preserve_all_existing_bytes() {
    let mut fs = FakeFileSystem::new();
    let original = fs.seed_current(b"zero".to_vec()).unwrap();
    let writer = fs.acquire_writer().unwrap();
    let mut first = synced(&mut fs, &writer, 1, b"one");
    assert!(matches!(
        fs.commit_temporary(
            &writer,
            &mut first,
            ConfigCommit::Replace {
                expected_revision: original,
                backup_id: id(10),
            }
        ),
        ConfigCommitOutcome::Committed(_)
    ));
    let revision = fs
        .read_config(&fs.current_config(), limit(100), &Cancellation::default())
        .unwrap()
        .revision;
    let mut second = synced(&mut fs, &writer, 2, b"two");
    assert_eq!(
        fs.commit_temporary(
            &writer,
            &mut second,
            ConfigCommit::Replace {
                expected_revision: revision.clone(),
                backup_id: id(10),
            }
        ),
        ConfigCommitOutcome::NotCommitted(FileSystemError::AlreadyExists)
    );
    assert_eq!(current(&fs), b"one");
    fs.seed_current(b"external".to_vec()).unwrap();
    assert_eq!(
        fs.commit_temporary(
            &writer,
            &mut second,
            ConfigCommit::Replace {
                expected_revision: revision,
                backup_id: id(11),
            }
        ),
        ConfigCommitOutcome::NotCommitted(FileSystemError::Conflict)
    );
    assert_eq!(current(&fs), b"external");
    assert_eq!(fs.peek_backup(id(10)), Some(b"zero".to_vec()));
    assert_eq!(fs.peek_backup(id(11)), None);
}

#[test]
fn bounded_backup_listing_returns_newest_first_without_truncation_and_prunes_by_revision() {
    let mut fs = FakeFileSystem::new();
    fs.seed_current(b"zero".to_vec()).unwrap();
    let writer = fs.acquire_writer().unwrap();
    for (temporary_id, backup_id, bytes) in [(1, 10, b"one".as_slice()), (2, 11, b"two".as_slice())]
    {
        let revision = fs
            .read_config(&fs.current_config(), limit(100), &Cancellation::default())
            .unwrap()
            .revision;
        let mut temporary = synced(&mut fs, &writer, temporary_id, bytes);
        assert!(matches!(
            fs.commit_temporary(
                &writer,
                &mut temporary,
                ConfigCommit::Replace {
                    expected_revision: revision,
                    backup_id: id(backup_id),
                }
            ),
            ConfigCommitOutcome::Committed(_)
        ));
    }
    assert_eq!(
        fs.list_backups(limit(1), &Cancellation::default()),
        Err(FileSystemError::TooLarge)
    );
    let backups = fs.list_backups(limit(2), &Cancellation::default()).unwrap();
    let observed: Vec<_> = backups
        .iter()
        .map(|file| {
            fs.read_config(file, limit(100), &Cancellation::default())
                .unwrap()
                .bytes
        })
        .collect();
    assert_eq!(observed, vec![b"one".to_vec(), b"zero".to_vec()]);
    let backup = fs
        .read_config(&backups[1], limit(100), &Cancellation::default())
        .unwrap();
    fs.remove_backup(&writer, &backups[1], &backup.revision)
        .unwrap();
    assert_eq!(
        fs.list_backups(limit(2), &Cancellation::default())
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn quarantine_collision_preserves_source_and_wrong_kind_cannot_be_pruned() {
    let mut fs = FakeFileSystem::new();
    let revision = fs.seed_current(b"corrupt-original".to_vec()).unwrap();
    let writer = fs.acquire_writer().unwrap();
    let quarantine = match fs.quarantine_current(&writer, &revision, id(10)) {
        QuarantineOutcome::Moved(file) => file,
        other => panic!("unexpected quarantine result: {other:?}"),
    };
    assert_eq!(fs.peek_current(), None);
    let new_revision = fs.seed_current(b"new-corrupt".to_vec()).unwrap();
    assert_eq!(
        fs.quarantine_current(&writer, &new_revision, id(10)),
        QuarantineOutcome::NotMoved(FileSystemError::AlreadyExists)
    );
    assert_eq!(current(&fs), b"new-corrupt");
    let preserved = fs
        .read_config(&quarantine, limit(100), &Cancellation::default())
        .unwrap();
    assert_eq!(preserved.bytes, b"corrupt-original");
    assert_eq!(
        fs.remove_backup(&writer, &quarantine, &preserved.revision),
        Err(FileSystemError::InvalidToken)
    );
}

#[test]
fn indeterminate_commit_preserves_evidence_and_latches_mutation_stop_before_or_after_publication() {
    for publish in [false, true] {
        let mut fs = FakeFileSystem::new();
        let revision = fs.seed_current(b"old".to_vec()).unwrap();
        let writer = fs.acquire_writer().unwrap();
        let mut temporary = synced(&mut fs, &writer, 1, b"new");
        fs.inject(FakeFailure::CommitIndeterminate {
            publish,
            error: FileSystemError::Unavailable,
        });
        assert_eq!(
            fs.commit_temporary(
                &writer,
                &mut temporary,
                ConfigCommit::Replace {
                    expected_revision: revision,
                    backup_id: id(10),
                }
            ),
            ConfigCommitOutcome::Indeterminate(FileSystemError::Unavailable)
        );
        assert!(fs.is_stopped());
        assert_eq!(
            current(&fs),
            if publish {
                b"new".to_vec()
            } else {
                b"old".to_vec()
            }
        );
        assert_eq!(fs.peek_temporary(id(1)), Some(b"new".to_vec()));
        assert_eq!(fs.peek_backup(id(10)), publish.then(|| b"old".to_vec()));
        assert_eq!(
            fs.cleanup_temporary(&writer, &mut temporary),
            Err(FileSystemError::Conflict)
        );
        assert_eq!(
            fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew),
            ConfigCommitOutcome::NotCommitted(FileSystemError::Conflict)
        );
        assert_eq!(
            fs.create_temporary(&writer, id(2)).unwrap_err(),
            FileSystemError::Conflict
        );
    }
}

#[test]
fn indeterminate_quarantine_stops_recovery_for_both_possible_namespace_results() {
    for move_file in [false, true] {
        let mut fs = FakeFileSystem::new();
        let revision = fs.seed_current(b"corrupt".to_vec()).unwrap();
        let writer = fs.acquire_writer().unwrap();
        fs.inject(FakeFailure::QuarantineIndeterminate {
            move_file,
            error: FileSystemError::Unavailable,
        });
        assert_eq!(
            fs.quarantine_current(&writer, &revision, id(10)),
            QuarantineOutcome::Indeterminate(FileSystemError::Unavailable)
        );
        assert!(fs.is_stopped());
        assert_eq!(fs.peek_current(), (!move_file).then(|| b"corrupt".to_vec()));
        assert_eq!(
            fs.peek_quarantine(id(10)),
            move_file.then(|| b"corrupt".to_vec())
        );
        assert_eq!(
            fs.quarantine_current(&writer, &revision, id(11)),
            QuarantineOutcome::NotMoved(FileSystemError::Conflict)
        );
        assert_eq!(
            fs.create_temporary(&writer, id(2)).unwrap_err(),
            FileSystemError::Conflict
        );
    }
}

#[test]
fn each_precommit_failure_stage_keeps_previous_config_and_existing_backup() {
    for stage in [
        FileSystemStage::CreateTemporary,
        FileSystemStage::WriteTemporary,
        FileSystemStage::SyncTemporary,
        FileSystemStage::CommitTemporary,
    ] {
        let mut fs = FakeFileSystem::new();
        let initial_revision = fs.seed_current(b"older".to_vec()).unwrap();
        let writer = fs.acquire_writer().unwrap();
        let mut baseline = synced(&mut fs, &writer, 90, b"old");
        let revision = match fs.commit_temporary(
            &writer,
            &mut baseline,
            ConfigCommit::Replace {
                expected_revision: initial_revision,
                backup_id: id(9),
            },
        ) {
            ConfigCommitOutcome::Committed(receipt) => receipt.revision,
            other => panic!("unexpected baseline commit: {other:?}"),
        };
        fs.inject(FakeFailure::Error(stage, FileSystemError::AccessDenied));
        let outcome = (|| {
            let mut temporary = fs.create_temporary(&writer, id(1))?;
            fs.write_temporary(&writer, &mut temporary, b"new", &Cancellation::default())?;
            fs.sync_temporary(&writer, &mut temporary, &Cancellation::default())?;
            Ok::<_, FileSystemError>(fs.commit_temporary(
                &writer,
                &mut temporary,
                ConfigCommit::Replace {
                    expected_revision: revision,
                    backup_id: id(10),
                },
            ))
        })();
        match outcome {
            Err(error) => assert_eq!(error, FileSystemError::AccessDenied),
            Ok(value) => assert_eq!(
                value,
                ConfigCommitOutcome::NotCommitted(FileSystemError::AccessDenied)
            ),
        }
        assert_eq!(current(&fs), b"old");
        assert_eq!(fs.peek_backup(id(9)), Some(b"older".to_vec()));
        assert_eq!(fs.peek_backup(id(10)), None);
        assert!(fs.operations().contains(&stage));
        assert_eq!(fs.pending_failures(), 0);
    }
}

#[test]
fn durability_and_cleanup_failures_cannot_undo_or_misreport_successful_commit() {
    let mut fs = FakeFileSystem::new();
    let writer = fs.acquire_writer().unwrap();
    let mut temporary = synced(&mut fs, &writer, 1, b"published");
    fs.inject(FakeFailure::CommitDurability(FileSystemError::Unavailable));
    let receipt = match fs.commit_temporary(&writer, &mut temporary, ConfigCommit::CreateNew) {
        ConfigCommitOutcome::Committed(receipt) => receipt,
        other => panic!("unexpected commit outcome: {other:?}"),
    };
    assert_eq!(
        receipt.durability,
        CommitDurability::Unconfirmed(FileSystemError::Unavailable)
    );
    assert!(receipt.backup.is_none());
    fs.inject(FakeFailure::Error(
        FileSystemStage::CleanupTemporary,
        FileSystemError::AccessDenied,
    ));
    assert_eq!(
        fs.cleanup_temporary(&writer, &mut temporary),
        Err(FileSystemError::AccessDenied)
    );
    assert_eq!(current(&fs), b"published");
    assert_eq!(
        fs.cleanup_temporary(&writer, &mut temporary),
        Ok(TemporaryCleanup::AlreadyAbsent)
    );
    let cancellation = Cancellation::default();
    cancellation.cancel();
    assert_eq!(
        fs.read_config(&fs.current_config(), limit(100), &cancellation),
        Err(FileSystemError::Cancelled)
    );
    assert_eq!(current(&fs), b"published");
}
