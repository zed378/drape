# MEMORY/ — Change Record and Decision Log

`TASKS/` is what will be done. `MEMORY/` is what **was** done, and why it ended up that way.

`docs/` describes the intended (later: as-built) system and the code describes the current system, but neither explains how one became the other. Months from now, the question "why does the Guard exist, and why is the host strategy still undecided?" is answerable from here and nowhere else. The code shows the behaviour; only the record shows what the decision cost.

Agents and contributors do not retain memory between sessions. `MEMORY/` is the mechanism for continuity, which is why writing to it is part of a task rather than a summary appended afterwards.

**Phase note:** the project is in the planning phase and there is no code yet. Right now this folder holds decisions and two records about the documentation itself. It will grow as soon as tasks reach `DONE`.

---

## Structure

| Path | Purpose |
|---|---|
| [`MEMORY-INDEX.md`](./MEMORY-INDEX.md) | One line per record and per decision, newest first. The entry point. |
| [`CHANGELOG.md`](./CHANGELOG.md) | Chronological summary, coarser than the records. |
| [`DECISIONS.md`](./DECISIONS.md) | The ADRs: every choice `docs/` left open, and every deviation from it. |
| [`records/`](./records/) | One file per completed task (and per significant event): what changed, why, and what to watch. |
| [`specs/`](./specs/README.md) | Feature specs for tasks marked `Spec required`, written **before** implementation. |
| [`templates/`](./templates/) | Change-record, phase-summary and feature-spec templates. |

**Where the current status lives:** [`../TASKS/PROGRESS.md`](../TASKS/PROGRESS.md), not here. This folder is history; the board is state. A snapshot in two places drifts, and then neither can be trusted.

## Read `DECISIONS.md` Part II First

`DECISIONS.md` has two parts.

- **Part I (ADR-001 to ADR-011)** was written on 2026-10-09, before anything was built. Some entries were overtaken: the WPF stack and the Indonesian-documentation rule.
- **Part II (ADR-012 onward)** records what changed or was added, and says which earlier ADR each one supersedes.

Earlier ADRs are **left in place**; only their Status line changes. An ADR records what was decided at a moment, and rewriting it destroys the evidence that the decision changed.

## What Gets a Record

**Every task that reaches `DONE`.** That is a Definition of Done item in [`../TASKS/00-TASK-CONVENTIONS.md`](../TASKS/00-TASK-CONVENTIONS.md): a task without a record is not done, however finished the code looks.

Also recorded:

- **Any deviation from `docs/`**, as an ADR, per the deviation protocol.
- **Any decision `docs/` deliberately left open**, for example the host-layer strategy or the icon-hiding technique.
- **Spike results.** A spike is a task whose product is knowledge; its record holds the results table for each Windows build, DPI setting and monitor layout it ran on, including the failures.
- **Phase completions**, using [`templates/PHASE-SUMMARY-TEMPLATE.md`](./templates/PHASE-SUMMARY-TEMPLATE.md).
- **Safety-relevant outcomes**: a kill-the-process restore test, a forced-reboot recovery test, an Explorer-restart test. An assertion that a test passed is not evidence.
- **Operational events with lasting consequence**: a release rollback, a signing-key change.
- **Large documentation changes**, such as the 2026-10-10 restructure.

## What Does Not Get a Record

- Work in progress. Records describe completed changes.
- Anything git history already tells accurately. A record explains **why**, not what changed on which line.
- Restating a `docs/` document. Link to it instead.

## Writing a Record

1. Copy [`templates/CHANGE-RECORD-TEMPLATE.md`](./templates/CHANGE-RECORD-TEMPLATE.md).
2. Name it `records/YYYY-MM-DD-<task-id>-<slug>.md` (no task ID for non-task work, as in the two existing records).
3. Fill in **every** section. "Not applicable" is valid; a blank section is not.
4. Add a line to [`MEMORY-INDEX.md`](./MEMORY-INDEX.md), newest first.
5. Add a [`CHANGELOG.md`](./CHANGELOG.md) entry if the change is user-visible or significant for the project.
6. Add an ADR to [`DECISIONS.md`](./DECISIONS.md) if a decision was made or a `docs/` document was deviated from.
7. Commit it **with the code**, not afterwards. A record written a week later is a reconstruction (the first record here is exactly that, and says so).

## Writing an ADR

Use the format at the top of `DECISIONS.md`. The two sections easiest to skip are the two that matter:

- **Alternatives Considered**: lets a future reader tell whether their new idea was already rejected or never considered.
- **Implications, including the bad ones.** An ADR listing only benefits will not be trusted when someone must decide whether to revisit it.

## Honesty Rules

A record is only worth reading if it can be trusted.

- **Record what happened, not what was supposed to happen.** If a test was skipped, say so. If a Definition of Done item was waived, say which and **who agreed**.
- **Record failures.** A spike that failed, an approach abandoned after two days: these are the most valuable records, because they stop the same ground being covered twice.
- **Never claim a safety test passed without naming it.** "Icon restore tested, all good" with no test named is worse than silence.
- **Distinguish "written" from "works".** Documentation that was written but never built or run is a plan. Say "no platform behaviour here has been verified" when that is the truth.
- **Do not retroactively edit a record to look better.** Add a follow-up record. If you only fix a typo or a broken link, say so.
- **Reconstructed records say they are reconstructed**, and from what.
- **Record open questions found during the work** and add each to [`../TASKS/BACKLOG.md`](../TASKS/BACKLOG.md) so it has a consequence.
- No secrets, personal paths or real config dumps in any record.

## Relationship to Other Folders

| Folder | Direction | Nature |
|---|---|---|
| [`../docs/`](../docs/README.md) | Reference | What the system is (intended now, as-built once code exists). Amended only through the deviation protocol. |
| [`../TASKS/`](../TASKS/README.md) | Forward | What will be built, in what order, and how it is judged done. |
| `MEMORY/` | Backward | What was built or decided, what it cost, and what to watch. |

A `docs/` document changing in substance is itself an event worth a record: it means reality taught the specification something.
