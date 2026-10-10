# 01 — Product Requirements (PRD)

Status: draft, 2026-10-09; revised 2026-10-10 (English, cross-platform scope). Priorities use MoSCoW: **M**ust, **S**hould, **C**ould, **W**on't (this version).

## Personas

- **Rani, analyst in an office.** 80+ files on the desktop. Wants groups such as "Reports", "Apps", "Temporary". Presents often; needs a clean desktop instantly.
- **Dimas, developer.** Many shortcuts and project folders. Wants Drapes that collect files automatically by extension and folder.
- **Sinta, laptop + external monitor.** Layout breaks every time she docks or undocks. Wants Drape positions to restore themselves.

## User Stories

| ID | As a | I want to | So that |
|---|---|---|---|
| US-01 | user | create a new Drape where I choose | I can group icons |
| US-02 | user | drag icons from the desktop or Explorer into a Drape | adding items is fast |
| US-03 | user | move and resize a Drape | I can lay out my desktop as I like |
| US-04 | user | open an item by double-clicking | a Drape works like the normal desktop |
| US-05 | user | remove an item from a Drape without deleting the file | I never fear losing files |
| US-06 | user | roll a Drape up until only its title shows | I save space |
| US-07 | user | double-click empty desktop to hide/show everything | my desktop is clean when presenting |
| US-08 | user | have new desktop files land in a specific Drape | the desktop stays tidy automatically |
| US-09 | user | change a Drape's colour, transparency and icon size | it matches my taste |
| US-10 | user | link a Drape to a folder (portal) | I see a folder's contents without opening it |
| US-11 | user | create auto-sort rules | files reach the right Drape on their own |
| US-12 | user | save and restore layout snapshots | I can return to a favourite arrangement |
| US-13 | multi-monitor user | have my layout restored after a resolution/monitor change | I don't re-arrange every time |
| US-14 | user | close the app and have my desktop return to normal | I feel safe |
| US-15 | user | navigate Drapes with the keyboard alone | the app is accessible |
| US-16 | user | have several desktop pages | I can separate work contexts |

## Functional Requirements

| ID | Requirement | Priority | Phase |
|---|---|---|---|
| FR-01 | Create, rename and delete a Drape | M | 1 |
| FR-02 | Move and resize a Drape with the mouse; position persists | M | 1 |
| FR-03 | Add items by drag-drop from Explorer/desktop | M | 1 |
| FR-04 | Open an item (double-click, Enter) | M | 1 |
| FR-05 | Remove an item from a Drape without deleting the file | M | 1 |
| FR-06 | Save and restore layout at startup | M | 1 |
| FR-07 | Hide native desktop icons while active; restore on exit | M | 2 |
| FR-08 | Sync with the Desktop folder (add/remove/rename) | M | 2 |
| FR-21 | Multi-monitor and per-monitor DPI | M | 1–2 |
| FR-23 | Emergency recovery: `--restore-desktop` and Guard | M | 2 |
| FR-09 | Rollup | S | 3 |
| FR-10 | Quick-hide by double-clicking the desktop | S | 2 |
| FR-11 | Style: tint, opacity, blur, icon size, font, spacing | S | 3 |
| FR-12 | Snap to screen edges and other Drapes | S | 3 |
| FR-13 | Lock a Drape (no move/resize) | S | 3 |
| FR-14 | Shell context menu for items (Open with, Properties, …) | S | 3 |
| FR-15 | Folder portal | S | 4 |
| FR-16 | Auto-sort rules | S | 4 |
| FR-22 | Start automatically at login | S | 2 |
| FR-17 | Desktop pages | C | 4 |
| FR-18 | Snapshot and restore layout | C | 4 |
| FR-19 | Tabs within a Drape | C | 4 |
| FR-20 | Search within a Drape | C | 4 |
| FR-24 | Export/import configuration | C | 4 |
| FR-25 | Global hotkeys | C | 4 |
| FR-26 | macOS, Linux, Android, iOS support | W (v1) | 6 |
| FR-27 | Virtual items (This PC, Recycle Bin, Network) | W (v1) | — |
| FR-28 | Cloud sync, plugins, widgets | W (v1) | — |

Behaviour of each feature: [`02-FEATURE-SPECIFICATION.md`](./02-FEATURE-SPECIFICATION.md).

## Non-Functional Requirements

| ID | Requirement | Target / how measured |
|---|---|---|
| NFR-01 | Idle CPU | Average < 0.5% over 10 idle minutes (Performance Monitor) |
| NFR-02 | Memory | < 100 MB working set with 20 Drapes × 20 items |
| NFR-03 | Startup | Drapes visible < 1.5 s after process start (SSD) |
| NFR-04 | Data durability | No config loss on crash/power loss; rolling backups |
| NFR-05 | Privacy | No telemetry and no network by default; update checks are opt-in ([ADR-017](../../MEMORY/DECISIONS.md)) |
| NFR-06 | Accessibility | Full keyboard navigation, high contrast, UIA names for controls |
| NFR-07 | Compatibility | v1: **minimum Windows 10 22H2 (build 19045), fully supported** ([ADR-023](../../MEMORY/DECISIONS.md)); Windows 11 is the primary development target; x64 only; x64 only; DPI 100–300%; 1–4 monitors. Later: see [`../PLATFORM/01-PLATFORM-SUPPORT.md`](../PLATFORM/01-PLATFORM-SUPPORT.md) |
| NFR-08 | Localisation | id-ID and en-US from v1; strings in resources, never hard-coded |
| NFR-09 | Privileges | Runs non-elevated |
| NFR-10 | Session resilience | Survives Explorer restart, sleep/resume, lock/unlock, display change, RDP |
| NFR-11 | Security | See [`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md) |
| NFR-12 | Quality | Warning-free build, green tests, Core coverage ≥ 80% |
| NFR-13 | Portability | `Drape.Core`, `Drape.Infrastructure` and `Drape.UI` contain no platform-specific code; verified by architecture tests |

## Constraints and Assumptions

- In the default mode Drape does not move physical files. Membership is stored in config.
- The Desktop folder may be redirected (OneDrive Known Folder Move); always obtain its location from the platform API, never from a hard-coded path.
- Virtual items (This PC, Recycle Bin, Network) are deferred; the MVP handles real files, folders and shortcuts only.
- An item belongs to exactly one Drape in v1 ([ADR-011](../../MEMORY/DECISIONS.md)).
- Stack: C# / .NET / Avalonia UI — confirmed by the owner ([ADR-012](../../MEMORY/DECISIONS.md)).

## Product-Level Acceptance Criteria (MVP, end of Phase 2)

1. A user can create 3 Drapes, fill them with desktop items, close the app, reopen it, and find everything unchanged.
2. Native desktop icons are hidden while the app runs and restored when the app is closed normally, killed, or crashes.
3. A file newly created on the desktop appears in the default Drape in < 2 s.
4. Layout survives a resolution change and plugging/unplugging a monitor.
5. Drapes stay visible after Win+D and after Explorer is restarted.
