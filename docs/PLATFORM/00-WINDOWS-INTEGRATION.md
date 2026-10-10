# 00 — Windows Integration (Win32)

The highest-risk document. Many techniques here are not officially documented and can change between Windows builds.
**Every claim below must be verified by a spike (P0-03 … P0-07) on the target Windows build before it becomes a foundation.** Results are recorded in `MEMORY/records/` and, when they decide something, as ADRs.

Scope: Windows only; all code lives in `Drape.Shell.Windows` (and the icon-restore part of `Drape.Guard`). UI code is Avalonia and never calls these APIs directly ([`../ENGINEERING/00-CODING-STANDARDS.md`](../ENGINEERING/00-CODING-STANDARDS.md)).

## 1. Desktop Window Hierarchy (overview)

Explorer generally draws the desktop through a set of windows such as:

```
Progman ("Program Manager")
 └─ SHELLDLL_DefView
     └─ SysListView32  ("FolderView")   ← desktop icons
WorkerW (one or more)                    ← used for wallpaper/slideshow and effects
```

The real arrangement varies: `SHELLDLL_DefView` may sit under `Progman` or under one of the `WorkerW` windows, and this changes between Windows 10/11 builds (including 24H2).
Never hard-code a window path. Discover it dynamically: `FindWindow("Progman")` → check for a `SHELLDLL_DefView` child → if absent, enumerate `WorkerW` with `EnumWindows` and find the one that has a `SHELLDLL_DefView` child.

Tools: Spy++ and `Inspect.exe` to examine the hierarchy on the target machine.

## 2. Placing Drape on the Desktop Layer

Goal: Drapes appear above the wallpaper (replacing the icons), below all application windows, stay visible after Win+D, and do not appear in the taskbar or Alt+Tab.

Each Drape is an Avalonia window; its native handle is obtained from the window's platform handle (`TryGetPlatformHandle()`, an `HWND` on Windows). Window chrome is removed, taskbar presence disabled, and transparency requested through Avalonia's transparency-level hint (property names differ between Avalonia 11 and 12; verify against the version locked in P0-01).

| Strategy | How | Pros | Risks |
|---|---|---|---|
| A. Parent to the desktop host | `SetParent` of the Drape's `HWND` to the right `Progman`/`WorkerW` (sometimes requires sending message `0x052C` to `Progman` so a `WorkerW` is created) | Survives Win+D, natural z-order | Depends on internal structure; changes between builds; per-pixel transparency on a child window and Avalonia's compositor may misbehave |
| B. Top-level window at the bottom of the z-order | `WS_EX_TOOLWINDOW`, `SetWindowPos(HWND_BOTTOM, SWP_NOACTIVATE)`, re-assert on activation/z-order events | Independent of desktop structure; normal Avalonia transparency | May be hidden by Win+D ("Show desktop" hides all top-levels); needs event hooks to show again; sometimes flickers |
| C. Hybrid | Try A; if verification fails, fall back to B | Most robust | Complex, two test paths |

Interim recommendation: **C**, behind the `IDesktopHost` contract that hides the difference. Final decision in ADR-007 after spike P0-03.

A strategy passes only if:
- It is visible above the wallpaper and does not cover application windows.
- It stays visible after Win+D and Win+, (peek).
- It does not steal focus when it appears.
- It recovers after an Explorer restart.
- It works at 100%, 150%, 200% DPI and with 2 monitors at different DPI.
- Transparency and blur stay correct.

## 3. Hiding Native Icons

Common approach: send `WM_COMMAND` with `wParam = 0x7402` to the `SHELLDLL_DefView` window — the same as the "Show desktop icons" menu item. The command is a **toggle**, so read the state first (for example the visibility of `SysListView32` via `IsWindowVisible`) so the state is not flipped by mistake.

Rules:
- Always read state before and after sending the command; record it in `state.json`.
- If the user changes it through the Windows menu, detect it and do not fight it.
- Restore on every exit path ([`../ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](../ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md)): normal, exception, Guard, `--restore-desktop`, next start.
- Alternatives to evaluate in spike P0-04: hiding `SysListView32` with `ShowWindow(SW_HIDE)` (risk: Explorer shows it again), or the registry setting `HideIcons` plus a broadcast (risk: persistently changes user settings — avoid unless truly necessary).
- Never kill `explorer.exe` to hide icons.

## 4. Enumeration and Item Icons

- Item source: the file system under the Desktop and the Public Desktop (Known Folder API: `SHGetKnownFolderPath`), exposed to Core as `IDesktopLocations`. Ignore `desktop.ini` and hidden/system attributes according to the Explorer settings.
- Icons and thumbnails: `IShellItemImageFactory::GetImage` (size and `SIIGBF_*` flags), on the thread pool. The result is converted to neutral `IconBitmap` pixel data (BGRA + size) and handed to Core/UI, which builds the Avalonia bitmap. LRU cache keyed by path + size + timestamp.
- Overlays (shortcut arrow, cloud/OneDrive status) may be lost with this approach; evaluate `SHGetFileInfo` / `IShellIconOverlay` in spike P0-05. Reading icons must not trigger OneDrive hydration.
- Virtual items (This PC, Recycle Bin, Network): deferred beyond the MVP; they need the Desktop `IShellFolder` and PIDLs, not file paths.

## 5. Shell Context Menu

- Obtain `IContextMenu` from `IShellFolder::GetUIObjectOf` for the item's PIDL, display it with `TrackPopupMenuEx`, and forward menu messages to `IContextMenu2/3` so submenus (Send to, Open with) work.
- Third-party handlers can be slow or crash. Load on a separate STA thread with a timeout; fall back to a minimal menu.
- Needs an owner window that receives `WM_INITMENUPOPUP`, `WM_MEASUREITEM`, `WM_DRAWITEM`, `WM_MENUCHAR`. With Avalonia this is a small hidden native window owned by the Shell project, not the Avalonia window.

## 6. Drag and Drop

- Receive: `CF_HDROP` and `CFSTR_SHELLIDLIST`. Use Avalonia's drag-drop (`DragDrop`, `DataFormats.Files`) where sufficient; fall back to a native `IDropTarget` if the Shell IDList or another format cannot be read through Avalonia.
- Send (dragging an item out): build an `IDataObject` with `CF_HDROP` so Explorer and other apps can accept it.
- **UIPI:** a non-elevated app cannot receive drops from an elevated process. Explain it in user documentation; never raise privileges as a workaround.

## 7. Mouse Hook for Quick-Hide and Right-Drag

- Install `WH_MOUSE_LL` on a dedicated thread with a message loop. The callback only: reads `MSLLHOOKSTRUCT`, decides a coarse event type (right button down/up, left double-click), pushes it to a `Channel<T>`, and calls `CallNextHookEx`.
- Deciding whether the point is "empty desktop" happens **outside the callback**: `WindowFromPoint`/`ChildWindowFromPoint` → confirm the window at that point is the desktop `SysListView32` → `LVM_HITTEST` (cross-process, needs shared memory; evaluate in P0-07) or UI Automation on the desktop to find out whether the point is over an item.
- Double-click timing follows `GetDoubleClickTime()` and `SM_CXDOUBLECLK`.
- The hook must not log or store coordinates/clicks beyond what the feature needs ([`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md)).
- Alternatives without a global hook: subclassing the desktop window (impossible cross-process), or a full-screen transparent window that catches clicks (blocks native interaction; rejected).
- Hook installation failure is non-fatal: quick-hide and right-drag are disabled with a message in settings, via `IPlatformCapabilities.HasGlobalMouseHook`.

## 8. DPI and Monitors

- Manifest: `PerMonitorV2`. Use `WM_DPICHANGED` and `GetDpiForWindow`. Avalonia scales its own content; Shell code works in physical pixels and converts at the `IDisplayMonitor` boundary.
- Monitor list: `EnumDisplayMonitors` + `GetMonitorInfo`; stable IDs from the device name/`DISPLAY_DEVICE` (not the index).
- Change messages: `WM_DISPLAYCHANGE`, `WM_SETTINGCHANGE` (work area), `WM_DPICHANGED`. Debounce 500 ms, because Windows sends several in a row.
- Position storage: see [`../ARCHITECTURE/02-DATA-MODEL.md`](../ARCHITECTURE/02-DATA-MODEL.md) (`Bounds`). Always recompute from the stored values, never from the current position.

## 9. Visual Effects

- Transparent background: an undecorated Avalonia window with a transparency-level hint (transparent; or acrylic/Mica where supported). Per-pixel transparency is the simple path; native blur needs the platform backdrop APIs.
- Blur/acrylic: wrapped in `IBlurEffect` with implementations (a) Avalonia's acrylic/blur transparency level, (b) `SetWindowCompositionAttribute` (undocumented), (c) `DwmSetWindowAttribute` with `DWMWA_SYSTEMBACKDROP_TYPE` (Windows 11), (d) solid translucent colour fallback. Chosen at runtime; tested on Windows 10 and 11.
- Rounded corners: `DWMWA_WINDOW_CORNER_PREFERENCE` on Windows 11, or a clip geometry on Windows 10.
- Windows 10 22H2 is the supported minimum ([ADR-023](../../MEMORY/DECISIONS.md)): it has no system backdrop type (Mica), so Windows 10 gets acrylic through the undocumented composition attribute or the solid-colour fallback. This is a first-class path, tested every release, not an afterthought.
- Performance: large layered windows with blur can be expensive; measure in P0-06 and Phase 5.

## 10. System Events to Handle

| Event | Source | Action |
|---|---|---|
| Explorer restart | Registered message `TaskbarCreated` | `IDesktopHost.Reattach()`, sync icon state |
| Display change | `WM_DISPLAYCHANGE`, `WM_DPICHANGED` | Recompute layout |
| Sleep/resume | `PowerModeChanged` | Verify host and icon state |
| Lock/unlock, fast user switching, RDP | `SessionSwitch` | Pause hook/animation; verify after returning |
| Theme/accent/contrast change | `UserPreferenceChanged`, `WM_SETTINGCHANGE` | Update theme; turn blur off in high contrast |
| Desktop content change | `FileSystemWatcher` | Reconcile |

## 11. Spikes and Required Outcomes

| Spike | Question to answer | Output |
|---|---|---|
| P0-03 | Which host strategy meets the criteria on Win10 22H2 and Win11 23H2/24H2+, **with an Avalonia window**? | Result table + final ADR-007 |
| P0-04 | Most stable way to hide/show native icons and read their state? | Procedure + ADR-006 completion |
| P0-05 | Icon/thumbnail via `IShellItemImageFactory`: quality, speed, overlays, off-UI-thread bitmap creation in Avalonia? | Benchmark + cache decision |
| P0-06 | Transparency/blur on Avalonia windows: CPU/GPU cost and compatibility with 20 windows? | Benchmark + `IBlurEffect` |
| P0-07 | Detect desktop double-click and right-drag without disturbing the built-in menu? | Prototype + AV notes |

## 12. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| A Windows update changes the desktop structure | Dynamic discovery, hybrid strategy, manual CI per Insider/GA build, a "compatibility mode" flag |
| Icons are not restored | Guard + `state.json` + `--restore-desktop` + tray button "Restore desktop icons" |
| Hook flagged by antivirus | Minimal hook, code signing, transparent documentation, an option to disable |
| Drape covers applications | Enforce lowest z-order; test Win+D, Alt+Tab, fullscreen games, presentations |
| Keyboard focus on the desktop layer | Activate the window only on explicit interaction; hotkey to focus |
| Third-party context-menu handler crashes | Isolate in a separate thread/process with a timeout |
| OneDrive overlays and cloud attributes | Test with Files On-Demand; never trigger hydration when loading icons |
| Avalonia window cannot be re-parented or kept transparent in the host | Spikes P0-03/P0-06; strategy B; native fallback window for the host layer only |
