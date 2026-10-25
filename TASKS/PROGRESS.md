# Progress

**The single status board.** History lives in [`../MEMORY/`](../MEMORY/README.md); this is state.

Last updated: 2026-10-25 (spike harness P0-03..P0-07 siap, menunggu hasil VM; 98 tests green).

## Where This Project Actually Is

Early implementation. `Drape.slnx` builds with 0 warnings; 98 tests pass (71 Core + 14 Infra + 7 Arch + 6 UI).
DONE: P0-01, P0-02, P0-08, P0-10, P1-01, P1-02, P1-03, P1-06, P1-07-Core, P1-11, P1-12.
P1-04 parsial: ViewModel + Window + 6 headless tests green.
Spike harness P0-03..P0-07: spec + kode + runbook DONE, kompilasi hijau; hasil empiris menunggu owner
menjalankan di VM. P0-09 (ADR final) terblokir sampai hasil itu masuk.

## Current Focus

**Nothing verifiable remains without hardware or the desktop-layer spike.** Next: P1-05 (host, needs
P0-03), P1-08/09/10 (need P1-04 window + P0-05 answer), P1-07-UI tray commands (need P1-04).

**Phase 0 spikes are still `TODO`.** No spike can be completed without the hardware matrix the cards
require; claiming otherwise would violate the "name the test" rule.

Owner decisions taken on 2026-10-10: MIT/free (Q-02), name Drape (Q-03), Windows first (Q-15), mobile scope delegated (Q-16, ADR-021), everything called Drape (Q-17). All other open questions (Q-04 … Q-13, Q-18) were answered on 2026-10-10 with research (ADR-022); none is open. See [`BACKLOG.md`](./BACKLOG.md).

## In Progress (WIP ≤ 3)

| ID | Title | Role | Started |
|---|---|---|---|
| P1-04 | DrapeWindow: basic Drape window (parsial — ViewModel + chrome + headless done) | ui-engineer | 2026-10-25 |

## Ready (priority order)

_Empty — everything verifiable without hardware or the P0-03 spike is done._

## Blocked

| ID | Title | Blocked by |
|---|---|---|
| P0-03 … P0-07 hasil VM | Hasil empiris dari owner | Owner menjalankan runbook di VM |
| P0-09 | Record spike results as ADRs | Hasil VM P0-03 … P0-07 |
| P1-04-rest, P1-05 | Desktop-layer attach, focus, DPI | P0-09 (ADR-007 final) |
| P1-08, P1-09, P1-10 | Items, drag-drop, DPI | P1-04 window + P0-05 answer |
| P1-07-UI | Tray/menu commands for DrapeService | P1-04 |

## Phase Summary

| Phase | Theme | Cards | Done | Status |
|---|---|---|---|---|
| 0 | Foundation and spikes | 10 | 4 | P0-01/02/08/10 DONE; spikes blocked on hardware |
| 1 | Core Drape | 12 | 7 | P1-01/02/03/06/07-Core/11/12 DONE; P1-04 parsial; rest blocked |
| 2 | Desktop integration (MVP) | 13 | 0 | Waiting for Phase 1 |
| 3 | Customisation and UX | 10 | 0 | Waiting for Phase 2 |
| 4 | Advanced features | 9 | 0 | Waiting for Phase 3 |
| 5 | Polish and release (v1.0 Windows) | 10 | 0 | Waiting for Phase 4 |
| 6 | Cross-platform expansion | 11 | 0 | Post-v1; spikes may start earlier |

## Done

| ID | Title | Date | Record |
|---|---|---|---|
| P1-01 | Core domain model (records, validation, one item per Drape) | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P1-02 | Interface contracts in Core, with a fake for each | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P1-06 | Layout maths (`LayoutService`) in Core | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P1-11 | Debounced persistence with a shutdown flush | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P0-01 | Scaffold repo and Avalonia solution | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P0-02 | CI on GitHub Actions + Dependabot | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P0-08 | App skeleton: composition root, DI, logging, single-instance, tray, `--restore-desktop` | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P1-03 | ConfigStore: atomic save, backup, quarantine, migration | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P1-07 | Create, rename and delete a Drape (Core logic) | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P0-10 | Custom subagent definitions | 2026-10-25 | [record](../MEMORY/records/2026-10-25-p0-10-p1-12-p1-04.md) |
| P1-12 | Phase 1 testing: coverage (Core 85.8%), baseline | 2026-10-25 | [record](../MEMORY/records/2026-10-25-p0-10-p1-12-p1-04.md) |
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
