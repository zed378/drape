# MEMORY/specs/

Feature specs, written **before** implementation, for tasks marked `Spec required: yes` in [`../../TASKS/`](../../TASKS/README.md).

Template: [`../templates/FEATURE-SPEC-TEMPLATE.md`](../templates/FEATURE-SPEC-TEMPLATE.md).
Naming: `<task-id>-<slug>.md`, for example `P2-03-desktop-host.md`.

**Current list: empty.** No task has started and no code exists.

---

## What a Feature Spec Is

A short document, written by whoever will implement a risky task, that lists what `docs/` already decides, what it leaves open, and what could go wrong, **before** the first line of code. Its purpose is to surface decisions and traps while they are still cheap. It is not a design essay and it does not replace `docs/`; where `docs/` is silent, contradictory or wrong, the spec says so and the gap goes to [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md) (and an ADR if a decision is needed).

The 2026-10-10 documentation review is the model case: careful reading found that the build document contradicted the privacy document and that Core was specified to return a UI type. A spec asking "which layer owns this, and what does it return?" finds that in minutes.

## When a Spec Is Required

A card has **Spec required: yes** per [`../../TASKS/00-TASK-CONVENTIONS.md`](../../TASKS/00-TASK-CONVENTIONS.md). That covers the areas where a mistake costs the user their desktop or breaks the layering:

| Required | Not required |
|---|---|
| Desktop host (`IDesktopHost`) | a bug fix within an existing shape |
| Hiding and restoring native desktop icons, `Drape.Guard`, `--restore-desktop` | a UI-only change with no new platform call |
| The global input hook | a dependency bump |
| Config store, schema change or migration | documentation-only change |
| Rule engine | a new test for existing behaviour |
| Shell context menu | |
| Anything that adds a platform capability or a new Shell project | |

When in doubt, write the spec; it is cheaper than the defect.

## How to Write One

1. Copy the template to `specs/<task-id>-<slug>.md` and fill in **every** section ("Not applicable" is valid, blank is not).
2. Read every document in the card's Spec refs first, and list what they already decide. Do not re-decide it.
3. Commit the spec **before** the implementation. Change the card's status to `WIP` only after that.
4. If the implementation then deviates from the spec, update the spec and record the reason in the task's change record, or add an ADR if the decision reaches beyond the task.
5. Add a line to [`../MEMORY-INDEX.md`](../MEMORY-INDEX.md) under Specs.

A spec says **what was intended**. The change record says **what happened**. Both stay.
