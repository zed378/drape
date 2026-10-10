# TASKS/ — Execution Plan

`docs/` describes **what** the system is and **why**. This folder describes **what to do next, in what order, and how to know it is finished**.

Nothing here invents architecture. Every task points back to the `docs/` document or ADR that already decided the design. **If a task would need a decision `docs/` does not contain, it is not a task** — it is an entry in [`BACKLOG.md`](./BACKLOG.md) under Open Questions.

## Read This First

This project is in the **planning phase**: there is no code, no repository history and no solution yet. The first task is [P0-01](./PHASE-0-FOUNDATION.md). The board will track *remaining* work from then on; it must never list shipped work as TODO. If you find it does, the board is wrong, not the code — fix the board.

## Files

| File | Purpose |
|---|---|
| [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md) | Task IDs, statuses, card anatomy, branches, commits, the global Definition of Done, deviation protocol |
| [`PROGRESS.md`](./PROGRESS.md) | **The single status board.** What is in progress, what is next, what is done |
| [`BACKLOG.md`](./BACKLOG.md) | Open Questions (`Q-n`) and deliberate deferrals (`B-n`) |
| [`PHASE-0-FOUNDATION.md`](./PHASE-0-FOUNDATION.md) | Solution, CI, Windows spikes, app skeleton, ADRs |
| [`PHASE-1-DRAPE-CORE.md`](./PHASE-1-DRAPE-CORE.md) | Domain, config store, Drape window, items, layout, persistence |
| [`PHASE-2-DESKTOP-INTEGRATION.md`](./PHASE-2-DESKTOP-INTEGRATION.md) | Hide icons safely, Guard, Desktop sync, Inbox, quick-hide, session resilience (MVP) |
| [`PHASE-3-CUSTOMIZATION.md`](./PHASE-3-CUSTOMIZATION.md) | Rollup, style, themes, snap, lock, shell menu, accessibility |
| [`PHASE-4-ADVANCED.md`](./PHASE-4-ADVANCED.md) | Portals, rules, pages, snapshots, export/import, tabs, search, hotkeys |
| [`PHASE-5-POLISH-RELEASE.md`](./PHASE-5-POLISH-RELEASE.md) | Localisation, performance, installer, signing, audits, v1.0 (Windows) |
| [`PHASE-6-CROSS-PLATFORM.md`](./PHASE-6-CROSS-PLATFORM.md) | macOS, Linux, Android, iOS — spikes first (post-v1) |

## How a Task Moves

1. Pick the topmost ready task in [`PROGRESS.md`](./PROGRESS.md) whose dependencies are `DONE`.
2. Set it to `WIP` on its card **and** in `PROGRESS.md`, with owner and date. **At most 3 tasks are `WIP` at once.**
3. Write the feature spec first if the card says `Spec required`.
4. Work on `{type}/{task-id}-{slug}`; verify; write the record; update `PROGRESS.md` in the same commit.
5. `REVIEW` by a different role (architect for interfaces, qa for behaviour, shell engineer for anything native); then `DONE`.

New findings (bugs, ideas) go to [`BACKLOG.md`](./BACKLOG.md). Do not quietly widen a task.

## Priority

- **P0:** blocks the phase, or concerns the safety of the user's desktop.
- **P1:** needed for the phase exit criteria.
- **P2:** improves quality; may slip to the next phase.
