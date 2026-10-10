# Progress

**The single status board.** History lives in [`../MEMORY/`](../MEMORY/README.md); this is state.

Last updated: 2026-10-10 (documentation restructure, [record](../MEMORY/records/2026-10-10-docs-restructure-and-translation.md)). A count is a dated snapshot — re-count before quoting.

## Where This Project Actually Is

Planning phase. Documentation, ADRs and task cards exist; **no code, no repository history, no solution.** Nothing in `DONE` below has been built.

## Current Focus

**Phase 0 — Foundation and spikes.** Goal: solution + CI running, every Windows spike written up, ADR-007 (host) and ADR-006 (icons) finalised.
Owner decisions taken on 2026-10-10: MIT/free (Q-02), name Drape (Q-03), Windows first (Q-15), mobile scope delegated (Q-16, ADR-021), everything called Drape (Q-17). All other open questions (Q-04 … Q-13, Q-18) were answered on 2026-10-10 with research (ADR-022); none is open. See [`BACKLOG.md`](./BACKLOG.md).

## In Progress (WIP ≤ 3)

_Empty._

| ID | Title | Role | Started |
|---|---|---|---|
| — | — | — | — |

## Ready (priority order)

| # | ID | Title | Priority | Estimate | Role | Depends on |
|---|---|---|---|---|---|---|
| 1 | P0-01 | Scaffold repo and solution | P0 | 1 day | architect | — |

> Only P0-01 has no dependency. After it is `DONE`, P0-02 … P0-08 and P0-10 can run in parallel (different roles), within the WIP limit of 3.

## Blocked

| ID | Title | Blocked by |
|---|---|---|
| P0-02 … P0-08, P0-10 | Everything else in Phase 0 | P0-01 |
| P0-09 | Record spike results as ADRs | P0-03 … P0-07 |
| P1-* | All of Phase 1 | P0-09 (ADR-007 final) |

## Phase Summary

| Phase | Theme | Cards | Done | Status |
|---|---|---|---|---|
| 0 | Foundation and spikes | 10 | 0 | Ready to start |
| 1 | Core Drape | 12 | 0 | Waiting for Phase 0 |
| 2 | Desktop integration (MVP) | 13 | 0 | Waiting for Phase 1 |
| 3 | Customisation and UX | 10 | 0 | Waiting for Phase 2 |
| 4 | Advanced features | 9 | 0 | Waiting for Phase 3 |
| 5 | Polish and release (v1.0 Windows) | 10 | 0 | Waiting for Phase 4 |
| 6 | Cross-platform expansion | 11 | 0 | Post-v1; spikes may start earlier |

## Done

| ID | Title | Date | Record |
|---|---|---|---|
| SETUP-01 | Project structure: `docs/`, `MEMORY/`, `TASKS/`, `CLAUDE.md`, `AGENTS.md` | 2026-10-09 | [project setup](../MEMORY/records/2026-10-09-project-setup.md) |
| DEC-03 | Windows 10 22H2 minimum (owner); other platforms' minimums and channels (ADR-023) | 2026-10-10 | [record](../MEMORY/records/2026-10-10-windows-10-minimum-and-platform-minimums.md) |
| DEC-02 | Remaining open questions answered with research (ADR-022); root README and LICENSE | 2026-10-10 | [record](../MEMORY/records/2026-10-10-remaining-questions-readme-licence.md) |
| DEC-01 | Owner decisions: MIT, final name Drape, Windows first, mobile scope, `Fence`→`Drape` (ADR-019..021) | 2026-10-10 | [owner decisions](../MEMORY/records/2026-10-10-owner-decisions-licence-name-rename.md) |
| DOCS-01 | Restructure and translate all documentation to English; reference layout adopted | 2026-10-10 | [docs restructure](../MEMORY/records/2026-10-10-docs-restructure-and-translation.md) |

## How to Update This Board

1. Taking a task: move its row from Ready to In Progress (and set `WIP` on the card).
2. Finishing: remove it from the board sections, add it to Done with its record, update the phase summary count.
3. A dependency finishing: move the dependent task from Blocked to Ready.
4. Card counts follow the phase files; update them when cards are added or dropped.
5. Do it **in the same commit** as the work.
