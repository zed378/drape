# Shell and snapshot parallel continuation

Date: 2026-11-25. Authority: owner requested parallel implementation. Status: WIP; no commit, merge or DONE implied.

## Ownership and scope

- `r0_06_shell` (shell-engineer-windows) owns `crates/drape-shell-windows/src/` for a partial R0-06 read-only Known Folder and memory-only icon probe.
- `r1_01_snapshot` (core-engineer) owns `crates/drape-core/src/domain.rs` and `tests/domain.rs` for immutable Snapshot values.
- The orchestrator owns Cargo manifests, task tracking and continuity records. The sole QA agent owns sequential validation and the authorized Cargo.lock update. Implementation agents run no verification commands.

## Architect handoff

Independent architect review found the existing R1-01 enumerated subset implemented but not sufficient to claim full dependent acceptance. Published snapshot fields support passive `LayoutSnapshot` and `DrapeSnapshot` values containing caller-supplied IDs/time, original Bounds/Style, and owned DrapeItem references. Constructors validate names and duplicate identities/membership. Save/restore/preview workflows remain P4-05.

Rule condition/operator/action encoding is not settled by the data-model envelope. P4-02 explicitly requires a feature specification and owns RuleCondition/RuleAction/RuleEngine behavior. Do not invent a typed rules schema here; R1-02 must retain existing rules losslessly in its wire representation. Remaining persistence/native ports still require their relevant specs and implementer review. R1-02 remains spec-only pending its prerequisites.

## R0-06 boundary

The architect selected the already locked `windows` 0.61.3 with target-specific Foundation, GDI, FileSystem, COM and Shell features, verified by the shell engineer against cached generated API sources. Known Folder allocations, COM apartments, GDI bitmap/DC owners use matching RAII cleanup. The probe rejects offline/recall/reparse attributes, requests memory-only icons, and returns owned raw BGRX DIB data with explicitly unverified alpha. It does not enable production capabilities or modify desktop state. Pure DIB row conversion is separate and requires callers to establish channel/alpha semantics.

## Validation checkpoint

Snapshot tests were authored before the new types. Initial targeted locked test stopped because Cargo.lock needed updating; it did not reach compilation and is not a tests-first RED result. QA is authorized to regenerate the lock offline, inspect dependency deltas and retry. Source and native runtime evidence remain pending.

## Not determined

No GUI/native APIs were launched for validation. Windows builds/DPI/monitor matrix, provider hydration behavior, overlays/alpha, 500-item enumeration, workers/cache, cold/warm timing, native resource retention and full R0-06 acceptance remain unverified. No actual directory enumeration or production icon provider is claimed by this partial probe. No recovery journal, icon visibility, Explorer, registry or referenced user files are changed.
