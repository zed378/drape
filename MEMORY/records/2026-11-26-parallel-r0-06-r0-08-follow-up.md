# Parallel R0-06 / R0-08 follow-up

Date: 2026-11-26. Status: partial REVIEW after independent QA of the bounded implementation subsets. Authority: owner request to continue implementation in parallel. No commit, merge or DONE status is implied.

## Ownership and validation

- `shell-enumeration` (shell-engineer-windows) owns only `crates/drape-shell-windows/src/enumeration.rs` and its in-file tests for the R0-06 metadata scan-budget correction.
- `hook-consumer` (shell-engineer-windows) owns only `crates/drape-shell-windows/src/hook_state.rs` and its in-file tests for the R0-08 serialized pure-consumer continuation.
- `qa-queue` (qa) owns sequential test execution. It runs the R0-06 test-first RED check, then the R0-08 RED check, and later validates completed changes serially. Implementers run no verification commands.
- The orchestrator owns `TASKS/`, this record and its `MEMORY-INDEX.md` entry. No manifest, lockfile or shared module-root edits are in this ownership split.

The existing uncommitted source and documentation changes in the shared worktree are preserved as the baseline for this follow-up. Codebase graph tools are unavailable in this session, so source discovery uses the repository-local search fallback.

## Bounded scope

R0-06 corrects the passive metadata probe's bounded scan accounting by counting each native entry before filtering it from retained results. `DesktopItemSource` and Windows `PathKey` semantics remain open in [`BACKLOG.md`](../../TASKS/BACKLOG.md), including case-sensitive directories and extended/UNC paths; this continuation does not invent them. It also does not claim asynchronous worker/UI integration, OneDrive hydration safety on all providers, overlays, performance results or the Windows VM matrix. It does not change desktop visibility or user files.

R0-08 adds a serialized consumer around the existing bounded queue and recognizer, including reset and host/session generation revalidation. This slice stays pure and installs no native hook, performs no desktop hit test or icon mutation, and exposes no raw input outside transient bounded state. Native callback/API choices remain subject to independent source review before implementation.

Tests are authored first. QA reports each behavioral RED result before implementation proceeds, then runs the accepted implementation checks and workspace gates in one sequential queue. Native Windows, DPI, monitor, callback-latency and antivirus claims remain undetermined.

## Resumed owner-authorized continuation

The owner again requested parallel implementation on 2026-11-26. The same two WIP slices resume without taking a third implementation task or discarding the dirty baseline. `shell_enumeration` and `hook_consumer` retain the source ownership above; `qa` is the sole sequential validation owner. The orchestrator integrates task and continuity updates after their handoffs.

Initial inspection found that scan accounting already precedes ordinary filtering, but malformed filenames are validated before the counter and can bypass observed-entry accounting. The hook file already contains a partial serialized consumer; its existing behavior must be reviewed and extended where evidence finds a gap rather than represented as newly authored in this resumed run.

### Tests-first checkpoints

- QA ran `rtk proxy scripts/cargo-windows.cmd test -p drape-shell-windows --lib metadata_scan_budget_counts_malformed_entries_before_validation --locked`: the named test failed with observed count 0 instead of 1 before the correction. After moving scan accounting ahead of filename validation, `rtk proxy scripts/cargo-windows.cmd test -p drape-shell-windows --lib metadata_ --locked` passed all eight tests, including `metadata_scan_budget_is_shared_across_roots_and_excluded_entries`.
- QA confirmed a behavioral RED with `rtk proxy scripts/cargo-windows.cmd test -p drape-shell-windows --lib consumer_first_poll_discards_samples_queued_before_context_validation --locked` before implementing initial-context queue invalidation.

### Implementation and review handoffs

R0-06 now increments the checked observed-entry count before parsing the filename terminator. The same budget spans roots and includes dot, hidden, system and desktop.ini entries; retained-result limits stay separate. No native API, interface or user-file operation changed. QA's source review found no blocker for this bounded correction.

R0-08 now rejects pre-validation queued samples, resets private candidates immediately on `reset(queue)`, and requires the queue Reset before further samples. The serialized owner must supply idle-timeout, movement and disable/re-enable cancellation signals. QA reviewed initial contention, reset ownership and final context revalidation; no blocker was found under that explicit serialized-owner contract. The spec records these shell-local semantics; no Core contract or capability is enabled.

New hook tests:

- `consumer_first_poll_discards_samples_queued_before_context_validation`
- `consumer_revalidates_enabled_and_generation_immediately_before_dispatch`
- `consumer_owner_timeout_reset_clears_idle_history_and_pending_input`
- `consumer_initial_contention_defers_reset_without_processing_old_input`
- `consumer_idle_context_change_cancels_history_before_later_samples`

Both implementers froze their disjoint source files before the sole QA owner acquired temporary exclusive mutation-check ownership. Implementers ran no verification commands. No new ADR or architecture deviation is needed for these existing bounded-budget, cancellation and stale-input requirements. Native hook installation, desktop hit testing, callback latency, AV, hydration/performance and Windows/DPI/monitor matrix remain untested.

### Sequential QA checkpoint

The eight metadata tests and nine `consumer_` tests pass. QA verified three reversible behavioral mutations: bypassing malformed-entry scan accounting fails its named test; removing initial context reset fails `consumer_first_poll_discards_samples_queued_before_context_validation`; weakening final dispatch revalidation fails `consumer_revalidates_enabled_and_generation_immediately_before_dispatch`. Each mutation was reverted with exact source bytes restored before final gates. An initial formatting check found test-only formatting; QA applied package formatting before continuing.

Final workspace gates reported by the sole QA owner:

- `rtk proxy scripts/cargo-windows.cmd build --workspace --release --locked` passed.
- `rtk proxy scripts/cargo-windows.cmd fmt --all -- --check` passed after formatting.
- `rtk proxy scripts/cargo-windows.cmd clippy --workspace --all-targets --locked -- -D warnings` passed.
- `rtk proxy scripts/cargo-windows.cmd test --workspace --locked` passed: 99 tests, including 39 shell tests (eight metadata and 21 queue/recognizer/consumer tests).

QA used `CARGO_INCREMENTAL=0` for final gates to avoid sandbox incremental hardlink warnings. A direct boundary-test invocation initially lacked Cargo on PATH and ran no tests; QA corrected the repository-local Cargo/Rustup environment without source changes. Final architecture and documentation results are recorded below. Core coverage was not rerun because this continuation changes only shell logic/tests and documentation; the earlier 89.27% result remains dated historical evidence, not a fresh measurement. Dependency/secret scans are not claimed rerun for this source-only follow-up.

QA accepted the bounded implementation subsets after source review and behavioral validation. Cards and board remain partial REVIEW because native/integration acceptance is incomplete and this continuation is unmerged. No native desktop operation or user-file mutation was performed.

Final architecture/document checks passed: seven architecture tests, including actual-Cargo dependency mutation checks for Core/UI/Guard/Infrastructure/native boundaries (93.845 seconds). The successful command supplies the repository-local toolchain explicitly:

```powershell
rtk proxy python -c "import os,runpy; from pathlib import Path; root=Path.cwd(); os.environ['CARGO_HOME']=str(root/'.tools/cargo'); os.environ['RUSTUP_HOME']=str(root/'.tools/rustup'); os.environ['PATH']=str(root/'.tools/cargo/bin')+os.pathsep+os.environ['PATH']; runpy.run_path('scripts/check-rust-boundaries.py',run_name='__main__')"
```

After the orchestrator's documentation-ready handoff, QA verified 98 relative links and zero trailing whitespace across the six changed Markdown files and two untracked shell sources. `rtk proxy git diff --check` passed (pre-existing CRLF notices only). No source edits remained pending. These checks do not stage the untracked implementation or authorize commit/merge.

## Bounded metadata and callback adapters — 2026-11-26

The owner requested another parallel continuation. `enumeration.rs` includes the fail-closed metadata-to-`DesktopSnapshot` adapter requiring one supplied comparison key per retained entry; it does not invent Windows key semantics. `hook_callback.rs` maps only eligible button messages, forwards negative/unknown codes, applies bounded queue admission, and latches discontinuities. QA found that the disable/re-enable test expected an event before the consumer observed the enable Reset. The test was corrected to poll Reset first, then verify that a later event is accepted; queue behavior and the spec were unchanged.

Named verification:

- `rtk proxy scripts/cargo-windows.cmd test -p drape-shell-windows --locked enumeration::tests::` passed 11 tests: `comparison_keys_are_attached_without_path_normalization`, `comparison_key_cardinality_mismatch_fails_closed`, `item_source_does_not_claim_unverified_windows_keys`, `metadata_scan_budget_counts_dot_entries_before_filtering`, `metadata_scan_budget_counts_malformed_entries_before_validation`, `metadata_scan_budget_is_shared_across_roots_and_excluded_entries`, `metadata_scan_counts_filtered_entries_and_never_silently_truncates`, `metadata_visibility_filters_independently_and_always_ignores_desktop_ini`, `metadata_preserves_literal_utf16_and_classifies_without_opening_children`, `metadata_path_budget_rejects_before_retaining_a_partial_entry`, and `metadata_rejects_malformed_components_and_honors_precancel_before_native_work`.
- `rtk proxy scripts/cargo-windows.cmd test -p drape-shell-windows --locked hook_callback::tests::` passed 5 tests: `negative_code_forwards_without_admitting_input`, `eligible_button_is_copied_and_forwarded`, `unknown_message_is_forwarded_without_queue_entry`, `disable_latches_loss_and_drops_until_enabled`, and `forwarding_panic_is_contained`.
- Before the format-only pass, `rtk proxy scripts/cargo-windows.cmd build --workspace --release --locked`, `clippy --workspace --all-targets --locked -- -D warnings`, and `test --workspace --locked` passed; workspace tests: 107. The source-boundary script passed 7 tests using the repository-local Cargo/Rustup environment.
- After workspace formatting, `rtk proxy scripts/cargo-windows.cmd fmt --all -- --check` and both focused suites passed again.

Not tested: real `WH_MOUSE_LL` installation, hit testing, callback latency, antivirus response, native desktop enumeration/icon extraction, OneDrive hydration behavior, and Windows 10/11 VM/DPI/monitor/Explorer-restart cases. No desktop visibility or user files were changed. Both cards remain partial REVIEW; no commit, merge or DONE status is implied.
