# Drape

A desktop organizer. Drape groups your desktop icons, shortcuts and files into semi-transparent panels — called **Drapes** — drawn above the wallpaper. Move them, resize them, roll them up, and double-click the empty desktop to hide everything at once.

> **Status: Rust + Slint target documented; implementation not started.** Early C#/.NET/Avalonia code and dated test records remain in the repository. No Rust scaffolding, code removal or CI change is part of this update. See [progress](./TASKS/PROGRESS.md) and [migration design](./docs/ARCHITECTURE/03-RUST-MIGRATION.md).

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
| Windows 10 22H2 (x64) | **Minimum target OS**; Rust backend validation pending |
| macOS 14+, Linux (Ubuntu 22.04 / Debian 12 families) | Planned after v1.0 (Phase 6), each behind a spike |
| Android 12+, iOS 18+ | Planned last as a shortcut-group companion; minimums/backend support require revalidation |

## Tech

Rust + Slint: native compiled UI without WebView/browser engine. Windows integration uses the Microsoft windows crate behind platform-free ports. The owner prioritises RAM, performance, memory safety and distribution size; results require measurements. See [stack](./docs/ARCHITECTURE/01-TECH-STACK.md) and [ADR-025](./MEMORY/DECISIONS.md).

## Build status

The repository still contains the legacy .NET solution; a Rust workspace does not exist yet. [R0-02](./TASKS/RUST-MIGRATION.md) creates it in a later implementation task. Future Rust build/run commands are documented in [build and release](./docs/DEVOPS/00-BUILD-AND-RELEASE.md); they are not usable as migration evidence today. Legacy measurements/test counts remain historical and were not rerun for this documentation update.

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

[MIT](./LICENSE) for Drape code — free to use, modify and distribute. Slint has separate royalty-free licensing and attribution requirements; see the stack/licensing documents. The licence grants no trademark rights.

Drape is an independent project. It is not affiliated with, endorsed by or derived from any other desktop-organizer product, and it uses none of their code, names, icons or text ([`docs/PLAN/05-LEGAL-AND-NAMING.md`](./docs/PLAN/05-LEGAL-AND-NAMING.md)). The name "Drape" has not yet been through a trademark search.
