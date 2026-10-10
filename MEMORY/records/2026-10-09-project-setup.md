# Project Setup (retrospective)

**Date:** 2026-10-09 (reconstructed on 2026-10-10)
**Task:** SETUP-01 in the old `TASKS/DONE.md` (not a `Pn-nn` task)
**Author:** orchestrator agent, written afterwards by the 2026-10-10 documentation agents
**Spec refs:** none (this created them)

> **This record is a reconstruction.** It was written on 2026-10-10 from the old `MEMORY/CHANGELOG.md`, `TASKS/DONE.md`, `MEMORY/decisions-log.md` and the ADR files. Nobody wrote it on 2026-10-09. It states what those sources claim; it cannot say what was confusing or rejected during the day, because that was not recorded. Per the honesty rules in [`../README.md`](../README.md), a reconstruction says so.

---

## Hook

The project started with a full set of planning documents and no code, and an answer to the stack question (WPF, Windows only) that the owner overturned within a day. Any decision from this date that names WPF or "Windows only" is stale; read [`../DECISIONS.md`](../DECISIONS.md) Part II first.

---

## What Changed

Per the old changelog and DONE archive, on 2026-10-09:

- Project structure set: `docs/`, `MEMORY/`, `TASKS/`, `CLAUDE.md`, `AGENT.md`.
- 15 technical documents written in Indonesian (numbered `00` to `14`): overview, PRD, features, architecture, tech stack, data model, UI/UX, Win32 integration, coding standards, testing strategy, build/release, roadmap, glossary, legal and naming, security and privacy.
- Five ADR files in `docs/adr/` (`0001` to `0005`: tech stack, config storage, desktop icon strategy, window per Drape and host layer, single process with Guard), plus a second `0001` after the stack changed to Avalonia.
- Task board and milestone files M0 to M5; M0 (foundation) marked as running.
- Decisions D-001 to D-011 and open questions Q-01 to Q-14 recorded.
- The old DONE archive lists one completed item: SETUP-01, "all files created; cross-document links checked".

These decisions are now ADR-001 to ADR-011 in [`../DECISIONS.md`](../DECISIONS.md).

---

## Why

The owner wanted a working pattern for Claude Code and several agents in parallel (ADR-002). With no code, the documents are the only product, and agents without memory need files to read at the start of each session.

---

## Alternatives Considered

Not recorded at the time beyond what the ADR files list. The alternatives for each individual decision are in the ADRs.

---

## What Surprised Me

Not recorded on the day. What later turned out to be wrong, found on 2026-10-10, is in the [restructure record](./2026-10-10-docs-restructure-and-translation.md): the name rationale, a leftover `iconsHiddenByPagar`, contradictions between documents, and a stack choice that was changed by the owner.

---

## Tests

None. There was no code. The only check mentioned in the sources is the cross-document link check in SETUP-01; the method used is not recorded.

---

## Definition of Done

Not applicable: the DoD in [`../../TASKS/00-TASK-CONVENTIONS.md`](../../TASKS/00-TASK-CONVENTIONS.md) was written after this date.

**Waived:** not applicable.

---

## Security and Compliance

No code, no security-relevant change. The safety principles later made binding (non-destructive icon handling, guaranteed restore) were already in the first drafts.

---

## What to Watch

- Everything in the 2026-10-09 documents was Indonesian and has since been translated and restructured; any link to an old path (`docs/0n-*.md`, `docs/adr/`, `MEMORY/open-questions.md`, `TASKS/BOARD.md`) is dead.
- The old decisions were marked Confirmed / Assumption / Hypothesis; the new ADRs carry that in their Status line, and assumptions point to a `Q-n` in [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md).

---

## What Was Not Determined

- Who authored which document and what was reviewed by the owner: not recorded.
- Whether the cross-document link check was done by a tool or by reading: not recorded.
- No platform behaviour described in these documents was verified.

---

## Follow-Ups

All tracked in [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md) (Q-01 to Q-14 were carried over).

---

## Links

- ADR: ADR-001 to ADR-011
- Changelog: [2026-10-09 entry](../CHANGELOG.md)
- Related records: [2026-10-10 restructure](./2026-10-10-docs-restructure-and-translation.md)
