# Passive enumeration and persistence contract continuation

Date: 2026-11-25. Authority: owner requested continued parallel implementation. Branch: `feat/R0-06-enumeration-contracts`. Existing uncommitted snapshot/cache/spec changes from the previous continuation are preserved. No commit, merge or DONE status is implied.

## Ownership and scope

- `shell_enumeration`, acting as shell-engineer-windows, owns only Windows shell source. R0-06 adds a passive bounded metadata probe; no desktop visibility mutation or GUI launch.
- `persistence_architect`, acting as architect, owns Core persistence interfaces, their spec and narrowly related architecture boundary policy. R1-01 adds interface declarations only; R1-02 remains service/parser/adapter implementation pending.
- `qa_queue` is the sole validation execution owner and reviews behavior sequentially after source freeze.
- The orchestrator owns task tracking, this record, index and changelog. The whole-team WIP count was two during implementation; both subsets are now frozen in REVIEW for QA.

No codebase-memory graph tools are available in this session. Localized source reads and diffs were used as the fallback; no indexing or graph result is claimed.

## Implementation

The raw Windows probe enumerates immediate children of private/public Known Folder desktop roots through FindFirst/FindNext. Owned literal UTF-16 paths, item classification and raw attributes cross the probe boundary; search handles use FindClose RAII. Caller-supplied limits bound retained entries, scanned entries including filtered entries, and retained path units. Cancellation is checked around native calls, errors discard the partial result, and hidden/system visibility is explicitly supplied. Desktop metadata is not opened as file content. Reparse/offline/recall roots fail closed.

The probe is not yet `DesktopItemSource`: platform-correct PathKey comparison semantics, Explorer settings acquisition, worker lifecycle and UI integration remain prerequisites. A point-in-time attribute preflight does not prove provider-specific no-hydration behavior or protect against concurrent root replacement.

Persistence contracts separate bounded stable reads, exclusive root-bound writer/temp ownership, full write and sync stages, no-overwrite first create, revision-checked replacement, collision-safe quarantine, bounded backup listing and owned-temp cleanup. Opaque associated tokens keep arbitrary paths and native handles out of Core. Commit outcome and storage durability are distinct; uncertain outcomes retain evidence and prohibit blind retry/prune. Numeric parser limits, lossless JSON, ConfigStore service and native filesystem adapter remain unimplemented.

## Validation and review

The exclusive QA queue passed the targeted Windows shell library suite: 15/15 tests, including all five new metadata tests below. The locked optimized workspace Release build passed with `CARGO_INCREMENTAL=0` (1 minute 30 seconds). Workspace formatting and locked all-targets Clippy with `-D warnings` also pass. All 58 workspace Rust tests pass: 3 App unit, 2 App CLI, 1 Core baseline, 16 domain, 14 layout, 1 Guard, 15 shell, 1 test-support clock, 2 capability and 3 port tests. Final architecture/coverage/mutation results are still pending.

- `metadata_scan_counts_filtered_entries_and_never_silently_truncates`
- `metadata_visibility_filters_independently_and_always_ignores_desktop_ini`
- `metadata_preserves_literal_utf16_and_classifies_without_opening_children`
- `metadata_path_budget_rejects_before_retaining_a_partial_entry`
- `metadata_rejects_malformed_components_and_honors_precancel_before_native_work`

Different-role shell review of the persistence interface identified two native-contract traps: revision checks are not an OS compare-and-swap against arbitrary uncooperative external writers, and quarantine namespace changes can become indeterminate just like commits. The architect scoped exclusion to cooperating Drape writers and added `QuarantineOutcome` with moved, verified not-moved and indeterminate states. Uncertain quarantine blocks backup recovery and overwrite. Adapter implementations must use verified object ownership for cleanup rather than path-only check/delete. No native guarantee is inferred from a declaration. [ADR-030](../DECISIONS.md) records the interface refinement and its costs. The architecture checker also receives a narrow allowance for the already pinned Windows binding in Shell/Guard's transitive closure, while actual-metadata mutations continue to reject direct native dependencies in shared crates and Guard.

Follow-up shell source review accepts the corrected interface declarations with native implementation/acceptance still pending. Independent architect source review finds no blocking enumeration lifetime/layering defect; FindClose owns search handles and literal metadata remains separate from unproved comparison keys and capabilities. QA first reproduced the outdated boundary policy failure against actual metadata before its narrow policy correction; final rerun is pending.

Before targeted compilation, QA granted the shell owner an exclusive correction window to check aggregate UTF-16 root/separator/name cost before constructing a retained PathBuf. Source refroze before formatting and tests. Implementation agents execute no verification commands.

## Not determined

No native API enumeration or icon probe is launched during this continuation. Windows 10/11 build, DPI/monitor, OneDrive hydration, 500-item performance, icon alpha/overlays, handle retention and native replacement/durability acceptance remain unverified. Fake or pure tests cannot establish those contracts. Full R0-06/R1-01 acceptance and R1-02 implementation prerequisites remain open.
