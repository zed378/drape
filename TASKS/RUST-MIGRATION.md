# Rust Migration — Replacement Foundation and Core

Spec refs for every card: [migration specification](../docs/ARCHITECTURE/03-RUST-MIGRATION.md), [ADR-025](../MEMORY/DECISIONS.md), plus the per-card references below. Status authority: [PROGRESS](./PROGRESS.md). R0/R1 IDs are new; P0/P1 are legacy history, not completed Rust work. Estimates await spike results. The owner authorised implementation continuation on 2026-11-25. R0-02 is the active implementation task; legacy source remains preserved.

## R0-01 — Update documentation for Rust + Slint

| Field | Value |
|---|---|
| Status | REVIEW — specialist documentation corrections complete; architect owner, merge pending, 2026-11-25 |
| Priority / role | P0 / architect (documentation), orchestrator handoff for board |
| Depends on | Owner's Rust + Slint decision |
| Spec refs | Architecture/00–03, PLAN/01–05, ENGINEERING/00, TESTING/00, DEVOPS/00 |
| Spec required | Yes — docs/ARCHITECTURE/03-RUST-MIGRATION.md |

Acceptance: active docs and agent instructions agree on Rust + Slint; legacy evidence is labelled; fresh migration IDs and open decisions exist; Markdown links and whitespace are checked; only Markdown changes; record, index and progress agree. Independent role review is required before DONE. Abuse case: renaming old tests/toolkits in historical reports to suggest a Rust port passed.

Review assignment: architecture/engineering, Windows/security and QA/release/UI specialists review disjoint document sets; root integrates task/agent/continuity fixes. Documentation review acceptance does not constitute implementation validation or a merge. No PR/commit is implied by the owner's review request.

## R0-02 — Rust workspace, contracts and dependency/licence baseline

| Field | Value |
|---|---|
| Status | REVIEW — architect, 2026-11-25; local checks passed on feat/R0-02-rust-workspace; independent review/merge pending |
| Priority / role | P0 / architect |
| Depends on | R0-01 |
| Spec refs | ARCHITECTURE/00–03; ENGINEERING/00; PLAN/05 |
| Spec required | Yes ? [workspace spec](../MEMORY/specs/R0-02-rust-workspace.md) |

Acceptance: documented workspace exists; exact stable toolchain and Cargo.lock pinned after verifying supported versions; Windows minimum and Slint licence/attribution recorded; no UI/OS/I/O dependencies in Core; actual cargo metadata checked by a negative boundary test; restore CLI binary target defined; no legacy source removed. Select a non-WebView, non-Qt deployment for the baseline. Abuse case: a hand-written allow-list that never inspects real Cargo dependency edges.

Evidence: [R0-02 record](../MEMORY/records/2026-11-25-R0-02-rust-workspace.md): locked release build, rustfmt, Clippy and 8 Rust tests pass; 6 architecture tests pass, including forbidden-edge mutation checks. ADR-028 isolates compiler-generated Slint unsafe glue. Native recovery/UI/runtime matrix remain untested. No commit/merge or independent review is implied.

## R0-03 — Rust CI and quality gates

| Field | Value |
|---|---|
| Status | REVIEW — release-engineer, 2026-11-25; local QA and architect review passed, hosted Actions/merge pending |
| Priority / role | P1 / release-engineer |
| Depends on | R0-02 |
| Spec refs | DEVOPS/00; TESTING/00; ENGINEERING/00 |
| Spec required | No |

Acceptance: pinned Windows CI runs locked release build, rustfmt, Clippy with warnings denied, tests and Core coverage >= 80%, licence/advisory/secret checks; deliberate warning and format failures are demonstrated; workflow distinguishes Rust from any retained legacy build. No Explorer manipulation in automation.

## R0-04 — Slint desktop-host and native-handle spike

| Field | Value |
|---|---|
| Status | REVIEW (partial) — shell-engineer-windows (host_spike), 2026-11-25; locked Release/format/Clippy and 2 CLI tests pass; full host strategies and real Windows matrix pending |
| Priority / role | P0 / shell-engineer-windows |
| Depends on | R0-02 |
| Spec refs | PLATFORM/00 sections 1–2; ARCHITECTURE/00,03; ADR-007 |
| Spec required | Yes |

Acceptance: strategies A/B/C tested with actual Slint windows and the public API of the pinned backend; borrowed HWND lifetime and destruction proven; Win+D, peek, activation, explicit keyboard focus, Alt+Tab/taskbar absence, manual Explorer restart, transparency and mixed-DPI two-monitor matrix recorded for Windows 10 22H2 and Windows 11 builds. A failed discovery never assumes Progman is safe. Abuse case: substituting a plain native window for the Slint test without identifying it.

## R0-05 — Native-icon state and independent recovery spike

| Field | Value |
|---|---|
| Status | WIP — orchestrator, 2026-11-26; recovery protocol spec written before implementation; native decision evidence remains required |
| Priority / role | P0 / shell-engineer-windows |
| Depends on | R0-02 |
| Spec refs | PLATFORM/00 section 3; ARCHITECTURE/00,03; SECURITY/00 |
| Spec required | Yes |

Acceptance: read-before-change and verified restoration, pre-hidden icons respected, durable intent before mutation, Guard readiness handshake, failed Guard launch leaves icons visible, normal/panic/abort/forced-kill/next-start/standalone restore paths documented and tested safely in a VM. Never automate killing Explorer. Abuse case: asserting Drop or a panic hook handles forced termination.

Required protocol cases: serialize App/Guard/CLI ownership per user/session with generation and OS process identity; do not recover a live owner's state during duplicate startup. Authenticate session-local IPC and reject stale/replayed acknowledgements. Test every crash boundary around intent/handshake/hide/verify/disarm, Guard death while App remains alive (restore owned changes and prevent rehide until rearmed), corrupted/truncated/unknown recovery state without blind toggles, simultaneous restore/exit/startup and explicit restore coordination with a live App. Revalidate liveness during the hide transaction; readiness is not a permanent guarantee. Recovery is idempotent and respects user changes. Record residual failure windows and recovery latency rather than claiming perfect atomicity across processes and Explorer.

## R0-06 — Shell enumeration, icons and cache spike

| Field | Value |
|---|---|
| Status | REVIEW (partial) — shell-enumeration, 2026-11-26; bounded STA icon worker returns raw DIB with request identity; 23 related shell tests, 50 shell tests, 110 workspace tests, locked build/format/Clippy and 7 boundary tests pass; pixel semantics/provider/UI and VM matrix pending |
| Priority / role | P1 / shell-engineer-windows |
| Depends on | R0-02 |
| Spec refs | PLATFORM/00 section 4; ARCHITECTURE/02; TESTING/00 |
| Spec required | No |

Acceptance: Known Folder Desktop + Public Desktop; 500 mixed items enumerated within 1 s with asynchronous icons; no OneDrive hydration; owned pixel buffers, COM apartment requirements, overlay quality, cold/warm timings and cache byte limit recorded; UI image conversion performed on the Slint event-loop thread. Release every native allocation with its matching API.

## R0-07 — Slint renderer, transparency, accessibility and drop feasibility

| Field | Value |
|---|---|
| Status | TODO |
| Priority / role | P0 / ui-engineer, handoff to shell-engineer-windows for native adapters |
| Depends on | R0-02, R0-04 |
| Spec refs | ARCHITECTURE/01,03; PLATFORM/00 sections 6,9; UI-UX/00; TESTING/00 |
| Spec required | Yes |

Acceptance: compare eligible native renderers with 1/5/20 windows; record CPU/GPU/working set/private bytes and distribution size; prove transparency with selected host; assess blur without deciding Q-19; high contrast, keyboard focus and Windows UIA/Narrator; inbound/outbound native OLE drops without changing Slint's borrowed HWND ownership. Record unsupported public APIs and explicit adapters instead of inventing equivalents to Avalonia APIs.

Keyboard acceptance includes entering tray/settings and choosing/focusing any Drape without a mouse or the optional global hotkey; Tab is not assumed to traverse independent native windows. Cover DPI through 300%, one to four monitors, mixed scale/negative origins and text scaling for the promised range. Renderer readback and coordinate conversion are verified under native parenting rather than assumed from ordinary top-level windows.

## R0-08 — Minimal global-hook spike

| Field | Value |
|---|---|
| Status | REVIEW (partial) — hook-consumer, 2026-11-26; callback data is deferred until non-negative hook codes; 27 hook tests, 50 shell tests, 110 workspace tests, locked build/format/Clippy and 7 boundary tests pass; native WH_MOUSE_LL/API and VM matrix pending |
| Priority / role | P1 / shell-engineer-windows |
| Depends on | R0-02 |
| Spec refs | PLATFORM/00 section 7; SECURITY/00 |
| Spec required | Yes |

Acceptance: dedicated message loop, preallocated bounded non-blocking event queue, no panic unwinds across FFI, no stored input, off-callback hit tests, context menu unaffected, full callback latency average < 1 ms, hook shutdown and queue-overflow behaviour tested. Build/DPI/monitor and antivirus observations recorded. Derive double-click from bounded transient button/time/position state outside WH_MOUSE_LL; it does not deliver a dedicated double-click event. Never persist/log input; clear transient sequences after overflow/session loss.

## R0-09 — Lifecycle, tray and resource baseline

| Field | Value |
|---|---|
| Status | TODO |
| Priority / role | P1 / ui-engineer (app), shell handoff for native lifecycle |
| Depends on | R0-02, R0-05, R0-07 |
| Spec refs | ARCHITECTURE/00,01,03; DEVOPS/00; TESTING/00 |
| Spec required | Yes |

Acceptance: explicit Rust composition, single-instance signalling, tray/event-loop integration, redacted rolling logs, independent restore mode that bypasses renderer startup, Guard lifecycle and panic/abort recovery distinction; idle no-redraw evidence; App + Guard memory, cold/warm startup and installed/download sizes measured. Propose Q-20 budgets from results, not from Gemini claims.

Single-instance/recovery coordination must precede icon mutation. A standalone restore path works without a renderer while coordinating a live App to suspend rehide; unknown ownership never becomes a blind icon toggle. Lifecycle includes loss of Guard health, denied IPC/locks and cancellation of stale UI/work completions after shutdown. Failed persistence/logging must not block restoration.

## R0-10 — Accept Windows host/backend/recovery decisions

| Field | Value |
|---|---|
| Status | BLOCKED — Rust spike evidence absent |
| Priority / role | P0 / architect |
| Depends on | R0-04, R0-05, R0-06, R0-07, R0-08, R0-09 |
| Spec refs | ARCHITECTURE/00,03; PLATFORM/00; ADR-006,007,025 |
| Spec required | No |

Acceptance: new ADRs cite matrix results, failures and fallback limitations; Q-21 settled; no basic bottom-most window described as Win+D-equivalent without proof; safety gates cannot be waived; update roadmap estimates. Record anything not determined.

## R1 — Rust core and first usable Drape

Each card requires a parity matrix of relevant legacy behaviours and new named Rust tests. Core tests are written before porting logic. No old DONE status counts as Rust completion.

| ID / title | Status / role | Depends on | Spec refs | Acceptance / abuse case |
|---|---|---|---|---|
| R1-01 Domain models and platform-free ports | REVIEW (partial) / core-engineer, 2026-11-26; ConfigStore contract and fake passed architect review, 126 workspace tests ([record](../MEMORY/records/2026-11-26-R1-01-config-store-contract.md)); native port contracts pending spikes | R0-02 | ARCHITECTURE/00,02,03; ENGINEERING/00 | Owned domain structs/enums, validation, capabilities, deterministic fakes and immutable Snapshot values; no platform/UI handles or I/O; tests for invalid IDs, dimensions, duplicate membership and snapshot validity. Rule condition/action semantics remain P4-02 scope and must not be invented here. |
| R1-02 Durable config and legacy schema compatibility | REVIEW (partial) / core-engineer, 2026-11-26; service verified against the filesystem fake ([record](../MEMORY/records/2026-11-26-R1-02-persistence-service.md), [ADR-031](../MEMORY/DECISIONS.md)); native adapter, path keys and legacy-file run pending | R1-01 | ARCHITECTURE/02; TESTING/00; [spec](../MEMORY/specs/R1-02-durable-config.md) | Atomic writes and failure injection, versioned hand-written fixtures, unknown fields/legacy values preserved, quarantine/backups/newer-version read-only. Do not generate fixtures with the migrator itself. |
| R1-03 Layout, snap and monitor recovery | REVIEW (pure subset) / core-engineer (layout), 2026-11-25; 14 new tests, 86.75% Core coverage and mutation pass; integration pending | R1-01 | PLAN/02; ARCHITECTURE/02; TESTING/00 | Stored-position math, DPI cycles without drift, monitor removal/return with ambiguous-ID/reconnect fallback, deterministic negative tests and >=80% Core coverage. Do not hide drift with rounding in tests. |
| R1-04 Slint panel, item grid, keyboard and native drops | TODO / ui-engineer, shell adapter handoff | R0-06, R0-10, R1-01, R1-03 | UI-UX/00; PLATFORM/00; PLAN/02 | Create/move/resize/rename/delete, one window per Drape, bounded asynchronous icons, item open/remove/reorder/drop, UIA/keyboard including tray/settings/Drape entry without optional hotkey, correct explicit focus and DPI through 300%/four monitors; no filesystem mutation for membership. A rendering screenshot alone is not acceptance. |
| R1-05 Config and application-service integration | TODO / ui-engineer, core service handoff | R0-09, R1-02, R1-04 | ARCHITECTURE/00,02; PLAN/01,02 | 500 ms save debounce, flush on normal shutdown without blocking recovery, load/reconcile, commands/tray, single instance, safe default/degraded behaviour; named integration tests. A fake host does not prove desktop attachment. |
| R1-06 Rust parity review and Phase 1 baseline | BLOCKED / qa | R0-03, R1-01, R1-02, R1-03, R1-04, R1-05 | TESTING/00; PLAN/01,03; ARCHITECTURE/03 | Run build/format/Clippy/tests, record coverage and performance, verify 3-Drape persistence workflow and manual environment matrix, cross-role review; no unsupported desktop claims. |

## Handoff architect → orchestrator · 2026-11-25

- Context: the owner chose Rust + Slint and requested documentation only.
- Done: target design, task IDs, historical-evidence policy and future gates documented.
- Not done: implementation, code retirement, Rust test execution, independent review.
- Contract/interface changed: target Rust ports only, ARCHITECTURE/00–03; existing source contracts untouched.
- How to verify: Markdown link/whitespace checks and diff scope; Rust commands are future gates, not executable migration evidence today.
- Risks / surprises: Windows host/Slint interoperability unproved; legacy task states were inconsistent; blur priority, numeric size budget and packaging remain open.

### R1-01 partial implementation checkpoint — 2026-11-25

The [domain record](../MEMORY/records/2026-11-25-R1-01-domain-models.md) lists approved validated data, three read-only ports, deterministic fakes and the legacy parity matrix. QA alone ran tests-first red phases and 18 targeted passing Core/test-support tests. Remaining FileSystem/ConfigStore/native lifecycle and other contracts/fakes are not implemented; integrated gates and full parity are pending. This partial REVIEW does not satisfy dependent-task acceptance or imply merge/DONE.

### Handoff architect → core-engineer / shell-engineer-windows · 2026-11-25

- Context: owner requested continued parallel implementation after the cache/snapshot checkpoint.
- Done: R1-01 now declares root-confined FileSystem contracts; shell source review corrected external-writer exclusion and quarantine uncertainty, with ADR-030/spec updates.
- Not done: deterministic filesystem fake, ConfigStore/parser, alias/lossless-wire/limit evidence, native filesystem adapter and full R1-01 acceptance.
- Contract/interface changed: FileSystem and typed commit/quarantine outcomes in crates/drape-core/src/ports.rs; no native handles or direct I/O in Core.
- How to verify: sole QA queue runs locked workspace gates and actual Cargo boundary mutations; named evidence in the [continuation record](../MEMORY/records/2026-11-25-enumeration-persistence-contracts.md).
- Risks / surprises: revision checks are not compare-and-swap against arbitrary external editors; indeterminate mutations retain evidence and stop saves/recovery until reconciliation.

### Handoff shell-engineer-windows → qa / ui-engineer · 2026-11-25

- Context: R0-06 passive metadata probe extends the prior icon/cache subset.
- Done: bounded Known Folder desktop metadata enumeration, lossless literal UTF-16 paths, visibility inputs, cancellation and FindClose RAII; architect static review passed.
- Not done: Windows comparison keys, complete DesktopItemSource, Explorer visibility acquisition, worker/UI integration and native hydration/performance matrix.
- Contract/interface changed: shell-local raw metadata probe in crates/drape-shell-windows/src/enumeration.rs; no Core native/source interface change.
- How to verify: metadata_* shell library tests; final QA evidence in the [continuation record](../MEMORY/records/2026-11-25-enumeration-persistence-contracts.md).
- Risks / surprises: root preflight has native race windows; metadata-only calls do not establish no-hydration behavior on every provider.

### Handoff core-engineer → architect / qa · 2026-11-26

- Context: R1-01's root-confined FileSystem declarations needed a deterministic consumer fake before persistence implementation could proceed.
- Done: staged in-memory fake, opaque root-bound tokens, failure injection for precommit and indeterminate outcomes, and named failure-preservation tests; architect review and sequential workspace QA passed.
- Not done: native filesystem adapter, ConfigStore/parser, lossless wire/alias handling and numeric parser limits; these remain implementation prerequisites for R1-02.
- Contract/interface changed: none in this handoff; fake implements `drape_core::ports::FileSystem` and does not use real I/O.
- How to verify: `scripts/cargo-windows.cmd test --workspace --locked` → 88 tests; Core coverage 89.27%; see the [continuation record](../MEMORY/records/2026-11-25-autonomous-continuation.md).
- Risks / surprises: revision checks do not prove native compare-and-swap against arbitrary external writers; indeterminate outcomes stop automatic mutation and preserve evidence.

### Handoff shell-engineer-windows → architect / qa · 2026-11-26

- Context: R0-08's reviewed scope was limited to pure bounded queue and transient double-click state, with no live hook installation.
- Done: 12 named queue/recognizer tests, overflow-reset and cross-generation mutation checks; architect reviewed the clarified in-flight discontinuity ordering and dispatch guard.
- Not done: native callback/message loop, hit testing, drag handling, latency/antivirus observations or Windows VM matrix.
- Contract/interface changed: no Rust type/API change; the spec requires one serialized consumer to process Reset before later samples and revalidate enabled state plus host/session generation before dispatch.
- How to verify: `scripts/cargo-windows.cmd test --workspace --locked` → 88 tests; named mutations and full gate evidence are in the [continuation record](../MEMORY/records/2026-11-25-autonomous-continuation.md).
- Risks / surprises: a poll already past its loss check may yield one event ordered before the concurrent discontinuity; the future consumer must apply the specified stale-intent guard.

### Handoff shell-engineer-windows → qa / orchestrator · 2026-11-26

- Context: owner requested parallel continuation of R0-06 bounded metadata and R0-08 pure consumer slices.
- Done: scan accounting before malformed-name validation, initial consumer queue reset, explicit immediate cancellation reset, context/generation dispatch guards; independent QA accepted the subsets, 99 workspace tests and three behavioral mutations pass.
- Not done: DesktopItemSource/PathKey, worker/UI/native enumeration matrix, native callback/hit-test/lifecycle and hook latency/AV/Windows matrix.
- Contract/interface changed: shell-local HookConsumer only; no Core interface or capability change. See the clarified R0-08 spec.
- How to verify: locked workspace Release/format/Clippy/tests; named tests and exact evidence in the [follow-up record](../MEMORY/records/2026-11-26-parallel-r0-06-r0-08-follow-up.md).
- Risks / surprises: serialized owner must deliver timeout/movement/disable commands even when no button event arrives; pending queue loss permits one already-claimed sample under the documented ordering. No native acceptance or merge/DONE is implied.

### Handoff shell-engineer-windows → qa / orchestrator · 2026-11-26

- Context: owner requested the next bounded R0-06/R0-08 continuation after the reviewed pure subsets.
- Done: R0-06 adds a fail-closed metadata-to-`DesktopSnapshot` adapter requiring caller-supplied comparison-key cardinality; R0-08 adds a callback policy adapter for negative hook codes, eligible button mapping, bounded admission and discontinuity resets.
- Not done: Windows key provider/worker/UI integration, actual `WH_MOUSE_LL` installation/message loop, hit testing, latency/AV observations and VM matrix.
- Contract/interface changed: shell-local `attach_comparison_keys`/`WindowsDesktopItemSource` and `hook_callback` module; no Core interface change.
- How to verify: sequential QA should run focused enumeration and callback tests, then locked workspace gates and source-boundary checks.
- Risks / surprises: both adapters deliberately fail closed and do not establish native safety or icon recovery acceptance; new files were untracked at handoff.

### Handoff shell-engineer-windows → qa / orchestrator · 2026-11-26

- Context: the owner requested parallel continuation; QA found one callback test needed to consume the enable Reset before expecting a later event.
- Done: R0-06 metadata-to-snapshot adapter and R0-08 callback policy adapter passed focused checks; callback test ordering now matches the queue reset contract; rustfmt was applied to the two shell modules.
- Not done: Windows key provider/worker/UI integration, actual `WH_MOUSE_LL` installation/message loop, hit testing, icon extraction/hydration proof, latency/AV observations or VM matrix.
- Contract/interface changed: none in Core; shell-local fail-closed adapters only.
- How to verify: Release build, format check, Clippy, workspace tests and all seven boundary tests passed. Final format check and focused suites passed after formatting: `enumeration::tests` (11) and `hook_callback::tests` (5).
- Risks / surprises: workspace tests/build/Clippy/boundaries ran before the whitespace-only rustfmt pass; focused suites and format check were rerun afterward. Native/VM acceptance remains open; no commit/merge/DONE is implied.
