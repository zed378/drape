# 06 — Glossary

## Product Terms

| Term | Meaning |
|---|---|
| **Drape** | The product name **and** one panel: a semi-transparent window on the desktop that holds items. The same word is used in UI and code (`Drape`, `DrapeItem`, `DrapeManager`, `DrapeWindow`, `DrapeViewModel`; [ADR-020](../../MEMORY/DECISIONS.md)) |
| **Item** | A reference to a file, folder or shortcut shown inside a Drape |
| **Inbox** | The special Drape for Desktop items not yet grouped |
| **Rollup** | Collapsing a Drape until only its title bar is visible |
| **Quick-hide** | Showing/hiding every Drape by double-clicking empty desktop |
| **Portal** | A Drape that mirrors the contents of a folder |
| **Rule** | Condition → action that places new items in a specific Drape |
| **Page** | A set of Drapes; a user can have several desktop pages |
| **Snapshot** | A saved copy of the Drape layout (position, style, membership) |
| **Tab** | A sub-group of items inside one Drape |
| **Guard** | `Drape.Guard`, the tiny process that restores desktop icons if the main process ends |
| **Reference mode** | Drape stores only references; files are never moved |
| **Capability** | A feature a platform can or cannot provide, reported by `IPlatformCapabilities` (e.g. desktop layer, hide native icons, global mouse hook) |
| **Desktop layer** | The z-order band between the wallpaper and application windows where Drapes live |

## Technical Terms

| Term | Meaning |
|---|---|
| **Avalonia UI** | Cross-platform .NET UI framework (XAML, MIT licence) used for all Drape windows |
| **Shell project** | A `Drape.Shell.<Platform>` project: the only place native/platform APIs are called |
| **Composition root** | The app project (`Drape.App.Desktop`) that wires interfaces to implementations through DI |
| **DIP** | Device-independent pixel; 1 DIP = 1/96 inch at 100% scale |
| **Per-Monitor DPI v2** | Windows DPI-awareness mode that notifies the app when a window's DPI changes |
| **Progman** | The "Program Manager" window; the parent of the Explorer desktop |
| **WorkerW** | Explorer worker window used for wallpaper/desktop effects |
| **SHELLDLL_DefView** | The shell folder-view window that hosts the desktop icon list |
| **SysListView32** | The list-view control that draws the desktop icons |
| **PIDL** | Pointer to an ID list; item identity in the Windows shell namespace |
| **Known Folder** | A registered Windows folder (Desktop, Public Desktop, …) with a GUID |
| **UIPI** | User Interface Privilege Isolation; blocks lower-privilege processes from sending messages/drops to higher-privilege ones |
| **UIA** | UI Automation; the Windows accessibility API |
| **CsWin32** | Source generator that produces P/Invoke bindings from Win32 metadata |
| **WH_MOUSE_LL** | The global low-level mouse hook (Windows) |
| **CGEvent tap** | macOS counterpart of a global input hook; needs user-granted permission |
| **wlr-layer-shell** | Wayland protocol for desktop-layer surfaces; not available on stock GNOME |
| **HRESULT** | COM/Win32 result code |
| **Explorer restart** | `explorer.exe` restarting; desktop windows are recreated |
| **KFM** | OneDrive Known Folder Move; redirects Desktop into OneDrive |
| **ADR** | Architecture Decision Record, kept in [`MEMORY/DECISIONS.md`](../../MEMORY/DECISIONS.md) |
| **Spike** | A short experiment that answers one technical question; its outcome becomes an ADR and a record |

## Internal Abbreviations

| Abbreviation | Meaning |
|---|---|
| FR / NFR | Functional / Non-functional requirement |
| PR-n | Project risk (risk register) |
| Q-n | Open question (Backlog) |
| P*n*-*nn* | Task ID: phase *n*, card *nn* |
| MoSCoW | Must / Should / Could / Won't |
| WIP | Work in progress |
| DoD | Definition of Done |
| QH | Quick-hide |
| MVP | End of Phase 2 |
