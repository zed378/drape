# 00 — System Architecture

Status: draft. Related decisions: ADR-005 (config), ADR-006 (icon strategy), ADR-007 (window per Drape and host layer), ADR-008 (single process + Guard), ADR-012 (Avalonia), ADR-015 (platform capabilities) in [`MEMORY/DECISIONS.md`](../../MEMORY/DECISIONS.md).

## Layers

```
┌──────────────── Drape.App.Desktop (composition root: Windows / macOS / Linux) ────────────────┐
│ Program · Host/DI · SingleInstance · Tray · Startup/Shutdown · CLI flags                       │
└───────┬─────────────────────────────┬──────────────────────────────┬───────────────────────────┘
        │                             │                              │
┌───────▼────────┐          ┌─────────▼───────────┐        ┌─────────▼──────────────┐
│   Drape.UI     │          │ Drape.Infrastructure │        │ Drape.Shell.<Platform> │
│ Avalonia views │          │ ConfigStore          │        │ Windows: Win32/COM     │
│ ViewModels     │          │ DesktopItemSource    │        │ Mac: Cocoa/CoreGraphics│
│ Controls·Theme │          │ Logging              │        │ Linux: X11/Wayland     │
│ Settings       │          └─────────┬────────────┘        │ host · icons · hook ·  │
└───────┬────────┘                    │                     │ DPI · monitor · menu · │
        │                             │                     │ autostart · locations  │
        └──────────────┬──────────────┴─────────────────────┴───────────┬────────────┘
                ┌──────▼───────┐
                │  Drape.Core  │  domain · rules · layout math · interfaces · capabilities
                └──────────────┘

Drape.App.Android / Drape.App.iOS: mobile composition roots (Phase 6) — UI + Core + Infrastructure,
no desktop-layer Shell project.
Drape.Guard (separate tiny process): waits for the main process, restores desktop icons when it ends.
```

**Dependency direction:** `App.* → UI, Infrastructure, Shell.<current platform>, Core`; `UI → Core`; `Infrastructure → Core`; `Shell.* → Core`; `Guard → Core, Shell.<platform>` (icon-restore part only); `Core → nothing`.
`UI` never references `Shell`; they meet through interfaces in Core, injected by the App. An architecture test enforces this (see [`../ENGINEERING/00-CODING-STANDARDS.md`](../ENGINEERING/00-CODING-STANDARDS.md)).

## Components and Responsibilities

| Component | Project | Responsibility |
|---|---|---|
| `Drape`, `DrapeItem`, `Page`, `Rule`, `Snapshot` | Core | Pure domain model |
| `DrapeManager` | Core | The Drapes of each page; add/remove/move items; change events |
| `RuleEngine` | Core | Evaluate rules against items, produce placement decisions |
| `LayoutService` | Core | Position maths: snap, clamp, scaling on DPI/resolution change, monitor relocation |
| `IPlatformCapabilities` | Core (interface) / Shell | Reports what the current platform can do (desktop layer, hide native icons, global mouse hook, acrylic blur, hotkeys …) so UI and Core never test the OS |
| `IConfigStore` | Core (interface) / Infrastructure | Load, atomic save, backup, migrate |
| `IDesktopLocations` | Core (interface) / Shell | Where the user's Desktop folder(s) are (Windows: Known Folder API) |
| `IDesktopItemSource` | Core (interface) / Infrastructure | Enumerate and watch the Desktop contents, using `IDesktopLocations` and `FileSystemWatcher` |
| `IDesktopHost` | Core / Shell | Attach Drape windows to the desktop layer, keep z-order, recover after the shell restarts |
| `IDesktopIcons` | Core / Shell | Show/hide native icons, read state, restore |
| `IIconProvider` | Core / Shell | Item icons/thumbnails as `IconBitmap` (neutral BGRA pixels + size; **not** a UI-framework image type), with a cache |
| `IInputHook` | Core / Shell | Desktop double-click and right-drag events, on a dedicated thread |
| `IShellContextMenu` | Core / Shell | The platform's item context menu |
| `IDisplayMonitor` | Core / Shell | Monitor list, DPI, display-change events |
| `IAutostart` | Core / Shell | Run-at-login (Windows: HKCU Run key) |
| `IBlurEffect` | Core / Shell | Acrylic/Mica/blur where available, solid fallback otherwise |
| `DrapeWindow` + `DrapeViewModel` | UI | View and interaction for one Drape |
| `TrayService`, `SettingsWindow` | UI / App | Tray menu and settings |
| `ThemeService` | UI | Theme, accent, high contrast, blur |
| `Drape.Guard` | Guard | Restore icons when the main process ends |

Interfaces for services that every platform must provide live in Core; an interface that only some platforms can implement is guarded by a capability flag, and a platform that cannot implement it registers no implementation (or a null object) instead of throwing.

## Main Flows

### Startup

1. Parse arguments. `--restore-desktop` restores icons and exits.
2. Single instance (named mutex or platform equivalent). A second instance signals "show settings" and exits.
3. Load config (`IConfigStore`); migrate if `schemaVersion` is older.
4. Read `IPlatformCapabilities`; features the platform lacks are disabled up front, with a message in settings.
5. `IDesktopHost.Attach()` finds the desktop window and prepares the host parent.
6. `IDesktopItemSource` scans the Desktop and the shared Desktop.
7. `DrapeManager` reconciles membership: drop missing references (after the grace period), place new items by rule or into the Inbox.
8. Create a `DrapeWindow` per Drape, attach it to the host, show it.
9. `IDesktopIcons.Hide()` if the option is on; write `state.json` (`iconsHiddenByDrape: true`); launch Guard.
10. Install `IInputHook`, tray, display monitor and Desktop watcher.

### Shutdown (normal)

1. Remove hook and watchers.
2. Flush config (synchronous save).
3. `IDesktopIcons.Show()`; set `iconsHiddenByDrape: false`.
4. Close Drape windows, release the host, dispose services.

### Crash or kill

- `AppDomain.UnhandledException` and the UI-thread unhandled-exception handler: log, restore icons, save what can be saved, exit.
- Forced kill: Guard notices the main process ended and restores the icons.
- Guard also dies (hard reboot): the next Drape start reads `state.json` and restores icons; `--restore-desktop` is available by hand.

### Desktop file changes

`FileSystemWatcher` (thread pool) → 250 ms debounce queue → batch → marshal to the UI thread → `DrapeManager.Reconcile(batch)` → view-models update collections → icons load asynchronously.

### Display changes

Platform notification (Windows: `WM_DISPLAYCHANGE`, `WM_DPICHANGED`, `WM_SETTINGCHANGE`) → `IDisplayMonitor` raises an event → `LayoutService` recomputes from the **stored** position → `DrapeWindow` updates its bounds.

### Shell restart

Windows: a hidden message window receives the registered `TaskbarCreated` message. `IDesktopHost.Reattach()` finds the new desktop window, re-parents, restores z-order, then `IDesktopIcons` reconciles state. Other platforms: see [`../PLATFORM/01-PLATFORM-SUPPORT.md`](../PLATFORM/01-PLATFORM-SUPPORT.md).

## Threading

| Thread | Work |
|---|---|
| UI | All Avalonia windows, view-models, bindings (`Dispatcher.UIThread`) |
| Hook | Dedicated thread with a message loop for the global mouse hook; the callback only enqueues events to a channel |
| Thread pool | File watcher, icon decoding, config save |
| Guard | Separate process, no UI |

Rules:
- Only the UI thread touches Avalonia objects. Marshal at the boundary with `Dispatcher.UIThread`.
- Hook callbacks: no large allocations, no I/O, no waiting for the UI thread — enqueue and return. The OS disables slow hooks.
- Icons are decoded on the thread pool into `IconBitmap`; the UI thread creates the Avalonia bitmap from it. Whether Avalonia bitmaps may safely be created off the UI thread is confirmed in spike P0-05.

## Error Handling

- Clear boundary: Shell turns `HRESULT`/`Win32Error` into a typed exception (`ShellException`) or a `Result` for expected failures.
- Non-critical failure (one icon fails to load) never brings the app down: log it and show a generic icon.
- Critical startup failure (cannot attach host) → degraded mode: Drapes appear as ordinary windows at the bottom of the z-order, native icons are *not* hidden, the user is told.

## Observability

- Serilog to a rolling file (`logs/`), Information by default, Debug optional in settings.
- File paths are logged only at Debug level (privacy; see [`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md)).
- "Copy diagnostic info" in settings: version, OS build, DPI, monitors, host status, capabilities. No user paths.

## Extensibility

No plugins in v1. Internal extension points: `IRuleCondition` and `IRuleAction` in Core, so new rules can be added without changing the engine.

## Open Decisions

- Final host strategy (parent `WorkerW` vs bottom-most window): spike P0-03 → ADR-007.
- Most stable way to hide native icons on Windows 11 24H2+: spike P0-04 → ADR-006.
- How `Drape.App.Desktop` references the right `Shell.<Platform>` project (multi-targeting vs conditional references vs separate heads): decided in P0-01 → recorded as an ADR.
- Whether an Avalonia window can be hosted in the desktop layer with per-pixel transparency: spikes P0-03, P0-06.
