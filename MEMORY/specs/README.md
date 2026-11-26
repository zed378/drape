# MEMORY/specs/

> Existing P0/P1 specs are historical C# implementation contracts. New Rust tasks write new specs before implementation; [Rust migration specification](../../docs/ARCHITECTURE/03-RUST-MIGRATION.md) covers R0-01. Do not reinterpret checked legacy acceptance as Rust proof.

Feature specs, written **before** implementation, for tasks marked `Spec required: yes` in [`../../TASKS/`](../../TASKS/README.md).

Template: [`../templates/FEATURE-SPEC-TEMPLATE.md`](../templates/FEATURE-SPEC-TEMPLATE.md).
Naming: `<task-id>-<slug>.md`, for example `P2-03-desktop-host.md`.

Rust baseline spec: [R0-02 workspace](./R0-02-rust-workspace.md). Legacy P0/P1 specs remain historical.

Rust native-handle diagnostic spec: [R0-04 Slint desktop-host boundary](./R0-04-slint-desktop-host.md). Only read-only acquisition is implemented; native hosting and the Windows matrix remain pending.

Rust hook spec: [R0-08 minimal hook and transient state](./R0-08-minimal-hook.md). The bounded pure queue/recognizer subset is under review; native hook and VM acceptance remain pending.

Rust recovery spec: [R0-05 native icon state and independent recovery](./R0-05-native-icon-recovery.md). Written before implementation; native state/readback and Guard protocol evidence remain pending.

Rust persistence spec: [R1-02 durable config](./R1-02-durable-config.md). Written before implementation; implementation waits for R1-01 acceptance.

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
