# R0-01 — Rust + Slint Documentation Baseline

Date: 2026-11-25. Role: architect/documentation, orchestrator board handoff; implementation not authorised. Status: documentation prepared, independent review pending. Decision: ADR-025.

## Request and result

The owner chose Rust for RAM, performance, memory safety and size, selected Slint to avoid WebView/web engines, supplied a Gemini discussion, then requested all documentation updated before implementation. Active docs now describe the Rust target; legacy C# source/build/CI remain untouched. This is not a claim of a working Rust port.

## Changes and rationale

- New ARCHITECTURE/03 migration contract and RUST-MIGRATION task plan use fresh R0/R1 IDs; preserve old P0/P1 cards/specs/runbooks/baselines as historical evidence. P2–P6 keep future scope with Rust prerequisites.
- Update system architecture, stack, stable JSON compatibility, native ownership/threading, Guard-before-hide ordering, unsafe/RAII/FFI policy, coding/tests/CI/release plans, UI/accessibility/licensing, risk register, roadmap and agent instructions.
- Preserve MIT/Drape name, Windows 10 22H2 minimum, Windows-first scope, no default network, opt-in updates and non-destructive item references.
- Correct stale no-code/planning claims. Prior board reported 98 C# tests; not rerun, not Rust evidence. Old card/board contradictions are recorded rather than retroactively rewritten.
- Blur remains planned; Q-19 priority is unanswered. Q-20 numeric footprint/tighter RAM targets need measurements. Q-21 integration/backend/renderer and Q-22 Rust updater compatibility remain unresolved.

## Evidence and validation

Reference Gemini page and official Slint site/licence were read in the preceding discussion (2026-11-25). Gemini sample versions/code/WorkerW fallback/RAM/estimates are not accepted as verified facts. Slint royalty-free licence 2.0 requires attribution; pinned release terms must be rechecked later.

Documentation validation: local Markdown target checks, task-ID/dependency consistency, legacy evidence preservation, whitespace and Markdown-only git diff checks. Exact final results are entered after running the checks below; no application test result is asserted.

## Tests not run

No Rust build, format, Clippy, unit, coverage, UI, E2E, desktop-safety or performance test: no Rust workspace/implementation is created. Legacy .NET tests were not rerun because code/build configuration is unchanged. Guard/native-host/API claims remain future spike acceptance criteria.

## Handoff and risks

Architect -> orchestrator: target/task/status docs updated, source interfaces untouched. Verification is documentation link/consistency/whitespace/scope checks. Review by a different role is still required before R0-01 is DONE; it must not count as Rust implementation completion. Next code task R0-02 requires a separate implementation request. No commit, PR, source retirement or deployment was requested.

Native safety is not guaranteed by Rust: COM affinity, borrowed HWND lifetime, native crashes, abort/kill and durable recovery require new tests. Restore ordering was clarified so config/log failure cannot strand icons. One window per Drape remains; actual backend/host/rendering/accessibility is unproved. Non-Windows minimums require fresh validation.

## Files and continuity

Scope: README/agent Markdown, docs/, TASKS/, MEMORY/, historical runbook notices only. ADR-025, MEMORY-INDEX, CHANGELOG and PROGRESS updated with this record. No source/manifests/workflows/configuration changed.

## Final check results

DOC-LINKS: 72 Markdown files inspected, 466 local link targets checked, 0 missing targets (template examples excluded). TASK-IDS: 16 R0/R1 cards defined, no undefined Rust task references. `rtk proxy git diff --check` passed with exit 0. Scope verification found only Markdown changes apart from temporary checking helpers, which were removed before handover. R0-01 is REVIEW, independent review pending; no application tests claimed.
