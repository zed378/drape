# Main integration — Rust migration baseline and reviewed subsets

Date: 2026-11-25. Owner: orchestrator. Authority: owner explicitly requested commit, merge to main and push.

## Scope

Integrate the accumulated Rust/Slint documentation migration, seven-crate baseline, CI, validated domain/read-only ports, pure layout and isolated handle diagnostic. Preserve legacy source and its historical evidence. Temporary review/secret-scan outputs and Python bytecode are ignored rather than published. No user files are deleted or desktop state changed.

The implementation was previously validated on frozen source: locked Release build, formatting, denied-warning Clippy and 38 workspace tests; 720/830 Core lines covered (86.75%); isolated no-drift mutation rejected by `resolution_and_dpi_cycles_always_use_original_snapshot_without_drift`. The isolated spike passed locked/offline Release, formatting, Clippy and its two named CLI tests. Current dependency audit and current-tree secret scan passed. Final documentation checks passed 14 documents/89 local links and relevant new-file whitespace. Detailed named evidence is in the [QA record](2026-11-25-rust-parallel-qa-continuation.md).

## Acceptance limits

Owner authorization permits integrating these concrete reviewed subsets. It does not establish native hosting/recovery, real HWND lifetimes, GUI/UIA, Windows/DPI/monitor matrix, persisted-layout reconstruction, performance or hosted Actions results. Partial tasks retain REVIEW and their remaining acceptance; no unfinished feature is marked DONE. Earlier records describing local/unmerged work remain dated checkpoints; this integration record documents the subsequent owner-authorized Git operation.

## Git operation

Commit the reviewed changes on the current feature branch, merge to main after fetching origin, then push main normally without force. Record exact resulting commit and remote verification in the completion report. A remote rejection must be resolved without rewriting unrelated history.
