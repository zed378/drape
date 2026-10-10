# Documentation Restructure and Translation to English

**Date:** 2026-10-10
**Task:** — (documentation overhaul, not a `Pn-nn` task)
**Author:** orchestrator agent with four parallel documentation agents, at the owner's request
**Spec refs:** [`../../docs/README.md`](../../docs/README.md), [`../../TASKS/00-TASK-CONVENTIONS.md`](../../TASKS/00-TASK-CONVENTIONS.md)

---

## Hook

The documents still described a WPF, Windows-only app after the owner had already chosen cross-platform Avalonia; the stated reason for the name "Drape" ("fence in Indonesian") was simply false; an earlier working name, "Pagar", survived as a config field `iconsHiddenByPagar`; and the Core layer was specified to return a WPF `ImageSource`, breaking the project's own first architecture rule. None of it was visible while the documents were in separate files in one language.

---

## What Changed

- **Everything translated to English** (ADR-013). Conversation with the owner stays Indonesian.
- **Layout adopted** from the reference repository (ADR-018): `docs/<CATEGORY>/NN-*.md`, ADRs in `MEMORY/DECISIONS.md`, `TASKS/` as progress + backlog + phases, `MEMORY/` with records, specs and templates.
- **Files removed or merged:**
  - `AGENT.md` renamed `AGENTS.md`; `GEMINI.md` thinned to a pointer.
  - `docs/00..14-*.md` moved to `docs/<CATEGORY>/NN-*.md`.
  - `docs/adr/*` merged into `MEMORY/DECISIONS.md` (ADR-004 to ADR-008, ADR-012).
  - `TASKS/BOARD.md`, `TASKS/DONE.md` and `TASKS/M0..M5` replaced by `PROGRESS.md` and `PHASE-0..6` (task IDs `Mn-nn` became `Pn-nn`).
  - `MEMORY/decisions-log.md`, the old `MEMORY/DECISIONS.md`, `open-questions.md`, `project-context.md`, `lessons-learned.md`, `user-preferences.md` merged into `DECISIONS.md`, `TASKS/BACKLOG.md`, `docs/PLAN` and `CLAUDE.md`.
  - `MEMORY/MEMORY-INDEX.md` and `CHANGELOG.md` rewritten: the old ones pointed to `records/`, `specs/` and `TASKS/PROGRESS.md` that did not exist.
- **Inconsistencies fixed** (all in documents; none verified by building anything):
  1. F-03 said an item may be in several Drapes while D-011 said exactly one. Resolved to one (ADR-011).
  2. The build document specified periodic update checks; the security document said no network by default. Resolved: opt-in (ADR-017).
  3. Core returned `ImageSource`, a UI type. Now a neutral `IconBitmap` (ADR-015).
  4. Known Folder lookup and autostart sat in Infrastructure although native/registry. Moved to Shell (ADR-015).
  5. The board omitted M0-08 and M0-10. Now present as P0-08 and P0-10.
  6. Task roles used a single `shell-engineer` although Shell is now per platform. Now per-platform roles.
  7. `tests/Drape.TestSupport` was used but absent from the structure. Added.
  8. Spikes M0-03b/c were mentioned but not listed. Now P6-01 to P6-04.
  9. The macOS window level was named `.desktopIcon`, which is not a real member. Replaced by `CGWindowLevelForKey`-based wording.
  10. The Visual Studio MAUI workload was listed as a prerequisite for an Avalonia project. Removed.
  11. Two ADR files were numbered `0001`. ADRs now have one numbering (ADR-001 to ADR-018).
  12. The name rationale and the `Pagar` leftover (ADR-001; the field is now `iconsHiddenByDrape`).
- **New content:** `docs/PLATFORM/01-PLATFORM-SUPPORT.md`, Phase 6, task P2-13, ADR-012 to ADR-018, open questions Q-15 to Q-18.

---

## Why

The owner asked for English and for the structure of the reference repository. The restructure surfaced the contradictions above because every document was re-read to be moved and translated. Reasoning for each new rule is in ADR-012 to ADR-018.

The owner's work-in-progress limit stays at **3** tasks.

---

## Alternatives Considered

- **Keep the flat numbered documents** and only translate: smaller change, but the cross-folder rules (records, specs, backlog) would stay missing, and the reference layout was an explicit request.
- **Keep `MEMORY/` in Indonesian** while translating `docs/`: two languages for one project; rejected with ADR-013.
- **Translate but not fix inconsistencies**, leaving a faithful but wrong copy: rejected; translating a contradiction preserves it in a new language.
- **Rewrite the history** so the new layout looked like the original: rejected; Part I of the ADRs stays as written and Part II shows what changed.

---

## What Surprised Me

- The mismatch with ADR-012 (cross-platform) was not noticed earlier because the stack change had been written as a new ADR file while the other documents kept saying WPF.
- A wrong rationale for the product name had been carried through the decisions log unchallenged.
- The reference repository is built for a large, multi-tenant, many-agent project. Its machinery (phase summaries, a spec per risky task) is heavy for a pre-code project; I kept it because the layout was requested, and scaled each file to what exists.

---

## Tests

None. This is documentation only. No build, no run, no link checker, no spelling or translation review by the owner.

- **Mutation check:** not applicable.

---

## Definition of Done

Per [`../../TASKS/00-TASK-CONVENTIONS.md`](../../TASKS/00-TASK-CONVENTIONS.md) where applicable to documentation:

- [x] Documentation updated (this is the documentation change)
- [x] This record written
- [x] Links machine-verified: relative Markdown links in all `.md` files were checked by script on 2026-10-10 after the restructure — 0 broken. Anchors (`#...`) and external URLs were not checked.
- [ ] Owner review of the English translation: **not done**
- [ ] Code, build, format, tests: not applicable, no code

**Waived:** build, format and test items are not applicable. Link verification is not waived but was not performed.

---

## Security and Compliance

No security-relevant code change. One privacy-relevant document change: update checks are now opt-in (ADR-017), which makes the "no network by default" statement consistent. No legal advice was sought on the `Fence` identifier choice (ADR-016).

---

## What to Watch

- Stale links: old paths (`docs/0n-*.md`, `docs/adr/`, `TASKS/BOARD.md`, `MEMORY/open-questions.md`, `AGENT.md`) may remain in places.
- `CLAUDE.md`, `AGENTS.md` and `GEMINI.md` were written by other agents in parallel; check they agree with each other and with `TASKS/00-TASK-CONVENTIONS.md`.
- ADR-014 and ADR-009 are assumptions the owner has not confirmed.
- Translation may have shifted a nuance, especially in the safety rules.

---

## What Was Not Determined

- **Nothing was built or run.** No solution exists.
- **No platform claim was verified.** Every statement about Win32, Explorer's window structure, Avalonia re-parenting, macOS levels and Linux compositors remains a hypothesis (see the Unverified Claims table in [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md)).
- Link targets were machine-checked at the end of the restructure (see Definition of Done); heading anchors and external URLs were not.
- Whether the owner agrees with the English phrasing of the documents.

---

## Follow-Ups

- Run a link check over `docs/`, `TASKS/` and `MEMORY/` and fix what it finds.
- Owner decisions: Q-02, Q-05, Q-15, Q-16, Q-17, Q-18 in [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md).
- Phase 0 spikes will turn the unverified claims into evidence (P0-03 and P0-04 gate ADR-007 and ADR-006).

---

## Links

- ADR: ADR-001 (corrected rationale), ADR-012 to ADR-018 in [`../DECISIONS.md`](../DECISIONS.md)
- Changelog: [2026-10-10 entry](../CHANGELOG.md)
- Backlog: Q-15 to Q-18
- Related records: [2026-10-09 project setup](./2026-10-09-project-setup.md)
