# 2026-10-10 — Owner decisions: MIT, final name, Windows first, mobile scope, Drape everywhere (ADR-019, ADR-020, ADR-021)

**Date:** 2026-10-10
**Task:** — (decisions on open questions Q-02, Q-03, Q-15, Q-16, Q-17)
**Author:** assistant, on the owner's answers
**Spec refs:** [`docs/PLAN/05-LEGAL-AND-NAMING.md`](../../docs/PLAN/05-LEGAL-AND-NAMING.md) · [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](../../docs/PLATFORM/01-PLATFORM-SUPPORT.md)

---

## Hook

Renaming the code concept to `Drape` collides with the root namespace `Drape`, so the entity needs a documented naming rule (CS-10) instead of a plain find-and-replace; and "final name" does not mean "cleared name" — no trademark search exists.

---

## What Changed

- Owner answers: Q-02 MIT and free; Q-03 name "Drape"; Q-15 Windows first; Q-16 delegated to the assistant; Q-17 call everything Drape.
- New ADRs: ADR-019 (licence + name), ADR-020 (rename, supersedes ADR-016), ADR-021 (mobile scope). ADR-001 and ADR-014 statuses updated; ADR-016 marked superseded.
- All documents: `Fence*` identifiers became `Drape*` (`DrapeItem`, `DrapeManager`, `DrapeWindow`, `DrapeViewModel`, `DrapeBuilder`, config keys `drapes`, `drapeId`, `targetDrapeId`, branch slug `drape-window`). Glossary, legal note, risk register PR-8, data model, coding standard CS-10, CLAUDE.md, BACKLOG, PHASE-0/1/5/6 cards and PROGRESS updated.

## Why

Direct owner decisions. Q-16 was delegated ("decide it yourself, it came from an earlier agent's generation"), so ADR-021 is the assistant's working decision, not the owner's.

## Alternatives Considered

See the ADRs. For the rename: keep `Fence` (ADR-016) or `Panel` (collides with Avalonia's `Panel`).

## What Surprised Me

The entity name `Drape` equals the root namespace, which makes simple-name lookups inside `Drape.*` find the namespace first. CS-10 prescribes explicit imports, a `using` alias and suppressing CA1724 for that one type. This is reasoned, not compiled: no code exists to prove it.

## Tests

None; documents only. The link-target script was re-run after the edits (see below).

## Definition of Done

- [x] Documents updated consistently (compound identifiers replaced by script, the rest by hand)
- [x] ADRs written; `MEMORY-INDEX` and `CHANGELOG` updated
- [x] Link targets re-checked by script after the edits
- [ ] A compile-level check of the `Drape`/`Drape` clash — **not possible yet**, no code

**Waived:** none.

## Security and Compliance

No security-relevant change. Licence is now MIT (ADR-019).

## What to Watch

- P0-01 must create `LICENSE` (MIT) and must prove the namespace/type clash is manageable (or raise a new ADR).
- P5-06 must run the trademark search for "Drape".
- ADR-021 store-policy statements (Android package visibility, iOS app listing) are unverified.

## What Was Not Determined

Trademark status of "Drape"; whether CA1724 suppression plus aliasing is enough in practice; whether the mobile product in ADR-021 is worth building.

## Follow-Ups

P6-09 confirms or overturns ADR-021; Q-04, Q-05, Q-18 remain open in [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md).

## Links

- ADR: ADR-019, ADR-020, ADR-021 in [`../DECISIONS.md`](../DECISIONS.md)
- Backlog: Q-02, Q-03, Q-15, Q-16, Q-17 closed
