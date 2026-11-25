# 00 — Testing Strategy

Principle: as much logic as possible lives in Core so it can be tested fast and deterministically; platform integration is tested through contracts, structured manual testing and limited E2E.

> **An assertion that a test passed is not evidence. Name the test.** "Layout tested, all good" with no test named is worse than saying nothing, because it stops anyone looking again.

## Pyramid

| Layer | Scope | Tools | Runs on | Target |
|---|---|---|---|---|
| Core unit | Models, `RuleEngine`, `LayoutService`, schema migration, snap, path normalisation | Rust unit tests, standard assertions; proptest only if useful | Every commit (CI) | Line coverage ≥ 80% |
| Architecture tests | Actual Cargo dependency/source boundaries | Rust integration tests + cargo metadata | CI | 100% pass |
| Infrastructure integration | `ConfigStore` (atomic, backup, corrupt), `DesktopItemSource` (temp folders) | Rust integration tests, isolated temp directories | CI | All main failure paths |
| View-model tests | VM behaviour with fake interfaces | Rust tests and hand-written port fakes | CI | Main flow per feature |
| Headless UI | Slint views/controls, bindings | Pinned Slint testing/headless support, verified in R0-07 | CI | Main controls |
| Shell contracts | Fake `DesktopHost` / `DesktopIcons` to test consumers; "harness" tests for real implementations | Rust tests/native harnesses | Local / Windows runner | Main flow |
| E2E smoke | Start app, create a Drape, add an item, close, reopen | Windows UIA harness selected after R0-07 plus manual checks | Local / self-hosted runner with an interactive desktop | 5–10 scenarios |
| Structured manual | Environment matrix and risky scenarios | Checklist below | Before each release and each Shell change | Every row ticked |

Hosted CI runners do not guarantee an interactive desktop session. Run native/E2E cases locally or on an interactive self-hosted VM, and attach their evidence to the change. **Every merge:** locked build, rustfmt, denied-warning Clippy, unit/architecture/integration and supported headless UI checks. **Native safety/UI changes additionally require the relevant real-desktop evidence before merge**; lack of a hosted runner is not a safety waiver. If pinned Slint headless support lacks a required operation, test the presentation logic deterministically and cover that operation in the real UI matrix; never mark an unsupported check passed.

## Test Support

- `FakeClock`, `FakeFileSystem` (in-memory), `FakeDesktopHost`, `FakeDesktopIcons`, `FakeDisplayMonitor`, `FakePlatformCapabilities` in `crates/drape-test-support`.
- JSON config fixtures per schema version in `crates/drape-core/tests/fixtures/`.
- Object builders (`DrapeBuilder`, `ConfigBuilder`) keep tests short.

## Mandatory Tests

Core:
- Snap to edges/other Drapes with an 8 DIP threshold; Alt disables it.
- Clamp to the work area; at least 40 DIP stays visible.
- Rescale on DPI/resolution change from the stored position, with no drift after 10 back-and-forth cycles.
- Relocate a missing monitor and bring the Drape back when it returns.
- `RuleEngine`: priority, combined conditions, `pinnedByUser` is not re-sorted.
- Path normalisation (case, environment variables, trailing slash, UNC, long paths).
- Schema migration v(N) → v(N+1) from fixtures; unknown properties are preserved.
- Capability gating: with `FakePlatformCapabilities` all-off, no feature that needs a capability is invoked.

Infrastructure:
- Atomic save: simulated failure mid-write leaves the old config intact.
- Corrupt config → quarantine + newest valid backup.
- Watcher debounce: 1,000 successive changes produce few batches.
- Rename tracked through the file ID.

UI / view-model:
- Drop adds an item and rejects a duplicate; dropping an item that lives in another Drape moves it.
- Removing an item does not delete the file.
- Rollup remembers the previous height.

Architecture:
- `drape-core`, `drape-ui` and `drape-infrastructure` reference no `drape-shell-*`, no direct platform bindings or unsafe FFI; neither Core nor shared UI performs I/O/network calls. Infrastructure accesses platform-neutral I/O ports; approved watcher/file adapters may have documented transitive native dependencies.

## Manual Environment Matrix

| Dimension | Values |
|---|---|
| OS | Windows 11 24H2+ (primary), Windows 11 23H2, **Windows 10 22H2 (minimum — a release gate, not optional)** |
| DPI | 100%, 125%, 150%, 200%, 300% (and mixed across monitors) |
| Monitors | 1; 2 (same DPI); 2 (different DPI); 4; negative virtual-screen origins; plug/unplug while running |
| Theme | Light, dark, high contrast |
| Desktop | Local; redirected to OneDrive (Known Folder Move); Files On-Demand |
| Item count | 0, 20, 500 |
| OS language | id-ID, en-US |

This covers the advertised NFR-07 range (100–300%, 1–4 monitors); initial two-monitor spikes do not prove the four-monitor release promise. Record exact OS build/edition and hardware. Select Windows 11 primary/secondary builds from the supported release policy at test time and record why; do not treat the old 23H2 row as a perpetual support promise. Minimum Windows 10 22H2 remains mandatory regardless of upstream lifecycle. Use pairwise coverage for ordinary combinations, but run every safety-critical exit/recovery scenario on both OS families with at least one mixed-DPI configuration. If required hardware is unavailable, report an uncovered release gate; do not invent coverage. Phase 6 adds its own matrix before acceptance.

## Risky Manual Scenarios (release checklist)

- [ ] Win+D, then the taskbar "Show desktop" corner: Drapes stay visible.
- [ ] Restart `explorer.exe` (in a VM): Drapes recover and icon state is in sync.
- [ ] Change resolution, DPI and orientation: layout recovers with no drift.
- [ ] Unplug and plug a monitor while running: Drapes move and return.
- [ ] Sleep/resume, lock/unlock, fast user switching, RDP in/out.
- [ ] Kill the Drape process in Task Manager: icons return (Guard).
- [ ] Hard reboot while icons are hidden: the next start restores them.
- [ ] `--restore-desktop` runs with the main app absent.
- [ ] Launch 2 instances: the second exits politely.
- [ ] Drag-drop from Explorer, from the desktop, from an elevated app (refused with a clear message).
- [ ] Items with Unicode names, very long names, very long paths, broken shortcuts.
- [ ] Fullscreen apps/games and presentations: Drape does not appear on top.
- [ ] Shell context menu with third-party handlers installed (archivers, cloud clients).
- [ ] Light/dark/high-contrast theme changes while running.
- [ ] Deliberately corrupt config: the app recovers and explains.
- [ ] Keyboard-only entry through the notification-area menu/Settings focuses any Drape without a configured global hotkey; move, resize, create, rename, remove, settings and emergency restore are reachable without a mouse.
- [ ] Narrator/UIA expose control roles, names, values, focus, selection and disabled states; labels remain usable at 300% DPI and OS text scaling.
- [ ] Quick-hide preserves original/user-selected native-icon visibility and uses the same armed recovery contract whenever it changes native icons.
- [ ] Read-only/full data directory, denied atomic replace, interrupted save, watcher overflow and invalid recovery state fail visibly without losing the last committed layout or blocking icon restoration.
- [ ] Install, portable run, update interruption/rollback, uninstall while App/Guard run, missing/corrupt Guard and invalid package signature preserve recoverability; uninstaller cannot remove the last usable recovery binary before restoration.

## Performance Tests

| Test | How | Pass |
|---|---|---|
| Idle CPU | App + Guard combined, Performance Monitor, 10 min, 20 Drapes × 20 items | < 0.5% average |
| Memory | Combined App + Guard working set after 30 min; private bytes/GPU memory also reported | < 100 MB working set, no steady growth |
| Startup | Process start to all configured Drapes visible and interactive, host attached and required Guard ready; 10 cold and 10 warm runs separately | median < 1.5 s for each declared profile; report every sample |
| Handle leaks | Loop create/delete 1,000 Drapes, watch handle/GDI/USER counts | Flat |
| Hook | Measure callback latency | < 1 ms average, never exceeding the OS timeout |

Record whether CPU percentage is normalised to one logical processor or to the whole machine and publish both the raw counter and machine-normalised result; compare the 0.5% gate against machine-normalised App + Guard CPU. Use the same convention across baselines. Record memory units (decimal MB versus MiB), collector/tool version and sampling interval; the 100 MB ceiling uses decimal MB. Summed working set can double-count shared pages, so label it as a process-sum and report private bytes separately rather than claiming it is unique physical RAM.

For the standard profile, seed the same 20-panel ×20-item mixed shortcut/file/folder dataset, wait for icon loading to settle, state cache warmth, and record blur/effect settings. Run each supported renderer/effect profile on the same machine; high-contrast fallback must not be substituted silently for an expensive default. A machine without a supported blur capability reports the fallback profile and cannot prove blur performance. Collect at least 30 minutes for retention; report baseline/final/peak and trend, plus App/Guard restart or Explorer-recovery effects. Fixed thresholds alone do not prove absence of a leak; repeat create/delete and icon-cache turnover with a bounded steady state after warm-up.

## Bug Reporting

Include: steps, expected and actual result, OS build, DPI/monitors, Drape version, logs (`logs/`), and a screenshot if relevant. Labels: `area/shell|ui|core|infra`, `severity/blocker|major|minor`.

## Release Criteria

- All CI gates are green.
- The full manual checklist has been run on Windows 11 and Windows 10 22H2, with at least one non-100% DPI configuration.
- No open blocker/major defect in safety, data durability, accessibility or other required functionality; shell defects are not the only release blockers.
- Performance tests pass.

## Distinguish "Renders" From "Works"

A claim such as "verified on Windows 11" must name the build, DPI and monitors. A feature checked on one machine is *known to work on that machine*, nothing more. Records and status updates keep that distinction; rounding it up is how a board stops being trusted.

## Rust migration evidence (ADR-025)

No Rust tests were run in the documentation migration. The [old Phase 1 baseline](./PHASE-1-BASELINE.md) describes C# only. R1-06 produces a fresh baseline, mapping legacy behaviour to new named Rust tests; a mechanical translation is not correctness evidence.

Before Rust merge: locked Release build, rustfmt, Clippy with denied warnings, cargo test and Core coverage >=80%; check actual dependency boundaries including negative mutation. Headless/presentation tests are deterministic; real HWND/host/transparency/Win+D/COM/drag-drop/UIA require actual Slint on the interactive Windows matrix.

Required recovery cases: user-hidden icons preserved; Guard failure leaves icons visible; durable intent/Guard readiness before hide; restore not blocked by config/log failure; normal error/panic, abort/forced App kill, both-process loss/next-start, standalone restore without renderer, invalid/legacy state records and process-identity mismatch. Never automate Explorer termination.

Native safety review covers handle ownership, COM apartment cleanup, callback ABI/lifetime/no unwinding, queue overflow and shutdown races. Legacy schema fixtures test unknown/nested fields and numeric precision; version 1 stays compatible without a language-only bump.

Exercise recovery ownership as a state machine. App, Guard and CLI serialise state changes through the same per-user/session owner. A second App must not replay stale recovery against a live generation; simultaneous CLI/Guard restore is idempotent. Explicit restore during a live session revokes or updates its hide authority so App cannot immediately hide again. Validate generation/handle identity, journal version/session ownership, caller authentication and IPC acknowledgements; reject cross-user/session or stale messages without changing visibility.

Test Guard death before readiness, after readiness but before mutation, and while App stays alive with icons hidden. App detects loss, restores through the coordinator and disables further hiding until Guard is ready again. Inject failures at every journal/readiness/mutation/verification/restore/disarm transition, including timeouts and queue saturation. Prove bounded fail-closed outcomes; never report failed restore as clean shutdown. A readiness acknowledgement alone does not prove Guard is still alive at mutation. Label simulation, real termination and manual VM evidence separately; record maximum recovery delay and define its acceptable bound in the spike contract before testing.

Measure Release 0/1/5/20 panels, cold/warm startup separately, 30-minute retention, App + Guard CPU/working set/private bytes, GPU memory, handles and per-panel growth. Record toolchain/lockfile/features/backend/renderer, machine/OS build/DPI/monitors/cache/blur state. Report App/Guard binary, compressed download, installed size and prerequisites separately; numeric size/tighter RAM gate awaits Q-20. Keep existing CPU <0.5%, combined working set <100 MB, startup median <1.5 s gates. Do not claim marketing figures as measurements.
