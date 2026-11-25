# R0-01 — Specialist Documentation Audit and Corrections

Date: 2026-11-25. Owner: architect for the migration baseline; root acts as orchestrator/docs integration. Request: owner explicitly asked several specialised agents to review all documents, repair gaps, add needed content and remove irrelevant material. Scope remains documentation only. Related: ADR-025 and ADR-026.

## Review team and ownership

| Reviewer | Assigned documents | Independent verdict |
|---|---|---|
| architecture_review — architect | ARCHITECTURE/*, ENGINEERING/00 | Planning contract acceptable after root alignment; no runtime claims |
| windows_safety_review — Windows shell/security | PLATFORM/*, SECURITY/* | Recovery/native safety planning acceptable after exact wording follow-through |
| qa_release_review — QA with release/UI review | TESTING/*, DEVOPS/*, UI-UX/*, PLAN/* | Ready for merge after identified task/template cross-references and checks |
| root — orchestrator/docs keeper | TASKS, MEMORY, agent/root guidance, historical notices | Integrates findings and validates whole-document set |

Three specialists ran with disjoint edit scopes and returned final read-only cross-document findings. This is one R0-01 task, not competing task owners or parallel implementation. No reviewer edits implementation, manifests or workflows. Root applied the final requested alignments before handover.

## Findings fixed

1. High/safety: startup recovery preceded live-instance ownership validation; readiness did not cover later Guard death. Specify serialized per-session generation, process identity, authenticated IPC, transaction crash boundaries, health monitoring, no rehide and unknown-state conservative handling. Quick-hide shares that protocol.
2. High/native: correct WH_MOUSE_LL double-click inference, bounded transient gesture data, callback no-unwind, borrowed HWND lifetime, COM apartment and native allocation release. A thread/timeout is not crash isolation or safe native cancellation.
3. High/data: clarify Shell-owned native FileSystem adapter injected into Infrastructure; single-writer atomic persistence, literal path/roots/case semantics, unknown values and bounded validation. Last committed config survives supported write failures; debounce/flush limits are explicit.
4. High/UI: keyboard-only tray/Settings chooser and move/resize modes do not depend on optional hotkeys or Tab across native windows. Native destructive shell verbs require explicit confirmation and reviewed invocation; OS prompts are not assumed.
5. Medium/evidence: test the promised 300% DPI/four-monitor range; remove green capability/support claims without Rust evidence. Describe App + Guard CPU/memory/startup/size measurement accurately, with shared-page and cache-state caveats.
6. Medium/release: remove .NET event/trimming/reflection assumptions, evaluate Rust updater under Q-22, sign formal public beta/stable, stage compatible App/Guard updates, test trust rejection/failure/rollback/minimum Windows/portable mode and restoration before deletion.
7. Medium/specification: Q-23 names missing pin/page/tab/portal identity/schema contracts. Preserve feature scope; require per-feature specs and incompatible-change migration before implementation. Regex rules use bounded engine/pattern/input/evaluation, not an assumed .NET timeout.
8. Medium/continuity: preserve historical records/results, label the old runs guide, fix task status/spec-required/Clippy/template contradictions and provisional estimates. Future platform hide paths stay disabled until their Guard gate passes.
9. Relevance: remove unrelated upstream-tool dashboard/savings/footer mandates and duplicated unavailable-tool/skill/compaction instructions. Centralise actual RTK/discovery/checkpoint guidance; Antigravity pointer updated with approved access to the protected .agents document.

## What was retained or not determined

Rust + Slint/no WebView, Drape MIT/name, Windows 10 22H2 minimum, Windows-first scope, no default network, opt-in updates and non-destructive user-file references remain. No implementation/API/version/renderer/desktop-host claim is newly accepted. Q-19 blur priority, Q-20 tighter RAM/size budgets, Q-21 native integration/backend, Q-22 packaging and Q-23 representation remain unresolved until owner/spec/spike evidence as applicable. Unsupported fixed consumer-ESU dates were removed from active risk prose, while historical ADR/record bodies remain intact.

No Rust/.NET unit, UI, desktop-safety, E2E or benchmark was run: no implementation changed and no Rust workspace exists. Documentation review is not runtime safety/performance/release acceptance. Source retirement, schema implementation, signing credentials, commit, PR and merge were not requested.

## Validation

Each specialist ran scoped git diff --check with exit 0. Root runs whole-set local link/anchor, task dependency, table/fence, historical evidence and Markdown-only scope checks plus global git diff --check. Final counts/results are appended after execution; no application test is asserted.

## Handoff

Architecture/shell/QA -> orchestrator: specialist patches and final residual wording/task requests are integrated. R0-01 becomes REVIEW with independent review completed, pending actual merge per task conventions. Follow-on Rust work requires a subsequent implementation task; no open safety finding is waived. MEMORY index/changelog/ADR and progress are updated with this record.

## Final check results

Pending whole-set validation.
