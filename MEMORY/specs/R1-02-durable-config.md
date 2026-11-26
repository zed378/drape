# Feature Spec - R1-02 Durable config and legacy schema compatibility

**Written:** 2026-11-25 - **before** implementation  
**Task:** R1-02  
**Author:** orchestrator / architect  
**Spec refs:** [data model](../../docs/ARCHITECTURE/02-DATA-MODEL.md), [system architecture](../../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md), [Rust migration](../../docs/ARCHITECTURE/03-RUST-MIGRATION.md), [testing strategy](../../docs/TESTING/00-TESTING-STRATEGY.md), [coding standards](../../docs/ENGINEERING/00-CODING-STANDARDS.md), ADR-005 and ADR-025

## Problem

The Rust application must load and save the user's existing schema-1 configuration without losing layout, paths, unknown fields or unsupported values. An interrupted or failed write must leave the last committed configuration recoverable. This task covers config persistence only; icon-recovery state has a separate safety protocol.

## What `docs/` Already Decides

| Decision | Source |
|---|---|
| Preserve schema version 1, JSON field names, enum spellings, IDs, timestamps and existing data location; a language rewrite does not bump the schema | DATA-MODEL, Rust migration |
| JSON parsing and persistence adapters stay outside Core; migrations and validation are pure Core logic; App injects platform-selected filesystem support | System architecture, Rust migration |
| Writes use one writer, an exclusive same-directory temporary file, flush/sync, then an OS-aware replace; only committed saves may prune backups | DATA-MODEL, ADR-005 |
| Keep the last 10 valid backups; corrupt config is preserved in app-managed quarantine before backup recovery; no valid backup yields an empty config with a visible recovery result | DATA-MODEL |
| Newer schema is read-only and never quarantined or overwritten | DATA-MODEL |
| Preserve unknown root and nested properties and unknown enum/discriminator values; use read-only mode if lossless save cannot be established | DATA-MODEL |
| Use hand-written fixtures, explicit input limits and injected failure tests; user paths are private data and never transmitted | DATA-MODEL, TESTING, SECURITY |
| Save debounce is 500 ms and normal shutdown flushes synchronously; persistence failure must not block desktop restoration | DATA-MODEL, system architecture, ADR-026 |

The historical P1-03 spec and C# tests are reference material only. They do not establish Rust behavior. The Rust Core's `DomainConfig` is not a lossless wire document; it must not be serialized directly as a round-trip representation.

### Legacy fixture inventory

Fixtures must cover the actual schema-1 writer as well as the abridged documentation example. The legacy [ConfigJson](../../src/Drape.Infrastructure/Configuration/ConfigJson.cs) uses camelCase properties, flat `settings.quickHideEnabled`, lowercase enum names such as `oneline`/`twolines` and `dateadded`, and omits null properties including nullable `state.zOrderHint`. The documentation example instead shows `settings.quickHide.enabled` and `labelMode: "two-lines"`. Preserve supported input representations through the lossless wire layer; do not silently rename, default or discard them through `DomainConfig` conversion. Define safe handling for conflicting alias forms and case-variant properties accepted by the legacy reader before enabling saves.

Legacy `addedAt`/`missingSince` values use `DateTimeOffset`; fixtures must exercise offsets and fractional-second precision beyond Core's millisecond projection. The [legacy store tests](../../tests/Drape.Infrastructure.Tests/FileConfigStoreTests.cs) include a 50-Drape by 50-item serialization profile (2,500 references). Compare input budgets with that profile as well as the 500-desktop-item matrix; neither is permission to truncate existing data. The legacy store's forced current-schema save and second-resolution backup/quarantine names are not safe Rust defaults: retain future-schema read-only behavior and test name collisions explicitly. These are source-grounded compatibility prerequisites, not Rust migration evidence or a schema change.

## Layering and Interfaces

- `drape-core`: pure schema-version transitions, known-field validation and neutral `ConfigStore`/`FileSystem` contracts. No parsing, filesystem calls, native types, UI types or platform selection.
- `drape-infrastructure`: bounded wire parsing/serialization, unknown-field preservation, load/save coordination, migration orchestration, backup and quarantine policy. It receives all filesystem operations through injected ports.
- `drape-shell-windows`: selected filesystem adapter, including Windows replacement semantics. Native I/O and any unsafe code stay in this adapter with reviewed RAII and safety comments.
- `drape-app`: selects and injects the adapter and config location. No OS checks in Core or shared UI.
- `drape-test-support`: deterministic in-memory filesystem with operation-level failure injection; no real user paths or files.

The Core `FileSystem` port must express bounded reads, exclusive temporary creation, complete writes, flush/sync, atomic replace or validated first create, listing managed backups, moving an app-managed corrupt config into quarantine without overwrite, and cleanup of the writer's own temporary file. Paths passed to operations are rooted beneath the injected Drape config directory; implementations reject escape/reparse traversal. The final method signatures and any new Core interface receive architect review before implementation. No blanket `Send + Sync` is required.

The wire representation must remain separate from domain structs. It must preserve unknown values recursively and retain numeric precision, including integers outside the IEEE-754 exact range. The selected parser/representation must prove this with fixtures before round-trip saving is enabled. Serde/JSON dependency versions and features are a later locked workspace change, reviewed by the architect; this spec does not alter manifests.

### R1-01 filesystem contract handoff

The interface-only increment in [Core ports](../../crates/drape-core/src/ports.rs) defines `FileSystem`; it does not implement a parser, ConfigStore or native filesystem adapter. Associated `File`, `Revision`, `Writer` and `Temporary` tokens are validated against the injected root/adapter and expose no native handles or arbitrary paths. This permits recognizing existing legacy backup names without renaming them. A writer excludes cooperating Drape config writers, including other processes; it does not replace the App's session/single-instance protocol. Revision check followed by replacement is not atomic compare-and-swap against an arbitrary external editor. The native adapter must document those residual windows, detect conflicts where possible and reject unsupported stronger guarantees before mutation.

| Requirement | Contract |
|---|---|
| Bounded stable read, distinct missing/access/oversize outcomes | `read_config` with `NonZeroUsize`, owned `ConfigRead` bytes/revision and `FileSystemError` |
| Bounded backup enumeration without silent truncation | `list_backups`, newest first with a stable adapter-defined tie-break; scan/retention bound checked before allocation |
| Exclusive ownership of temporary output | `acquire_writer`, `create_temporary` with caller-supplied GUID; collisions return `AlreadyExists` |
| Separate injected write/sync failures | `write_temporary`, `sync_temporary`; adapters track eligibility and reject a partial/unsynced/replayed commit |
| Distinct no-overwrite first creation and revision-checked replacement | `ConfigCommit::CreateNew` / `Replace`; replacement preserves previously validated bytes in an exclusive backup |
| Namespace commit versus durability versus cleanup | `ConfigCommitOutcome`, `ConfigCommitReceipt`, `CommitDurability`; prune/cleanup errors stay separate from commit |
| Collision-safe quarantine and safe pruning | `quarantine_current` with `QuarantineOutcome`, `remove_backup` with expected revision; only recognized app-managed artefacts are eligible |
| Cleanup only what this writer owns | `cleanup_temporary` returns `TemporaryCleanup`; foreign replacement tokens fail validation |

Per [ADR-030](../DECISIONS.md), `NotCommitted` guarantees the previous destination and existing backups are unchanged. `Committed` identifies a successful namespace change even if later durability confirmation fails. `Indeterminate` is an explicit native-adapter fault state: retain all evidence, stop saves, and do not automatically retry, prune or clean the temporary artefact until reconciliation establishes what committed. It is never a successful durability acceptance result. Native acceptance must still demonstrate recoverability of the last committed valid bytes at every supported failure point; this interface does not waive that requirement.

Quarantine reports `Moved`, `NotMoved` or `Indeterminate` under the same principle. Only `NotMoved` guarantees that the source remained unchanged. An indeterminate move stops backup recovery and any overwrite/retry until reconciliation establishes the source and quarantine identities; it cannot be treated as successfully preserving corrupt input.

Calls run on the selected synchronous persistence worker, with cooperative cancellation before mutation and no blanket `Send + Sync`. Commit has no cancellation argument: once started, its actual outcome is observed even if the request is cancelled or superseded. Shutdown stops admission, waits for native completion, resolves the outcome, cleans only owned temporaries when safe, then releases writer ownership. Cancellation after a successful commit must not report that the save did not happen. Desktop restoration remains independent of all persistence shutdown work.

Parser budgets, alias-conflict handling, lossless wire representation, ConfigStore service types and the native adapter remain later prerequisites. No numeric parser defaults, JSON dependencies, schema fields or recovery-state protocol are selected by these interfaces.

## Desktop Safety

- Icon hiding: none. This task never calls desktop-host or icon-visibility APIs.
- Restore paths and Guard: untouched. Config save/load failure must return to the app lifecycle and must never delay or prevent restoration.
- User files referenced by config are never moved, renamed, opened, deleted or hydrated. Quarantine and backups may contain only Drape-managed config files beneath the injected app-data root.
- No Explorer, registry, process or global-hook operations.
- Failure to write or replace leaves the previous committed config and valid backups intact. Failure to preserve corrupt input must not silently overwrite it with defaults.

## Data and Config

| Field | Type | Default | Notes |
|---|---|---|---|
| `schemaVersion` | integer | `1` | Preserve current schema; no language-only bump |
| `settings`, `pages`, `drapes`, `rules` | schema-1 JSON values | existing validated defaults only when no recoverable config exists | Explicit wire names and enum values |
| extension properties | raw JSON values | none | Preserve recursively and without numeric precision loss |
| backups | app-managed files | newest 10 | Prune only after verified commit |
| quarantine | app-managed files | retained | Never overwrite an existing quarantine entry |

`state.json`, Guard journals, recovery ownership, icon visibility and runtime process identity are explicitly out of scope. Do not apply config parsing/backup behavior to recovery state; R0-05 owns that protocol. No schema additions for pinning, tabs or portals; Q-23 remains a separate prerequisite.

All input is bounded before unbounded allocation: document bytes, nesting depth, collection counts, string lengths and unknown-value size have explicit limits. Limits must be checked against the existing 500-item testing profile and historical config corpus before implementation is accepted. Oversize input is preserved and reported; it is never truncated, quarantined as corrupt, or overwritten by an empty config merely because it exceeded a limit.

## Behaviour

| Rule | Enforcement point |
|---|---|
| A single writer creates a unique exclusive temp file in the config directory | Config-store coordinator plus concurrent-writer/fake tests |
| A failed write/sync and `NotCommitted` replace cannot change the committed destination; indeterminate native failure preserves evidence and stops further mutation | `FileSystem` contract and failure-injection tests at each operation; ADR-030, native recoverability acceptance remains required |
| Windows replacement uses the reviewed OS-aware adapter; first create cannot overwrite an unexpected destination | Windows adapter contract tests and manual/native review |
| Backups represent previously committed valid bytes and are pruned only after a successful replacement | Infrastructure tests with a fake filesystem |
| Corrupt/truncated/schema-invalid current config is preserved in a unique quarantine entry before trying backups | Load coordinator; collision and move-failure tests |
| Newer schema is returned read-only and its bytes remain unchanged | Wire-version gate before migration/quarantine; load and save refusal tests |
| Unknown nested fields, unsupported values and large numeric values survive a supported-schema round trip byte-semantically as values | Hand-written v1 fixture and exact-value assertions |
| A supported v1 migration/repair is deterministic and obtains no ID/time from the environment | Pure Core tests with caller-supplied inputs |
| A newer/unknown value that cannot be saved losslessly makes the document read-only instead of silently dropping data | Wire adapter and negative round-trip tests |
| No valid config or backup produces a typed empty/recovery result without touching Desktop items | Infrastructure integration tests |
| Shutdown flush is synchronous; failed persistence remains visible to the caller and cannot block the independent restore sequence | Core service tests and lifecycle handoff contract |

The exact filesystem guarantees remain bounded by OS/filesystem behavior. `sync_all` plus atomic replacement protects against ordinary process failure, but does not establish absolute power-loss immunity on every storage device. The implementation and record must state the achieved guarantees precisely.

## Platform Matrix

| Target | Applies? | Notes / how verified |
|---|---|---|
| Windows 11 | Yes | Fake-backed logic plus Windows adapter review; exact build for native replacement tests |
| Windows 10 22H2 | Yes | Release minimum; native replacement behavior must be checked on this OS before release |
| DPI / monitors | Not applicable | Persistence logic is display-independent; layout values remain logical coordinates |
| macOS / Linux / Android / iOS | Deferred | Phase 6; no empty adapters or support claim in this task |

## Security and Privacy

- No network calls, telemetry, elevation or new process.
- Config and backups contain personal file paths. They remain local and are not included in logs, test output, fixtures from real users or reports.
- Parse sizes/counts/depth before large allocations. Treat config as untrusted input; do not execute paths or shell verbs.
- Filesystem operations are confined to the injected Drape-owned config root. They cannot target Desktop content or follow a path outside that root.

## UI

No UI is implemented. The service returns typed loaded/recovered/read-only/error state for a later UI to present. User-facing recovery text must omit file paths; localization and accessibility belong to the integrating task.

## Tests

- Core unit: `migration_v1_is_deterministic_and_preserves_supplied_identity_and_time`, `migration_rejects_invalid_identity_without_overwriting_unrelated_records`, `future_schema_domain_edits_remain_read_only`.
- Infrastructure integration: `save_failure_at_each_stage_keeps_previous_config_and_backup`, `successful_replace_prunes_only_after_commit`, `corrupt_config_is_quarantined_without_overwriting_prior_entry`, `backup_recovery_uses_newest_valid_document`, `missing_valid_config_returns_empty_without_touching_references`, `future_schema_is_read_only_and_bytes_remain_unchanged`, `schema_v1_round_trip_preserves_nested_unknown_values_and_numeric_precision`, `oversized_document_is_preserved_and_never_replaced`, `failed_quarantine_never_falls_through_to_overwrite`.
- Fixtures: hand-written schema-1 config with nested unknown fields, unsupported enum/discriminator values, non-ASCII/UNC/long literal paths and integers above 2^53; no fixture generated by the migrator.
- Architecture: actual Cargo dependency/source-boundary mutation tests ensure Core performs no I/O and Infrastructure has no direct shell/native dependency.
- Negative cases: failed exclusive create, partial write, sync failure, replace failure, first-create collision, backup enumeration failure, quarantine-name collision, invalid/truncated JSON, invalid domain state, future version, unknown values and resource limits.
- Mutation check: replacing the atomic replacement operation with write-in-place must fail `save_failure_at_each_stage_keeps_previous_config_and_backup`.
- UI/E2E: none. Native Windows filesystem replacement gets adapter-level tests; Explorer is never restarted by automation.

## What Was Not Determined

- Exact numeric resource limits must be selected after comparing the 500-item desktop profile, legacy 2,500-reference serialization profile and hand-written sanitized legacy-format fixtures; no sanitized config corpus has yet been established. Oversized bytes are preserved and never silently replaced.
- The storage device's power-loss guarantees cannot be inferred from process-level failure injection.
- Windows 10/11 adapter behavior requires named OS builds and different-role review; fake filesystems cannot prove native replacement semantics.
- Recovery `state.json` compatibility, Guard coordination and concurrent restore are owned by R0-05, not this task.

## Open Questions

| Question | Default assumption | Backlog entry |
|---|---|---|
| None requiring an owner decision for config-only implementation | Keep schema 1 and preserve unknown data losslessly; unresolved limits are implementation evidence, not a product-scope choice | None |
