# 03 — Rust and Slint Migration

Status: documentation baseline, 2026-11-25. Owner decision: [ADR-025](../../MEMORY/DECISIONS.md). No implementation is authorised by this documentation change.

## Decision and boundaries

The target is Rust with Slint for native desktop UI, with no WebView, browser engine, JavaScript UI runtime or .NET runtime in the shipped Rust application. The owner's priorities are RAM usage, performance, memory safety and distribution size. These are measurement goals, not achieved results. Slint renders its own UI; native compilation does not imply that every control is an operating-system widget.

Windows 10 22H2 x64 remains the supported minimum; Windows 11 remains primary. Other platforms remain post-v1, capability-gated and unverified. Product scope, desktop safety, non-destructive membership, MIT licensing of Drape, offline defaults and opt-in updates are unchanged.

Blur remains a planned capability with a solid/translucent fallback. Whether blur must be enabled in the MVP is still an owner question (Q-19); the earlier suggestion to make it optional was not accepted as a scope change.

## Current code versus target design

The repository contains an early C#/.NET/Avalonia implementation and spike harnesses. The previous board reported 98 tests (71 Core, 14 Infrastructure, 7 architecture, 6 UI); this number is historical and has not been rerun for this change. There is no completed Rust migration or Rust test baseline.

This update changes Markdown only. It does not create Cargo manifests, change CI, move source directories, delete the old solution, port code or run desktop experiments. The legacy .NET build configuration continues to describe the current code, not the new target.

Old P0/P1 cards, feature specs, runbooks and measurement reports retain their original implementation details and dates. They are marked historical. A green C# test or a Win32-only experiment does not prove Slint window behaviour. Rust replacement tasks use fresh R0/R1 IDs in [the migration plan](../../TASKS/RUST-MIGRATION.md); old IDs are never reused. Unstarted P2–P6 work keeps its IDs and uses the new prerequisites.

## Workspace baseline (R0-02)

Implementation continuation is authorised by ADR-027. The seven-crate scaffold now exists; verification and limitations are in the [R0-02 record](../../MEMORY/records/2026-11-25-R0-02-rust-workspace.md). The documentation-only statements above describe R0-01, not the subsequent workspace implementation. Native features and parity remain unimplemented. ADR-028 confines Slint-generated unsafe glue to a private compiler-output module; hand-written UI/Core forbid unsafe.

```text
Cargo.toml                 workspace and dependency policy
Cargo.lock                 committed application dependency lock
rust-toolchain.toml        exact stable toolchain, selected during R0-02
crates/
  drape-core/              domain, layout, rules, ports, migrations
  drape-infrastructure/    config persistence, watchers, caches
  drape-shell-windows/     Win32/COM, icon recovery, native adapters
  drape-ui/                Slint components and Rust presentation adapters
  drape-app/               composition root, CLI, lifecycle, tray
  drape-guard/             independent icon-recovery process
  drape-test-support/      deterministic fakes and fixtures
tests/                    system scenarios and shared fixtures
spikes/                   isolated future Rust experiments
```

macOS/Linux shell crates and mobile entry points are deferred to Phase 6; do not scaffold empty crates for unproved platforms. Source ownership moves to these paths only when their migration tasks are authorised. Legacy `src/Drape.*` paths remain historical and are not renamed by this change.

## Migration sequence and evidence

1. R0-01 establishes this documentation baseline. Independent role review is required before it is DONE.
2. R0-02 creates the workspace, pins versions and validates licences, dependency boundaries and MSRV/Windows minimums. R0-03 adds Rust CI. Both are future implementation work.
3. R0-04 through R0-08 repeat desktop host, icon recovery, icon extraction, renderer/transparency and hook experiments in Rust. Each has an explicit Windows-build/DPI/monitor matrix. No production icon hiding before recovery is demonstrated.
4. R0-09 proves application lifecycle, tray integration and a resource baseline. R0-10 records the evidence and final host/backend/renderer decisions. A toolkit choice is not a desktop-host acceptance result.
5. R1-01 through R1-06 port domain contracts, durable config, layout, UI/items and lifecycle behaviour. Then P2–P5 implement the existing product phases. A parity matrix maps legacy behaviour and tests to Rust evidence; tests and code are not mechanically translated as proof of correctness.
6. Source retirement is a separate owner-authorised action after Rust parity and recovery gates pass. No deletion of legacy code or user files is part of this plan execution today.

## Compatibility and safety

- Preserve schema version 1, JSON camelCase names, enum values, IDs, timestamps, unknown fields and file locations. Changing implementation language alone does not justify a schema bump.
- Use hand-written legacy fixtures to verify semantic load/save compatibility. Preserve exact unknown values and avoid path encoding loss; decide representation before porting persistence.
- Core ports become Rust traits without the C# `I` prefix (`FileSystem`, `Clock`, `DesktopHost`, `DesktopIcons`, etc.). `PlatformCapabilities` is neutral capability data. Concrete Win32 handles never enter domain models.
- The UI owns windows; the shell borrows a native handle only during a defined lifetime. The composition root joins the two through an opaque window identifier. Reattachment and destruction invalidate old bindings.
- RAII manages native resources, but destructors and panic hooks cannot recover from abort, forced kill or reboot. A separate Guard, durable intent recorded before hiding, recovery at next start and `--restore-desktop` are mandatory.
- Guard must be started and acknowledged before icons are hidden. If it cannot be armed, continue with native icons visible. Restore the state Drape changed; respect icons already hidden by the user.
- App, Guard and restore CLI share serialized per-user/session ownership transitions, authenticated peer/generation messages and bounded parsing. Validate single-instance/live-generation ownership before startup recovery; never let a second launch restore icons still owned by a live session. Guard loss while App lives causes immediate restoration of owned changes, disables native-icon mutation including quick-hide, and forbids rehide until a new Guard is armed. R0-05 covers readiness-to-hide races, corrupt/unsupported state, spoofed messages and simultaneous failures; no fallback is a blind toggle or an assumption that successful launch lasts forever.
- Infrastructure coordinates persistence through injected FileSystem/DesktopLocations ports; the low-level OS-specific FileSystem implementation is selected by App from the matching shell adapter. This permits Windows atomic replacement/file identity without an Infrastructure-to-Shell dependency or direct Win32 calls in shared code.
- Port specifications define call thread/apartment, owned/borrowed arguments, errors, cancellation, response generation and shutdown. No blanket Send + Sync requirement; native resources remain alive until native work stops even if the UI has cancelled its request.

## Performance acceptance

Keep the existing ceilings: average idle CPU < 0.5%, combined App + Guard working set < 100 MB with 20 Drapes × 20 items, and startup median < 1.5 s on SSD. These are ceilings, not promised Rust results. Record private bytes, GPU memory, handle counts and per-panel growth as well.

Use a pinned Release build with the toolchain, dependency lock, Slint backend/renderer, Windows build, hardware, DPI, monitor count, cache state, icon counts and blur setting recorded. Measure 0/1/5/20 panels, warm and cold startup separately, and 30-minute retention. Report application and Guard executables, compressed portable download, total installed size and any runtime prerequisites separately. A numeric distribution-size gate and tighter RAM target await R0-09 evidence (Q-20).

Compare eligible Slint backends/renderers under the same workload. Avoid an accidental Qt deployment or multiple unused renderer stacks; do not choose a renderer from marketing RAM figures. Stop rendering when idle, bound icon caches and queues, and avoid adding a general async runtime until a measured need exists.

## Reference assessment

- [Owner-supplied Gemini discussion](https://share.gemini.google/UwyvsCzSs0aM), read 2026-11-25: useful brainstorming for Rust + Slint + Win32, not primary API documentation or test evidence. Its version pins, sample code, estimates, low-RAM claims and automatic Progman fallback are not adopted as verified facts.
- [Slint official site](https://slint.dev): native compilation and selectable rendering methods; embedded runtime figures do not describe total Windows process memory.
- [Slint royalty-free desktop/mobile/web licence 2.0](https://github.com/slint-ui/slint/blob/master/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md), read 2026-11-25: attribution via AboutSlint or the public download-page badge. Recheck the licence shipped with the pinned release in R0-02; do not describe Slint as MIT.
- [Slint documentation](https://docs.slint.dev) and [Microsoft Win32 documentation](https://learn.microsoft.com/windows/win32/): primary references for subsequent API verification. Exact versions and supported public handle/event-loop APIs remain unselected.

## Open decisions

Q-19: blur's MVP priority. Q-20: measurable distribution-size and tighter memory targets. Q-21: backend/renderer, raw handle lifecycle, native drag/drop and UIA integration. Q-22: Rust-compatible packaging/updater path. The existing updater preference is evaluated, not silently replaced. Non-Windows minimums are retained as planning targets and must be revalidated against Rust/Slint rather than old Avalonia support tiers.

Q-23: persisted versus derived representation and ownership semantics for pin-to-all-pages, tabs and portal projections. These features remain planned; their implementation requires feature specs and a versioned migration when schema-1 compatibility cannot be preserved. Current conceptual fields are not an accepted future wire format.
