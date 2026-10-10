# 00 — Testing Strategy

Principle: as much logic as possible lives in Core so it can be tested fast and deterministically; platform integration is tested through contracts, structured manual testing and limited E2E.

> **An assertion that a test passed is not evidence. Name the test.** "Layout tested, all good" with no test named is worse than saying nothing, because it stops anyone looking again.

## Pyramid

| Layer | Scope | Tools | Runs on | Target |
|---|---|---|---|---|
| Core unit | Models, `RuleEngine`, `LayoutService`, schema migration, snap, path normalisation | xUnit, Shouldly | Every commit (CI) | Line coverage ≥ 80% |
| Architecture tests | Dependency rules between projects/assemblies | xUnit + reflection/analyzer | CI | 100% pass |
| Infrastructure integration | `ConfigStore` (atomic, backup, corrupt), `DesktopItemSource` (temp folders) | xUnit, temp directories | CI | All main failure paths |
| View-model tests | VM behaviour with fake interfaces | xUnit, NSubstitute | CI | Main flow per feature |
| Headless UI | Avalonia views/controls, bindings | Avalonia.Headless.XUnit | CI | Main controls |
| Shell contracts | Fake `IDesktopHost` / `IDesktopIcons` to test consumers; "harness" tests for real implementations | xUnit | Local / Windows runner | Main flow |
| E2E smoke | Start app, create a Drape, add an item, close, reopen | FlaUI (UIA3) | Local / self-hosted runner with an interactive desktop | 5–10 scenarios |
| Structured manual | Environment matrix and risky scenarios | Checklist below | Before each release and each Shell change | Every row ticked |

Note: hosted CI runners do not guarantee an interactive desktop session. Treat E2E and part of the Shell tests as local or self-hosted, not as merge gates. **Merge gate:** build, format, unit, architecture, integration (and headless UI).

## Test Support

- `FakeClock`, `FakeFileSystem` (in-memory), `FakeDesktopHost`, `FakeDesktopIcons`, `FakeDisplayMonitor`, `FakePlatformCapabilities` in `tests/Drape.TestSupport`.
- JSON config fixtures per schema version in `tests/Drape.Core.Tests/Fixtures/`.
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
- `Drape.Core`, `Drape.UI` and `Drape.Infrastructure` reference no `Drape.Shell.*`, no `System.Windows`, no `Microsoft.Win32`, and make no network call.

## Manual Environment Matrix

| Dimension | Values |
|---|---|
| OS | Windows 11 24H2+ (primary), Windows 11 23H2, **Windows 10 22H2 (minimum — a release gate, not optional)** |
| DPI | 100%, 125%, 150%, 200% (and mixed across monitors) |
| Monitors | 1; 2 (same DPI); 2 (different DPI); plug/unplug while running |
| Theme | Light, dark, high contrast |
| Desktop | Local; redirected to OneDrive (Known Folder Move); Files On-Demand |
| Item count | 0, 20, 500 |
| OS language | id-ID, en-US |

Phase 6 adds one matrix per platform; a platform's matrix is written before its Shell project is accepted.

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

## Performance Tests

| Test | How | Pass |
|---|---|---|
| Idle CPU | Performance Monitor, 10 min, 20 Drapes × 20 items | < 0.5% average |
| Memory | Working set after 30 min | < 100 MB, no steady growth |
| Startup | From process start to Drapes visible, 10 runs | median < 1.5 s |
| Handle leaks | Loop create/delete 1,000 Drapes, watch handle/GDI/USER counts | Flat |
| Hook | Measure callback latency | < 1 ms average, never exceeding the OS timeout |

## Bug Reporting

Include: steps, expected and actual result, OS build, DPI/monitors, Drape version, logs (`logs/`), and a screenshot if relevant. Labels: `area/shell|ui|core|infra`, `severity/blocker|major|minor`.

## Release Criteria

- All CI gates are green.
- The full manual checklist has been run on Windows 11 and Windows 10 22H2, with at least one non-100% DPI configuration.
- No open `blocker`/`major` bug in the Shell area.
- Performance tests pass.

## Distinguish "Renders" From "Works"

A claim such as "verified on Windows 11" must name the build, DPI and monitors. A feature checked on one machine is *known to work on that machine*, nothing more. Records and status updates keep that distinction; rounding it up is how a board stops being trusted.
