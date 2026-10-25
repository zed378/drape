# Feature Spec — P1-03 ConfigStore: Atomic Save, Backup, Quarantine, Migration

**Written:** 2026-10-25 — **after** implementation (retrospective; the card's DoD requires the spec, and the implementation already exists from 2026-10-23)
**Task:** P1-03
**Author:** code agent
**Spec refs:** [`docs/ARCHITECTURE/02-DATA-MODEL.md`](../../docs/ARCHITECTURE/02-DATA-MODEL.md) · ADR-005 in [`MEMORY/DECISIONS.md`](../DECISIONS.md)

---

## Problem

A half-written `config.json` loses the user's layout. The store must survive a crash mid-write, a corrupt file, and a schema change — without ever deleting user data.

---

## What `docs/` Already Decides

| Decision | Source |
|---|---|
| One atomic `config.json`, schema version, backups, quarantine | ADR-005 |
| Write sequence: temp file → flush → replace; last 10 backups | DATA-MODEL §Atomic Writes |
| Corrupt file → quarantine → newest valid backup → empty config | DATA-MODEL §On load |
| Migration is a pure function `Migrate(vN) → v(N+1)` with one JSON fixture per version | DATA-MODEL §Schema Versioning |
| Unknown properties preserved via `JsonExtensionData` | DATA-MODEL §Schema Versioning |
| Newer schema opens read-only, never overwritten | DATA-MODEL §Schema Versioning |
| Config contains user file paths: personal data, never transmitted | DATA-MODEL §Data Privacy |

**No gap found.** `docs/` settles everything this task needs.

---

## Layering and Interfaces

- [x] `Drape.Core`: `IConfigStore`, `ConfigLoadResult`, `ConfigLoadOutcome` — no I/O, no UI, no platform types.
- [x] `Drape.Infrastructure/Configuration/`: `FileConfigStore` (the `IConfigStore`), `ConfigJson` (DTOs + serialisation), `ConfigMigrations` (migration table + `Reconcile`).
- [x] No platform type crosses a boundary. No native handle is created.
- [x] No capability gating needed — the store works on every platform through `IFileSystem`.

---

## Desktop Safety

- [x] **Icon hide path:** none. The store never touches icons.
- [x] **Restore paths:** not applicable.
- [x] Non-destructive: the store never moves, renames or deletes a user file. It only writes inside its own data directory. A corrupt config is *moved* to `quarantine\`, never deleted — asserted by `LoadAsync_WithCorruptConfig_DoesNotDeleteIt`.
- [x] Failure mode: a failed save throws (`IOException`); the live file is untouched because the temp file is never promoted. A corrupt load recovers from backup or starts empty, and tells the user why.

---

## Data and Config

| Field | Type | Default | Notes |
|---|---|---|---|
| `schemaVersion` | int | 1 | Bumped only by an explicit migration |
| backups | files | 10 kept | `backup\config-YYYYMMDD-HHmmss.json`, pruned oldest-first |
| quarantine | files | kept | `quarantine\config-YYYYMMDD-HHmmss.json`, never auto-deleted |

- [x] Schema version bump: v1 is the first schema; `ConfigMigrations.Migrate` has the table shape with a v1→v2 placeholder. A real v2 must add a hand-written `config-v1.json` fixture.
- [x] Writes stay atomic (temp, flush, replace); unknown root properties preserved via `_unknownRoot`.
- [x] Nothing derived is stored.
- [x] Does not touch `state.json`.
- [x] ADR-011 enforced in `DrapeConfig` construction, not in the store.

---

## Behaviour

| Rule | Enforcement point |
|---|---|
| A save never leaves a half-written live file | `FileConfigStore.Write`: temp → replace; test `SaveAsync_WhenReplaceFails_LeavesOldConfigIntact` |
| A corrupt file is quarantined, never deleted | `Quarantine()` moves; test `LoadAsync_WithCorruptConfig_DoesNotDeleteIt` |
| Newest valid backup is used when the live file is corrupt | `FindNewestValidBackup`; test `LoadAsync_WithCorruptConfig_QuarantinesAndUsesBackup` |
| No valid backup → empty config, user told why | `Recover()`; test `LoadAsync_WithCorruptConfigAndNoBackup_StartsEmpty` |
| Newer schema → read-only, save refused | `IsReadOnly` flag; test `LoadAsync_WithNewerSchema_IsReadOnly` |
| A document that parses but violates a domain rule is corrupt | `TryParse` catches `ArgumentException`; test `LoadAsync_WithDomainViolation_TreatsAsCorrupt` |
| Unknown root properties survive a round-trip | `_unknownRoot` retained; test `SaveThenLoad_PreservesUnknownProperties` |
| Backups pruned to 10 | `PruneBackups`; test `SaveAsync_PrunesBackupsBeyondLimit` |

---

## Platform Matrix

| Target | Applies? | Notes / how verified |
|---|---|---|
| Windows 11 (23H2, 24H2+) | Partial | Logic tested via `FakeFileSystem`; real-disk timing measured on dev machine (see record) |
| Windows 10 22H2 (minimum) | No | Same code path (`IFileSystem`); no OS-specific branch |
| DPI / monitors | Not applicable | The store is DPI- and monitor-independent |
| macOS / Linux / Android / iOS | not applicable until Phase 6 | `IFileSystem` is the seam; no platform code in the store |

---

## Security and Privacy

- [x] No network call.
- [x] Config contains user file paths (personal data). Never transmitted. Never logged (CS-37: paths only at Debug).
- [x] No new process, DLL or injection.
- [x] No elevation.

---

## UI

Not applicable — no UI in this task. The user-visible message (`ConfigLoadResult.Message`) contains no file path.

---

## Tests

- Unit (Core): `ConfigPersistenceTests` — debounce, flush-before-due, last-write-wins, failed-save-throws.
- Integration (Infrastructure, `FakeFileSystem`): `FileConfigStoreTests` — 13 tests, all named in the record.
- UI / headless: none.
- E2E: none — the store is below the UI.
- Negative cases: corrupt JSON, domain-violating JSON, unreadable file, failed replace, failed write, newer schema, no backup, backup pruning.
- Mutation check: write-in-place instead of replace → `SaveAsync_WhenReplaceFails_LeavesOldConfigIntact` fails. Observed 2026-10-23, then reverted.

---

## What Was Not Determined

- The 20 ms budget for 50 Drapes × 50 items had no measurement at spec time. Measured 2026-10-25 on the dev machine; see the record.
- Real-disk behaviour under power loss (as opposed to a simulated `IOException`) cannot be tested in CI; the atomicity argument rests on `File.Replace` semantics.

---

## Open Questions

| Question | Default assumption | Backlog entry |
|---|---|---|
| None | — | — |
