# Icon cache and snapshot validation continuation

Date: 2026-11-25. Authority: owner requested continued parallel implementation. Status: validation in progress; no commit, merge or DONE implied.

## Ownership

- `core_snapshot` reviewed the existing R1-01 immutable snapshot implementation, then performed read-only architect review of R1-02 prerequisites.
- `shell_icons` owns `crates/drape-shell-windows/src/`, extending R0-06 with a bounded worker-owned icon cache and conversion tests.
- `qa_queue` is the sole sequential validation owner. Implementation agents execute no verification commands.
- The orchestrator owns task tracking, continuity and supported persistence-spec clarifications. Shared manifests and legacy source are preserved.

Codebase-memory MCP failed during initialization. Targeted local source reads were used as the documented fallback; no graph results are claimed.

## Implementation and review

The cache retains validated owned `IconBitmap` values, promotes hits to most recently used, evicts the least recently used entry to satisfy both pixel-byte and entry-count limits, and rejects oversized/invalid replacements without discarding existing data. Keys distinguish literal item identity, physical pixel size and caller-supplied revision. Item invalidation removes every size/revision; clear resets byte accounting. Identities are bounded to 64 KiB. Incoming buffers are compacted rather than retaining arbitrary caller capacity.

The byte budget covers retained pixel payload, not total process memory. Entry count bounds bookkeeping and separately bounded keys. No production cache budget is selected, no worker/executor or UI conversion is wired, and no native raw BGRX result is treated as a verified alpha-bearing `IconBitmap`.

Independent architect source review found the passive `DrapeSnapshot`/`LayoutSnapshot` subset consistent with the approved contract: owned private fields, supplied IDs/time, preserved original bounds/style/item order, blank-name rejection and duplicate Drape/item/membership rejection. Empty layouts are valid. Persistence, restore and preview workflows remain outside this subset.

## Persistence handoff

The [R1-02 spec](../specs/R1-02-durable-config.md) now inventories actual legacy writer shapes, nullable fields, case-insensitive reads and timestamp precision; [BACKLOG](../../TASKS/BACKLOG.md) records the unresolved alias/limit prerequisite. Evidence: `src/Drape.Infrastructure/Configuration/ConfigJson.cs` (camelCase/null policy at lines 32–35, enum writers at 349/393, nullable z-order at 374, timestamps at 421/433/457/461, flat quick-hide at 486/533) and `tests/Drape.Infrastructure.Tests/FileConfigStoreTests.cs` (50 by 50 profile at 282–290). Legacy source was not edited or used as Rust test evidence.

Before persistence implementation, architect review must settle bounded reads and distinct missing/oversize/access errors; exclusive temporary ownership; separate write/sync failure points; atomic replace versus no-overwrite first create; bounded managed-backup listing and collision-safe quarantine; owned-temp cleanup; and typed load/commit/cleanup outcomes. Preserve an opaque lossless document alongside domain data. Specify worker/cancellation/shutdown semantics, including cancellation after a successful commit. These are interface prerequisites, not newly implemented ports. No numeric parser budget was invented; ten backups and 500 ms debounce remain the existing documented constants.

## Validation

QA reports the targeted Core domain suite passed all 16 tests, including `layout_snapshot_rejects_blank_names_and_duplicate_drape_ids`, `layout_snapshot_rejects_duplicate_item_ids_and_path_keys_across_drapes`, and `drape_snapshot_rejects_duplicate_membership_and_layout_values_are_preserved`.

The targeted Windows shell library suite passes all 10 tests: five cache tests, four DIB conversion tests and `literal_wide_paths_reject_empty_and_embedded_nul_before_native_access`. Cache tests are `cache_evicts_least_recently_used_pixels_within_byte_budget`, `cache_replacement_accounts_for_old_pixels_and_rejects_oversized_atomically`, `cache_keys_distinguish_scale_revision_and_literal_identity`, `cache_entry_limit_bounds_tiny_icons_and_zero_disables_caching`, and `cache_rejects_unbounded_identity_and_wrong_requested_size`. Conversion coverage includes `dib_conversion_skips_source_padding_and_preserves_channel_bytes`.

Both load-bearing mutation checks fail the intended test assertions (Cargo exit 101). Disabling cross-Drape PathKey rejection makes `layout_snapshot_rejects_duplicate_item_ids_and_path_keys_across_drapes` receive an accepted layout instead of `Duplicate("membership")`. Disabling promotion on a cache hit makes `cache_evicts_least_recently_used_pixels_within_byte_budget` retain the wrong entry. These are behavioral assertion failures, not compile/tool errors. QA restored both original source files byte-for-byte in `finally` before running final gates.

Final workspace, architecture and coverage results are pending the exclusive QA queue and will be recorded before handover.

Subsequent owner-authorized continuation preserves this checkpoint and records the later integrated verification, passive enumeration and persistence-interface review in the [enumeration/interface record](./2026-11-25-enumeration-persistence-contracts.md). Its results apply to that later source tree; they do not reconstruct missing earlier tests-first evidence.

## Not determined

No native API probe or GUI was launched. Named Windows 10/11 builds, DPI/monitor matrix, hydration behavior, overlays/alpha, 500-item enumeration, cold/warm performance, native resource retention and event-loop integration remain unverified. Full R0-06 and R1-01 prerequisite acceptance is not established. R1-02 remains spec-only until its persistence interfaces, lossless wire representation and input limits are reviewed.

The earlier [shell/snapshot checkpoint](./2026-11-25-shell-snapshot-parallel-continuation.md) records that its first locked test stopped before compilation; this is not a tests-first RED result. Later passing tests do not reconstruct that missing evidence.
