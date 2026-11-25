# 00 — Project Overview

## Vision

A tidy desktop without manual work: icons and files grouped into panels that can be arranged, hidden and restored in one gesture — light on resources, and never able to damage or lose a user's files or leave their desktop unusable.

## Problem

- The desktop fills up quickly with downloads, shortcuts and temporary documents.
- Windows offers no icon grouping on the desktop, only sort and auto-arrange.
- Manual tidying does not last; icon layout is often scrambled after a resolution or monitor change.
- Existing commercial products are paid and closed; users cannot adapt them.

## Target Users

| Persona | Primary need |
|---|---|
| Office worker with a crowded desktop | Group documents and shortcuts per project; hide everything quickly before a presentation |
| Developer / creator with many shortcuts | Portals to working folders, automatic rules, per-monitor layout |
| Multi-monitor / docked-laptop user | Layout that survives resolution and monitor changes |

Detailed personas and user stories: [`01-PRODUCT-REQUIREMENTS.md`](./01-PRODUCT-REQUIREMENTS.md).

## Product Value

1. Group desktop icons into Drapes that can be moved, resized and rolled up.
2. Quick-hide: double-click the desktop to hide or show everything.
3. Safe: non-destructive by default, and icons can always be restored.
4. Light: runs in the background with small CPU and memory use.
5. Customisable: colour, transparency, icon size, theme.

## Design Principles

- **Non-destructive.** Drape stores references; it does not move files ([ADR-006](../../MEMORY/DECISIONS.md)).
- **Always recoverable.** Whatever happens to the app, the user's desktop returns to normal.
- **Feels native.** Follows the platform's DPI, theme, language and behaviour.
- **Small first.** MVP is Drapes, items, persistence and basic desktop integration. Advanced features follow.
- **Measurable.** Performance budgets live in the NFRs ([`01`](./01-PRODUCT-REQUIREMENTS.md)) and are tested.
- **Portable core.** Domain logic and UI are platform-independent; everything that touches the operating system sits behind an interface in a per-platform Shell project ([ADR-025](../../MEMORY/DECISIONS.md), [ADR-015](../../MEMORY/DECISIONS.md)).

## Scope

**v1.0 (in scope)**
- Windows 10 22H2 (minimum, fully supported) and Windows 11 (primary development target), x64.
- Single user, local, no account, no cloud.
- Drapes, items, rollup, quick-hide, styles, folder portals, rules, pages and snapshots, phased across milestones. Tabs, search and global hotkeys remain optional Later candidates in the feature matrix; their Phase 4 cards do not make them mandatory v1 release gates.

**After v1.0 (Phase 6, planned but not committed)**
- macOS, Linux (X11, Wayland where the compositor allows it), Android, iOS. The architecture is cross-platform from day one; the *shipping scope* of v1.0 is Windows. See [`../PLATFORM/01-PLATFORM-SUPPORT.md`](../PLATFORM/01-PLATFORM-SUPPORT.md) and [ADR-014](../../MEMORY/DECISIONS.md).

## Non-Goals

- Not a replacement for Explorer or any file manager.
- Not a replacement for the OS shell, nor a wallpaper or system-theme changer.
- No cloud sync, accounts or telemetry in v1.
- No copying of code, assets or text from any other product ([`05-LEGAL-AND-NAMING.md`](./05-LEGAL-AND-NAMING.md)).
- Never runs elevated.
- No "desktop layer" panel on Android or iOS — those platforms have no such concept; they get a regular shortcut-group management UI ([`../PLATFORM/01-PLATFORM-SUPPORT.md`](../PLATFORM/01-PLATFORM-SUPPORT.md)).

## Success Metrics

| Metric | Target |
|---|---|
| Previously committed layout lost to process failure | 0 cases; debounced uncommitted changes and storage power-loss limits are stated in NFR-04 |
| Desktop icons not restored after the app dies | 0 cases (Guard + `--restore-desktop`) |
| Average idle CPU | App + Guard combined, machine-normalised < 0.5% |
| Memory with 20 Drapes × 20 items | < 100 MB combined App + Guard working set; private bytes/GPU memory also reported |
| Process start to Drapes ready | Median < 1.5 s (SSD), cold/warm measured separately, including required host/Guard readiness |
| Layout restored after resolution/monitor change | ≥ 95% of Drapes return to their logical position |

## Top Risks

The full list, with severity and mitigation, is in [`04-RISK-REGISTER.md`](./04-RISK-REGISTER.md). The ones that decide the project:

1. **PR-1** — the desktop window structure changes between Windows builds.
2. **PR-2** — hiding native icons relies on undocumented behaviour; failure leaves users stuck.
3. **PR-3** — a global mouse hook can be flagged by antivirus.

## Architecture direction

Rust + Slint, native compilation without WebView/browser engine, is the owner-selected target (ADR-025). Lower RAM, performance, memory safety and smaller distribution are the priorities; none is claimed achieved. Existing C# work remains legacy evidence, with replacement Rust tasks and measured parity gates. See [migration plan](../ARCHITECTURE/03-RUST-MIGRATION.md).
