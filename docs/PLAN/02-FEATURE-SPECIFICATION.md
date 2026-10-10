# 02 — Feature Specification

Requirements: [`01-PRODUCT-REQUIREMENTS.md`](./01-PRODUCT-REQUIREMENTS.md). Visuals: [`../UI-UX/00-UI-UX-SPEC.md`](../UI-UX/00-UI-UX-SPEC.md). Windows mechanics: [`../PLATFORM/00-WINDOWS-INTEGRATION.md`](../PLATFORM/00-WINDOWS-INTEGRATION.md). Other platforms: [`../PLATFORM/01-PLATFORM-SUPPORT.md`](../PLATFORM/01-PLATFORM-SUPPORT.md).

Release column: **MVP** (end of Phase 2), **v1** (end of Phase 5), **Later** (after v1). Every feature below is specified for Windows first; where a feature depends on a platform capability it is gated by `IPlatformCapabilities` ([ADR-015](../../MEMORY/DECISIONS.md)), never by OS checks in UI or Core.

## Feature Matrix

| ID | Feature | Summary | FR | Phase | Release |
|---|---|---|---|---|---|
| F-01 | Basic Drape | Panel on the desktop holding items | FR-01 | 1 | MVP |
| F-02 | Move / resize | Drag title, drag edges | FR-02 | 1 | MVP |
| F-03 | Items by drag-drop | From Explorer/desktop into a Drape | FR-03 | 1 | MVP |
| F-04 | Open and remove items | Double-click, Enter; Delete = remove from Drape | FR-04, FR-05 | 1 | MVP |
| F-05 | Persistence | Save/restore layout | FR-06 | 1 | MVP |
| F-06 | Hide native icons | Built-in desktop icons hidden | FR-07 | 2 | MVP |
| F-07 | Desktop folder sync | Watch add/remove/rename | FR-08 | 2 | MVP |
| F-08 | Inbox | Default Drape for ungrouped items | FR-08 | 2 | MVP |
| F-09 | Quick-hide | Double-click empty desktop | FR-10 | 2 | MVP |
| F-10 | Create Drape by right-drag | Right-button drag on the desktop | FR-01 | 2 | MVP |
| F-11 | Emergency recovery | `--restore-desktop`, Guard | FR-23 | 2 | MVP |
| F-12 | Autostart | Run at login | FR-22 | 2 | MVP |
| F-13 | Multi-monitor / DPI | Per-monitor DPI; restore on display change | FR-21 | 1–2 | MVP |
| F-14 | Rollup | Collapse a Drape to its title bar | FR-09 | 3 | v1 |
| F-15 | Style | Tint, opacity, blur, icons, font | FR-11 | 3 | v1 |
| F-16 | Snap | Magnet to edges and other Drapes | FR-12 | 3 | v1 |
| F-17 | Lock | Prevent move/resize | FR-13 | 3 | v1 |
| F-18 | Shell context menu | The platform's own item menu | FR-14 | 3 | v1 |
| F-19 | Item ordering | Manual, name, type, date | — | 3 | v1 |
| F-20 | Folder portal | A Drape that shows a folder's contents | FR-15 | 4 | v1 |
| F-21 | Auto-sort rules | Extension / name pattern / age | FR-16 | 4 | v1 |
| F-22 | Desktop pages | Several pages of Drapes | FR-17 | 4 | v1 |
| F-23 | Snapshots | Save/restore a layout | FR-18 | 4 | v1 |
| F-24 | Tabs in a Drape | Several groups per Drape | FR-19 | 4 | Later |
| F-25 | Search | Find items inside a Drape | FR-20 | 4 | Later |
| F-26 | Export / import | Config backup | FR-24 | 4 | v1 |
| F-27 | Global hotkeys | Toggle show/hide | FR-25 | 4 | Later |

## Detailed Behaviour

### F-01 Basic Drape
- A Drape is a borderless, semi-transparent window on the desktop layer (below application windows, above the wallpaper).
- Has a title (editable), an item area (grid) and a resize grip.
- Default names: "Drape 1", "Drape 2", … ; unique within a page.
- Minimum size 120 × 80 DIP. Default size 320 × 240 DIP.
- Does not appear in the taskbar or Alt+Tab (or the platform equivalent).
- One Avalonia window per Drape ([ADR-007](../../MEMORY/DECISIONS.md)).

### F-02 Move and resize
- Move: drag the title. Resize: drag an edge or corner. All coordinates are in DIP, stored per monitor.
- A Drape may never leave the monitor work area entirely; at least 40 DIP stay visible.
- While moving, show snap guides if snap is enabled (F-16).

### F-03 Items by drag-drop
- Accept dropped files/folders/shortcuts from the file manager and from the desktop (Windows: `CF_HDROP` and Shell IDList).
- Items from the Desktop folder are references only; the file stays where it is.
- Items from outside the Desktop are added as references (default mode). A "copy/move to Desktop" option exists in settings and **always asks for confirmation**.
- Duplicates within one Drape are rejected.
- **An item belongs to exactly one Drape** (v1, [ADR-011](../../MEMORY/DECISIONS.md)): dropping an item that already lives in another Drape *moves* its membership to the target Drape.
- Dropping a Drape item onto empty desktop removes it from the Drape and returns it to the Inbox.

### F-04 Open and remove items
- Double-click or Enter: open with the default handler (Windows: `ShellExecute`).
- Delete or the "Remove from Drape" menu: removes the reference, **not** the file.
- "Delete file" exists only in the shell context menu (F-18), using the platform's own confirmation and trash/Recycle Bin.
- An item whose file is gone is shown dimmed with a warning badge and is kept for 24 hours before the reference is dropped.

### F-05 Persistence
- Changes are saved with a 500 ms debounce. Format and schema: [`../ARCHITECTURE/02-DATA-MODEL.md`](../ARCHITECTURE/02-DATA-MODEL.md).
- At startup: load config, migrate if needed, reconcile with the Desktop contents, show Drapes.
- Corrupt config: use the newest valid backup, tell the user once, keep the corrupt file for diagnosis.

### F-06 Hide native icons
- While Drape is active and "Hide desktop icons" is on, the native icons are hidden.
- On normal exit, crash or kill, icons are restored (F-11).
- If the user toggles "Show desktop icons" through the OS menu, Drape detects it and adapts rather than fighting.
- Gated by `IPlatformCapabilities.CanHideNativeIcons`.

### F-07 Desktop folder sync
- Watches the user's Desktop and the shared/public Desktop. 250 ms debounce, processed in batches.
- New file: goes to the Inbox (or to the Drape chosen by a rule, F-21).
- Deleted file: reference removed after the grace period. Renamed file: followed via NTFS file ID where available.
- Ignores `desktop.ini`, system files and hidden files, unless the Explorer "show hidden files" setting is on.

### F-08 Inbox
- One special Drape per page holding Desktop items not yet grouped. It can be renamed and cannot be deleted while Desktop sync is on.
- If disabled, ungrouped items appear in an automatically created "Other" Drape.

### F-09 Quick-hide
- Double-clicking empty desktop shows/hides all Drapes (and any remaining native icons).
- "Empty" = no native item and no Drape at the click point, determined by hit-test, never by guessing coordinates.
- A Drape can be excluded from quick-hide in its settings.
- 150 ms fade; honours the OS "reduce animations" setting.
- Gated by `IPlatformCapabilities.HasGlobalMouseHook`.

### F-10 Create a Drape by right-drag
- Press and drag the right button on empty desktop; a preview rectangle appears; on release a "Create Drape here" menu is shown.
- A plain right-click (no drag) still shows the OS context menu. Drape must never break that menu.

### F-11 Emergency recovery
- `Drape.App.Desktop --restore-desktop`: restores icons and exits. Does not require the main app to be running.
- `Drape.Guard`: a tiny process that waits for the main process; however it ends, Guard restores the icons.
- `state.json` records that Drape is hiding icons, for recovery after a hard reboot.

### F-12 Autostart
- A setting. Mechanism per platform behind `IAutostart` (Windows: HKCU `Run` key — non-elevated; never an elevated scheduled task).

### F-13 Multi-monitor and DPI
- Per-Monitor DPI Awareness v2 on Windows. Positions are stored relative to a monitor (device ID + work area), not raw virtual-screen coordinates.
- Monitor disappears: its Drapes move to the primary monitor, scaled proportionally, and remember their original position. Monitor returns with the same ID: they go back.
- Resolution changes: scale proportionally from the **stored** position, never from the current one, to avoid drift.

### F-14 Rollup
- Double-click the title to roll a Drape up to a single title bar. Options: auto-roll, and open on hover (after 300 ms).
- The pre-rollup height is stored.

### F-15 Style
- Per Drape and as a global default: tint colour, opacity (0–100%), blur (off/acrylic), corner radius, title font colour and size, icon size (32/48/64/96 px), icon spacing, label mode (one/two lines, hidden).
- Theme: follow system, light, dark. High contrast follows the system theme and disables blur.

### F-16 Snap
- Magnet to monitor edges, other Drapes and an optional grid; threshold 8 DIP. Hold Alt to disable temporarily.

### F-17 Lock
- A locked Drape cannot be moved or resized, but items can still be opened. Adding items is optional via a setting.

### F-18 Shell context menu
- Right-clicking an item shows the platform's native item menu (Windows: `IContextMenu`) plus Drape commands ("Remove from Drape", "Move to Drape ▸").
- If a third-party handler is slow or fails, show a minimal menu and log it.

### F-19 Item ordering
- Manual (drag to reorder), name, type, modified date. "Folders first" is optional.

### F-20 Folder portal
- A Drape linked to a folder; its contents are mirrored and kept up to date. Portal items are not stored in config — only the folder path and options (recursive, filter).
- Dropping onto a portal copies or moves the file into the folder, after confirmation.

### F-21 Auto-sort rules
- Rule: condition (extension, name pattern glob/regex, file age, size, type) → action (put into Drape X). Priority is orderable. Rules run on new items; "Apply now" runs them on existing items.
- Items moved manually by the user are not re-sorted (`pinnedByUser` flag).

### F-22 Desktop pages
- Several pages; one active. Switch via screen edge, hotkey or tray menu. A Drape can be "pinned" to all pages.

### F-23 Snapshots
- Saved with a name and time; contents are position, size, style and membership. Restore with a preview and confirmation.

### F-24 Tabs in a Drape
- A Drape holds several tabs; each tab is an item list or a portal. Drag an item onto a tab title to move it.

### F-25 Search
- Typing while a Drape has focus filters its items by name.

### F-26 Export / import
- Export a single `.drape.json` file (absolute paths anonymised when the "anonymise" option is on). Import merges or replaces, with a preview and a backup.

### F-27 Global hotkeys
- Default: none. The user picks combinations for show/hide and page switching. Registered through the OS hotkey API (Windows: `RegisterHotKey`) — never a keyboard hook.

## Out of Scope for v1

Virtual items (This PC, Recycle Bin, Network), widgets/clock/weather, cloud sync, third-party plugins, community-shared themes, full ARM64 support, and any non-Windows platform (see Phase 6 in [`03-IMPLEMENTATION-ROADMAP.md`](./03-IMPLEMENTATION-ROADMAP.md)).
