# 00 — UI/UX Specification

Status: draft. Numeric values are starting points; adjust after the Phase 1 prototype. Unit: DIP (device-independent pixel) unless stated.
Common patterns follow what desktop organizers customarily do; all visuals, icons and text are original ([`../PLAN/05-LEGAL-AND-NAMING.md`](../PLAN/05-LEGAL-AND-NAMING.md)). UI is Slint; it is platform-neutral and never calls native APIs ([`../ENGINEERING/00-CODING-STANDARDS.md`](../ENGINEERING/00-CODING-STANDARDS.md)).

## Anatomy of a Drape

```
┌───────────────────────────────────────┐  ← title bar (height 28)
│ Reports                          ▾  ⋯ │     title · rollup button · menu
├───────────────────────────────────────┤
│  [icon]   [icon]   [icon]   [icon]    │  ← item area (grid, padding 10)
│  label    label    label    label     │
│  [icon]   [icon]                      │
│  label    label                       │
│                                    ◢  │  ← resize grip
└───────────────────────────────────────┘
```

| Element | Initial spec |
|---|---|
| Corner radius | 8 |
| Title height | 28; 12 pt semi-bold |
| Item-area padding | 10 |
| Icon size | 48 px (choices 32/48/64/96) |
| Icon spacing | 12 |
| Label | max 2 lines, ellipsis, 11 pt |
| Background opacity | 0.45 (range 0–1) |
| Default tint | follows the system accent |
| Blur | Acrylic where available, otherwise solid translucent colour |
| Shadow | Soft all round; off in high contrast |
| Minimum size | 120 × 80 |
| Default size | 320 × 240 |

## Visual States

| State | Change |
|---|---|
| Normal | Base look |
| Hover | Title slightly lighter; rollup/menu buttons appear |
| Focus (keyboard) | 2 px focus ring on the item; contrast ≥ 3:1 |
| Selected | Item background with accent at 30% + outline |
| Drag-over | 2 px accent outline and the text "Drop to add" |
| Rolled up | Title bar only; arrow rotates 180° |
| Locked | Small padlock icon in the title; resize cursor disabled |
| Missing item | Icon dimmed to 50% + warning badge; tooltip "File not found" |
| Loading icon | Generic placeholder, replaced when the icon is ready without layout shift |

## Mouse Interaction

| Action | Result |
|---|---|
| Click item | Select |
| Ctrl/Shift + click | Multi-select |
| Double-click item | Open |
| Drag item within a Drape | Reorder (manual mode) |
| Drag item to another Drape | Move membership |
| Drag item out to empty desktop | Remove from Drape (returns to Inbox) |
| Right-click item | Shell menu + Drape commands |
| Right-click empty Drape area | Drape menu |
| Drag title | Move Drape |
| Drag edge/corner | Resize |
| Double-click title | Roll up / open |
| Double-click empty desktop | Quick-hide |
| Right-drag on empty desktop | Preview area; release → "Create Drape here" menu |
| Mouse wheel over a Drape | Scroll if items exceed the height |

## Keyboard Interaction

| Key | Result |
|---|---|
| Tab / Shift+Tab | Move through controls in the currently focused window; cross-window navigation uses the Drape chooser |
| Arrows | Move between items |
| Enter | Open the focused item |
| Delete | Remove from Drape (optional confirmation) |
| F2 | Rename the Drape (focus on title) |
| Ctrl+A | Select all items in the Drape |
| Shift+F10 / Menu key | Context menu |
| Esc | Cancel drag, close menu |

A Drape must not steal focus when shown, but must accept deliberate keyboard interaction. A keyboard-reachable notification-area menu exposes Settings and a "Focus Drape" chooser; Settings also lists Drapes with a focus action. These routes work without configuring optional global hotkeys (F-27) and remain available while Drapes are quick-hidden. Selecting a Drape focuses its title/item area through the validated host adapter; closing a chooser restores the previous focus where possible. Do not assume Tab moves between separate native child windows. The R0-07/R1-04 matrix must prove this route, including high contrast and Narrator. Native activation policy must distinguish passive show from explicit user focus.

## Menus

Drape menu: Rename · Roll up/Open · Lock/Unlock · Sort ▸ (manual, name, type, date) · Style… · Exclude from quick-hide · Delete Drape…
Item menu: the platform's native item menu + Remove from Drape · Move to Drape ▸ · Show in file manager.
Tray menu: Show/Hide Drapes · New Drape · Focus Drape ▸ · Page ▸ · Snapshot ▸ · Settings… · Restore desktop icons · Quit. Expose only implemented phase features; restore and Quit remain available when normal UI/hosting is degraded.

Delete Drape: confirm when it contains items; items return to the Inbox and **files are not deleted**. The confirmation text must say so.

## Settings Window

Tabs: General (language, autostart, theme) · Default Drape (style) · Desktop (hide icons, Inbox, drop mode) · Rules · Snapshots · Advanced (log, diagnostics, export/import, update checks) · About.
Settings apply immediately with a preview; each tab has a "Restore defaults" button. Settings for capabilities the platform lacks are shown disabled with a one-line reason.

## Theme and Accessibility

- Follow the system light/dark theme and accent colour. High-contrast support: blur and shadow off, solid outline.
- Text and icons have contrast ≥ 4.5:1 against the effective background. For a translucent background over an unknown wallpaper, offer an "Ensure contrast" option that adds a backing layer.
- Every interactive control has a supported Slint accessible role/name/state/action, verified in Windows UIA; a Drape is exposed as a group, items as list items named after the file.
- Honour the OS "show animations" setting: when off, no rollup/fade animation.
- Text size follows the OS text-size setting.
- All strings live in resources for id-ID and en-US.
- Mouse-only gestures have keyboard/menu equivalents: New Drape, move/resize, style, rollup, lock, item ordering and drag-to-other-Drape are reachable through commands. Selecting Move/Resize enters a keyboard mode with arrows, Enter to commit and Esc to cancel; the exact step sizes are UI prototype parameters, not domain coordinate changes.
- Settings and context menus expose disabled reasons to accessibility clients, retain visible focus and avoid traps. Native drag initiation and UIA interaction must be validated with the actual Slint backend, not inferred from a headless test.

## Animation

- Rollup: 150 ms, ease-out. Quick-hide fade: 150 ms. No animation at startup.
- Never block interaction waiting for an animation.

## Interface Text (examples)

| Context | en-US | id-ID |
|---|---|---|
| New Drape | "Drape 1" | "Drape 1" |
| Drop | "Drop to add" | "Lepas untuk menambahkan" |
| Delete Drape | "Delete Drape \"{name}\"? Your files will not be deleted; items will move to the Inbox." | "Hapus Drape \"{name}\"? File Anda tidak akan dihapus; item akan dipindahkan ke Inbox." |
| Missing item | "File not found. It may have been moved or deleted." | "File tidak ditemukan. Mungkin dipindahkan atau dihapus." |
| Restore icons | "Desktop icons restored." | "Ikon desktop dipulihkan." |
| Corrupt config | "Settings could not be read. We used the latest backup. The original file was kept for diagnosis." | "Pengaturan tidak dapat dibaca. Kami memakai cadangan terakhir. Berkas asli disimpan untuk diagnosa." |

## Empty States

- Empty Drape: faint text "Drag files here".
- First run: a short 3-step tour (create a Drape, drag an item, double-click the desktop), skippable, never covering the desktop.

## Imitation Policy

Imitating common function is fine; do not copy another product's distinctive look pixel for pixel, its icons, branded feature names or UI text.

## Slint implementation constraints (ADR-025)

Compile .slint components; Rust adapters connect domain state through models/properties/callbacks. No WebView or web UI engine. Shared theme tokens/components replace XAML resources. UI updates run on the Slint event loop; workers send owned data, weak UI references prevent closed-window retention.

Native compilation does not imply OS-native widgets or automatic accessibility parity. R0-07 must verify keyboard focus, Windows UIA/Narrator, high contrast, text scaling, native OLE drops and per-monitor DPI with the chosen host/backend/renderer. Native effects/drop handling stay behind capabilities and shell adapters. Blur remains planned; its MVP priority is unresolved (Q-19), with an always-available solid/translucent fallback. Avoid continuous idle animations/redraw.

Settings/About, reachable through the top-level menu/tray, exposes AboutSlint and third-party notices for the selected royalty-free licence. Validate exact attribution requirements against the pinned Slint release before distribution.
