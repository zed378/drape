# Parallel layout and host diagnostics continuation

Date: 2026-11-25. Owner: orchestrator/architect. Status: REVIEW; local implementation/validation complete, native/integration acceptance and merge pending.

The owner requested continued parallel implementation. Existing dirty changes and the current branch are preserved; no commit, merge or DONE claim is implied. ADR-029's single sequential QA owner remains in force. Three active cards are R0-03 (remaining validation), R1-03 (pure layout subset), and R0-04 (safe diagnostic subset). Root owns shared board/index/records; implementation file ownership is disjoint.

## Architect handoffs

- Core engineer owns new `crates/drape-core/src/layout.rs` and `crates/drape-core/tests/layout.rs`; root adds the module export. Additive pure layout uses the reviewed R1-01 domain subset without changing existing contracts. Stored original bounds/work-area snapshots remain immutable across display changes; monitor identity is adapter-supplied. Missing or ambiguous identity falls back to a unique primary; invalid primary topology returns a typed error. Snap tie precedence is work-area edges, supplied neighboring panels, then grid. Tests precede logic and QA executes the red phase.
- Windows shell engineer owns `MEMORY/specs/R0-04-slint-desktop-host.md` and `spikes/r0-04-slint-desktop-host/`. The isolated manifest retains Slint 1.13.1's baseline features, adds its public raw-window-handle-06 feature and exact raw-window-handle 0.6.2 (already present transitively). Root workspace/lock remain unchanged. The safe diagnostic only observes a borrowed Slint-owned native handle; it does not parent windows, hide icons or manipulate Explorer. The full A/B/C and Windows matrix remain pending.
- Sole QA owns `MEMORY/records/2026-11-25-rust-parallel-qa-continuation.md`, all sequential validation execution and explicitly handed-off formatting. Runtime/desktop claims require separate real evidence.

## Limitations

R1-01 remains partial REVIEW. Local layout development is authorized over its reviewed data subset, not evidence of completed prerequisites. R0-04 diagnostics do not satisfy desktop-host acceptance. No desktop icons or user files are modified. Detailed evidence and remaining acceptance are recorded in the task records and sole QA continuation record.

## Layout parity review notes

Graph discovery located the existing Rust Bounds and legacy LayoutService/MonitorResolver. The reviewed implementation follows current specifications rather than copying legacy case-insensitive device-name guesses or first-match ambiguity. Original work-area size is a caller-supplied runtime snapshot, not a new schema field; reconstructing it from persisted data and adapter observations remains application/config integration work. Keeping the title edge reachable avoids stranding keyboard/mouse entry above the work area. Snap is a pure positioning operation; callers clamp the final result before presentation.

## Validation checkpoint

The sole QA owner passed the locked workspace Release build, format check, denied-warning Clippy and all 38 Rust tests. Core coverage is 720/830 lines (86.75%), with every source file present and LCOV exported. An isolated intentional DPI-scaling mutation caused the named no-drift test to fail after the clean copy passed. Current cargo-deny advisory/licence/source checks passed; scoped current-source secret checks and prior Git-history checks are recorded with their exact limits. The isolated spike passed locked/offline Release, format, denied-warning Clippy, both CLI tests and compiled help/invalid-argument checks without GUI launch. R0-03 is local REVIEW, R1-03 pure-subset REVIEW and R0-04 diagnostic-subset REVIEW; persistence/native integration and merge remain pending. Final documentation/secret validation is recorded by QA.

Final current-tree Gitleaks passed on a curated copy of all 1,287 Git-tracked/untracked nonignored files (2.31 MB). Ignored build/tool caches were excluded; the redacted report is `coverage/gitleaks-current.json`. Temporary staging was cleaned. This complements the previously recorded committed-history scan.
