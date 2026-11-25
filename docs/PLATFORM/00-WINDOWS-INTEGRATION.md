# 00 — Windows Integration (Win32)

The highest-risk document. Many techniques here are not officially documented and can change between Windows builds.
**Every claim below must be verified by a spike (R0-04 … R0-08) on the target Windows build before it becomes a foundation.** Results are recorded in `MEMORY/records/` and, when they decide something, as ADRs.

Scope: Windows only; all code lives in `drape-shell-windows` (and the icon-restore part of `drape-guard`). UI code is Slint and never calls these APIs directly ([`../ENGINEERING/00-CODING-STANDARDS.md`](../ENGINEERING/00-CODING-STANDARDS.md)).

## 1. Desktop Window Hierarchy (overview)

Explorer generally draws the desktop through a set of windows such as:

```
Progman ("Program Manager")
 └─ SHELLDLL_DefView
     └─ SysListView32  ("FolderView")   ← desktop icons
WorkerW (one or more)                    ← used for wallpaper/slideshow and effects
```

The real arrangement varies: `SHELLDLL_DefView` may sit under `Progman` or under one of the `WorkerW` windows, and this changes between Windows 10/11 builds (including 24H2).
Never hard-code a window path or treat the drawing above as a discovery algorithm. Enumerate and inspect candidate Explorer windows and descendants on the current interactive desktop/session; validate their ownership, classes, relationships and observed behaviour. A top-level `EnumWindows` scan alone can miss a changed/nested arrangement. If discovery is ambiguous or Explorer is unavailable, do not guess a Progman parent or hide native icons. All handles are transient and must be rediscovered after shell recreation.

Tools: Spy++ and `Inspect.exe` to examine the hierarchy on the target machine.

## 2. Placing Drape on the Desktop Layer

Goal: Drapes appear above the wallpaper (replacing the icons), below all application windows, stay visible after Win+D, and do not appear in the taskbar or Alt+Tab.

Each Drape is a Slint window. R0-04 must verify the pinned backend's supported public native-handle API, materialisation timing and borrowed HWND lifetime. Request frameless/translucent presentation using verified Slint APIs; shell style/parent/z-order changes belong to drape-shell-windows and must remain compatible with the renderer. Detach/unregister before window destruction. Do not mechanically translate Avalonia handle or transparency-level APIs.

| Strategy | How | Pros | Risks |
|---|---|---|---|
| A. Parent to a verified desktop host | Evaluate `SetParent` with a discovered `Progman`/`WorkerW`; undocumented `0x052C` is an experiment, not a required startup command | Candidate for natural desktop z-order and Win+D survival | Unproved with the selected Slint backend; cross-process parenting, window styles, DPI contexts and transparency can fail |
| B. Bottom-most top-level window | Evaluate `WS_EX_TOOLWINDOW`, `SetWindowPos(HWND_BOTTOM, SWP_NOACTIVATE)` and bounded z-order maintenance | Avoids desktop parenting | Win+D/peek may hide it; re-showing can flicker or steal focus. Degraded unless it meets every host criterion |
| C. Hybrid | Try an accepted A path, otherwise enter a labelled B degraded mode | Offers an explicit fallback | Two test paths and more lifecycle complexity; robustness is not established before measurement |

Evaluate these alternatives behind `DesktopHost`; no strategy is accepted merely by appearing in this table. R0-04/R0-10 record the selected strategy and limitations in an ADR. If the fallback is not a safe desktop host, native icons remain visible.

A strategy passes only if:
- It is visible above the wallpaper and does not cover application windows.
- It stays visible after Win+D and Win+, (peek).
- It does not steal focus when it appears.
- It recovers after an Explorer restart.
- It works at 100%, 150%, 200% DPI and with 2 monitors at different DPI.
- Transparency stays correct; blur is tested separately when enabled. Blur's MVP priority remains Q-19, not a prerequisite silently imposed by the host spike.

Perform handle acquisition, window-style changes, native attachment, OLE registration and destruction on the owning UI/message-loop thread unless the verified backend explicitly permits otherwise. Preserve/rollback changed styles and parent state on partial failure. `IsWindow` alone cannot establish that a reused HWND still identifies the registered window. Cross-process parenting needs explicit DPI-awareness/error checks; convert screen, parent-client and monitor-logical coordinates through documented APIs. Test negative-coordinate monitor layouts and explicit keyboard focus rather than permanently disabling activation.

## 3. Hiding Native Icons

Candidate approach: send `WM_COMMAND` with `wParam = 0x7402` to a verified `SHELLDLL_DefView`. This undocumented command is believed to match "Show desktop icons" and is a **toggle**, not a setter. R0-05 must prove both the command and the state-reading method. `IsWindowVisible` alone can reflect ancestor visibility or an unavailable desktop and must not be treated as authoritative user intent. Represent uncertain state explicitly; do not toggle on unknown state.

Rules:
- Always read state before and after sending the command; record it in `state.json`.
- If the user changes it through the Windows menu, detect it and do not fight it.
- Restore on every exit path ([`../ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](../ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md)): normal, exception, Guard, `--restore-desktop`, next start.
- Alternatives to evaluate in spike R0-05: hiding `SysListView32` with `ShowWindow(SW_HIDE)` (risk: Explorer shows it again), or the registry setting `HideIcons` plus a broadcast (risk: persistently changes user settings — avoid unless truly necessary).
- Never kill `explorer.exe` to hide icons.

Serialise App, Guard and standalone-restore operations using a per-user/session recovery coordinator. Two readers followed by two toggles are not idempotent; cross-process ownership/generation checks and read-back are required. Bound cross-process messages (for example validated `SendMessageTimeoutW` use) so hung Explorer cannot freeze shutdown. Explorer absence is retryable, not proof that recovery succeeded.

### Recovery transaction and Guard health

R0-05 must specify and test each crash boundary: observe original state → durably persist a versioned ownership/intent record → arm Guard with a live process handle and matching transaction/session identity → receive readiness → recheck Guard/host/state → hide and verify → record active state. A crash after intent but before hide must not blindly flip an untouched desktop. A user visibility change supersedes Drape's prior ownership; do not later restore over that choice. Where intent cannot be established safely, do not issue a new hide or a blind show/toggle; preserve the observed state, report the uncertainty and offer explicit manual restoration.

App must observe Guard exit/loss of readiness while it remains alive. Restore owned icon changes immediately and disable further hides until a replacement Guard is armed; no retry loop may continue hiding without protection. Readiness is not a lifetime guarantee, so test Guard death immediately before/after hide, App death during rearming, both-process loss, and Explorer loss during restoration. App/Guard processes can both fail; next-start and standalone recovery remain necessary. Define bounded retries and retain pending state until verified recovery, without pretending that a panic hook or watchdog eliminates all failure windows.

The standalone `--restore-desktop` path bypasses Slint/rendering and normal settings activation, but still coordinates with a running App/Guard so it cannot immediately rehide icons or clear another generation's state. Exit/report status must distinguish restored/already safe from pending/failed. Malformed, newer-version or legacy recovery records must never trigger a blind toggle or unauthorised process control; preserve evidence and expose safe manual recovery instructions.

## 4. Enumeration and Item Icons

- Item source: the file system under the Desktop and the Public Desktop (Known Folder API: `SHGetKnownFolderPath`), exposed to Core as `DesktopLocations`. Ignore `desktop.ini` and hidden/system attributes according to the Explorer settings.
- Icons and thumbnails: evaluate `IShellItemImageFactory::GetImage` and its flags on bounded, apartment-correct workers. Return owned, validated neutral pixels (dimensions/stride/format/alpha semantics), then create UI images on the Slint event loop. Explicitly release HBITMAP/HICON/PIDL/COM/Known Folder allocations with their matching APIs. Bound cache bytes and queued requests; cache keys account for identity, requested scale/size and invalidation.
- Overlays (shortcut arrow, cloud/OneDrive status) may be lost with this approach; evaluate `SHGetFileInfo` / `IShellIconOverlay` in spike R0-06. Reading icons must not trigger OneDrive hydration.
- Virtual items (This PC, Recycle Bin, Network): deferred beyond the MVP; they need the Desktop `IShellFolder` and PIDLs, not file paths.

## 5. Shell Context Menu

- Obtain `IContextMenu` from `IShellFolder::GetUIObjectOf` for the item's PIDL, display it with `TrackPopupMenuEx`, and forward menu messages to `IContextMenu2/3` so submenus (Send to, Open with) work.
- Explicitly confirm destructive verbs before side effects; neither Windows nor a third-party extension is assumed to prompt or use Recycle Bin. P3-06 must classify/restrict invocation sufficiently to uphold reference-only membership and user confirmation; unreviewed opaque destructive verbs are not an exception to the shared contract.
- Third-party handlers can block or crash. A dedicated STA/message loop and a bounded request policy protect responsiveness, not process integrity. A timeout does not cancel the native call or permit killing/unloading its thread/COM objects; prevent abandoned-request/resource growth. Return a minimal menu only when lifecycle/reentrancy remains safe. Out-of-process containment is a separate design/ADR if required, not an implied feature.
- Needs an owner window that receives `WM_INITMENUPOPUP`, `WM_MEASUREITEM`, `WM_DRAWITEM`, `WM_MENUCHAR`. Use a small hidden native owner window in the shell adapter if required; prove message routing and apartment behaviour with Slint in R0-07.

## 6. Drag and Drop

- Receive: `CF_HDROP` and `CFSTR_SHELLIDLIST`. R0-07 verifies the pinned Slint/backend drop support. Use a shell-owned native OLE `IDropTarget` adapter where required; register/revoke it for the borrowed window lifetime, initialise COM correctly and bridge neutral item data to UI. Never assume `DragDrop`/`DataFormats.Files` are Slint APIs.
- Send (dragging an item out): build an `IDataObject` with `CF_HDROP` so Explorer and other apps can accept it.
- Native OLE registration and `DoDragDrop` belong to the owning STA with message pumping. Register/revoke once per live HWND; handle existing toolkit registrations rather than replacing them blindly. Bound and validate incoming data sizes, counts and formats before creating owned item references; reject unsupported virtual payloads. Do not transfer pointers/COM interfaces across threads without valid marshaling.
- Negotiate drop effects explicitly: reference membership never requests physical MOVE/DELETE semantics. Opening or moving real files remains an explicit user action through the approved platform port, not a side effect of dropping an item into a Drape.
- **UIPI:** a non-elevated app cannot receive drops from an elevated process. Explain it in user documentation; never raise privileges as a workaround.

## 7. Mouse Hook for Quick-Hide and Right-Drag

- Install `WH_MOUSE_LL` on a dedicated message-loop thread. Its callback receives button/move events, not a reliable preclassified double-click event. Copy only the bounded transient fields needed for hit-testing/gesture detection, try a non-blocking enqueue, and call `CallNextHookEx` as required by the callback contract (including negative hook codes).
- Derive double-click timing/position and right-drag state outside the callback using system settings such as `GetDoubleClickTime()` and the double-click rectangle dimensions. Cancel incomplete gestures on overflow, timeout, session change, host loss or disable. Temporary coordinates/time exist only in bounded in-memory gesture state and are discarded when the decision completes; never log/persist input history.
- Empty-desktop hit-testing also happens outside the callback against a currently validated desktop target. `LVM_HITTEST` uses a pointer-bearing structure and cannot send a local pointer into Explorer. Any cross-process buffer method requires an explicit reviewed allocation/read/free/bitness/timeout design; evaluate bounded UIA or another safer method in R0-08. Unknown/timeout/inaccessible targets produce no gesture, not an assumed empty desktop.
- The hook must not log or store coordinates/clicks beyond what the feature needs ([`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md)).
- Alternatives without a global hook: subclassing the desktop window (impossible cross-process), or a full-screen transparent window that catches clicks (blocks native interaction; rejected).
- Hook installation failure is non-fatal: quick-hide and right-drag are disabled with a message in settings, via `PlatformCapabilities.HasGlobalMouseHook`.

## 8. DPI and Monitors

- Manifest: `PerMonitorV2`. Use `WM_DPICHANGED` and `GetDpiForWindow`. Slint scales its own content; Shell code works in physical pixels and converts at the `DisplayMonitor` boundary.
- Monitor list: `EnumDisplayMonitors`/`GetMonitorInfo` plus validated display-topology metadata. Device names and enumeration indexes are not guaranteed persistent identity; R1 layout design defines reconnect matching and a safe fallback for missing/ambiguous monitors. Do not claim IDs are stable before tests across docking, driver changes and RDP.
- Change messages: `WM_DISPLAYCHANGE`, `WM_SETTINGCHANGE` (work area), `WM_DPICHANGED`. Debounce 500 ms, because Windows sends several in a row.
- Position storage: see [`../ARCHITECTURE/02-DATA-MODEL.md`](../ARCHITECTURE/02-DATA-MODEL.md) (`Bounds`). Always recompute from the stored values, never from the current position.

## 9. Visual Effects

- Transparent background: an undecorated Slint window with a transparent background, using the pinned backend/renderer verified by R0-04/R0-07. Per-pixel transparency is the simple path; native blur needs the platform backdrop APIs.
- Blur/acrylic: wrapped in `BlurEffect` with implementations (a) a verified Slint/backend facility if one exists, (b) `SetWindowCompositionAttribute` (undocumented), (c) `DwmSetWindowAttribute` with `DWMWA_SYSTEMBACKDROP_TYPE` (Windows 11), (d) solid translucent colour fallback. Chosen at runtime; tested on Windows 10 and 11.
- Rounded corners: `DWMWA_WINDOW_CORNER_PREFERENCE` on Windows 11, or a clip geometry on Windows 10.
- Windows 10 22H2 is the supported minimum ([ADR-023](../../MEMORY/DECISIONS.md)): it has no system backdrop type (Mica), so Windows 10 gets acrylic through the undocumented composition attribute or the solid-colour fallback. This is a first-class path, tested every release, not an afterthought.
- Performance: large layered windows with blur can be expensive; measure in R0-07 and Phase 5.

## 10. System Events to Handle

| Event | Source | Action |
|---|---|---|
| Explorer restart | Registered message `TaskbarCreated` | `DesktopHost.Reattach()`, sync icon state |
| Display change | `WM_DISPLAYCHANGE`, `WM_DPICHANGED` | Recompute layout |
| Sleep/resume | Native power notifications, e.g. `WM_POWERBROADCAST`, with registration as needed | Verify host, Guard and icon state |
| Lock/unlock, fast user switching, RDP | Native WTS session notifications, e.g. registered `WM_WTSSESSION_CHANGE` | Pause/cancel transient gestures; verify same-session host/Guard/state after return |
| Theme/accent/contrast change | native theme/contrast notifications, including `WM_SETTINGCHANGE` | Update theme; turn blur off in high contrast |
| Desktop content change | `notify` adapter with overflow rescan | Reconcile |

## 11. Spikes and Required Outcomes

| Spike | Question to answer | Output |
|---|---|---|
| R0-04 | Which host strategy meets the criteria on Win10 22H2 and Win11 23H2/24H2+, **with a Slint window**? | Result table + final ADR-007 |
| R0-05 | Most stable way to hide/show native icons and read their state? | Procedure + ADR-006 completion |
| R0-06 | Icon/thumbnail via `IShellItemImageFactory`: quality, speed, overlays, owned pixel extraction on apartment-correct workers and UI-thread Slint image conversion? | Benchmark + cache decision |
| R0-07 | Transparency/blur on Slint windows: CPU/GPU cost and compatibility with 20 windows? | Benchmark + `BlurEffect` |
| R0-08 | Detect desktop double-click and right-drag without disturbing the built-in menu? | Prototype + AV notes |

## 12. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| A Windows update changes the desktop structure | Dynamic discovery, hybrid strategy, manual CI per Insider/GA build, a "compatibility mode" flag |
| Icons are not restored | Guard + `state.json` + `--restore-desktop` + tray button "Restore desktop icons" |
| Hook flagged by antivirus | Minimal hook, code signing, transparent documentation, an option to disable |
| Drape covers applications | Enforce lowest z-order; test Win+D, Alt+Tab, fullscreen games, presentations |
| Keyboard focus on the desktop layer | Explicit keyboard-accessible tray/Settings chooser to focus a Drape; optional global hotkey is additional, not required |
| Third-party context-menu handler blocks/crashes | Bounded apartment-correct work protects responsiveness only; thread timeout is not crash containment. Guard remains armed; process isolation requires its own design |
| OneDrive overlays and cloud attributes | Test with Files On-Demand; never trigger hydration when loading icons |
| Slint window cannot be re-parented or kept transparent in the host | Spikes R0-04/R0-07; strategy B; native fallback window for the host layer only |

## Rust/Slint acceptance additions (ADR-025)

Use the Microsoft windows crate with minimal features. RAII must match each HWND/HICON/HBITMAP/PIDL/COM allocation to its correct releaser; Slint-owned HWNDs are borrowed. Never unwind over FFI. Worker threads initialise COM with the correct apartment; timing out a thread does not make COM cancellation or unloading safe. In-process native/third-party handler crashes are not isolated by moving them to another thread; Guard is still required.

No hard-coded Progman fallback after discovery failure, no assumption that HWND_BOTTOM is Win+D-proof, and no unconditional WS_EX_NOACTIVATE that prevents the required explicit keyboard interaction. Test actual Slint windows, native drops, high contrast/UIA and mixed-DPI input/rendering. Preserve hit-testing/coordinate conversions after reparenting; do not assume virtual-screen origin equals parent-client origin.

Guard readiness and durable intent precede every hide; failed Guard startup leaves native icons visible. Restore before disarming Guard, including when logging/config save fails. Panic hooks/Drop cover only best-effort controlled paths; independent Guard/next-start/CLI handle abort, forced kill and reboot. Slint-specific results from R0-04–R0-10 are mandatory; old .NET runbooks are historical references.

## Primary API contracts for future verification

These links identify the contracts each implementation spike must check; listing them is not runtime evidence or a claim that all backend combinations are supported.

- [SetParent](https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setparent): cross-process parenting, window styles and DPI-awareness caveats.
- [SendMessageTimeoutW](https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw): bounded cross-process messaging and error semantics.
- [LowLevelMouseProc](https://learn.microsoft.com/windows/win32/winmsg/lowlevelmouseproc): hook-code forwarding, permitted messages, timing and callback rules.
- [RegisterDragDrop](https://learn.microsoft.com/windows/win32/api/ole2/nf-ole2-registerdragdrop): OLE initialisation, message-loop and registration lifetime.
- [WTSRegisterSessionNotification](https://learn.microsoft.com/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification): native session registration and notifications.
- [Slint documentation](https://docs.slint.dev): pin and verify the actual window/backend, event-loop and renderer APIs before design acceptance.
