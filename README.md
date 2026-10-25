# Drape

A desktop organizer. Drape groups your desktop icons, shortcuts and files into semi-transparent panels — called **Drapes** — drawn above the wallpaper. Move them, resize them, roll them up, and double-click the empty desktop to hide everything at once.

> **Status: early implementation. Core domain, config store, layout maths and Windows capability skeleton exist and are tested (81 tests green).** The desktop host, icon hiding and UI windows are still hypotheses until their spikes prove them. See [`TASKS/PROGRESS.md`](./TASKS/PROGRESS.md) for the current state.

## What It Will Do

- Group desktop items into Drapes; drag items in from Explorer or the desktop.
- Roll up, snap, lock and style each Drape (tint, transparency, blur, icon size).
- **Quick-hide:** double-click empty desktop to hide or show every Drape.
- Folder portals, auto-sort rules, desktop pages, layout snapshots, export/import.
- Keep your layout across resolution and monitor changes.

## Principles

- **Non-destructive.** Drape stores references to your files. It never moves, renames or deletes a file without explicit confirmation.
- **Always recoverable.** If the app is closed, killed or crashes, your native desktop icons come back (`--restore-desktop`, a small Guard process, and recovery at the next start).
- **Private.** No telemetry and no network by default. Update checks are opt-in.
- **Unprivileged.** Runs as a normal user; never asks for administrator rights.

## Platforms

| | Status |
|---|---|
| Windows 11 (x64) | **v1.0 target**, primary |
| Windows 10 22H2 (x64) | **Minimum supported OS** |
| macOS 14+, Linux (Ubuntu 22.04 / Debian 12 families) | Planned after v1.0 (Phase 6), each behind a spike |
| Android 12+, iOS 18+ | Planned last, as a shortcut-group companion without a desktop layer |

## Tech

C# / .NET 10 · Avalonia UI · CommunityToolkit.Mvvm · Microsoft.Extensions.Hosting · Serilog · CsWin32 (Windows shell integration) · xUnit, NSubstitute, Shouldly. Details and alternatives: [`docs/ARCHITECTURE/01-TECH-STACK.md`](./docs/ARCHITECTURE/01-TECH-STACK.md).

## Building

Not possible yet: the solution is created by task [P0-01](./TASKS/PHASE-0-FOUNDATION.md). Once it exists the commands will be:

```
dotnet restore
dotnet build Drape.sln -c Debug
dotnet test Drape.sln
dotnet run --project src/Drape.App.Desktop
dotnet run --project src/Drape.App.Desktop -- --restore-desktop   # restore desktop icons
```

## Documentation

| Folder | Contents |
|---|---|
| [`docs/`](./docs/README.md) | Reference: plan, architecture, platform integration, UI/UX, engineering, testing, DevOps, security |
| [`TASKS/`](./TASKS/README.md) | Execution plan: phases, task cards, progress board, backlog |
| [`MEMORY/`](./MEMORY/README.md) | History: decisions (ADRs), change records, feature specs |
| [`CLAUDE.md`](./CLAUDE.md), [`AGENTS.md`](./AGENTS.md) | Rules for AI agents and role definitions (also read by humans) |

Start with [`docs/README.md`](./docs/README.md) for the recommended reading order.

## Contributing

Work is organised as small task cards in [`TASKS/`](./TASKS/README.md). Read [`TASKS/00-TASK-CONVENTIONS.md`](./TASKS/00-TASK-CONVENTIONS.md) first: it defines the branch and commit conventions and the Definition of Done, including the desktop-safety rules that cannot be waived. Ideas and bugs go to [`TASKS/BACKLOG.md`](./TASKS/BACKLOG.md). All repository content (docs, code, comments, commits) is in English.

## Licence

[MIT](./LICENSE) — free to use, modify and distribute. The licence grants no trademark rights.

Drape is an independent project. It is not affiliated with, endorsed by or derived from any other desktop-organizer product, and it uses none of their code, names, icons or text ([`docs/PLAN/05-LEGAL-AND-NAMING.md`](./docs/PLAN/05-LEGAL-AND-NAMING.md)). The name "Drape" has not yet been through a trademark search.
