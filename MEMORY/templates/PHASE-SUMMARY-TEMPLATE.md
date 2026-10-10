# Phase <N> — <Name> — Summary

**Completed:** YYYY-MM-DD
**Duration:** <start> to <end>
**Tasks:** <n> completed, <n> deferred

> Save to `MEMORY/records/YYYY-MM-DD-phase-<n>-summary.md`.
>
> A phase is complete only when every task is `DONE`, every `DONE` task has a record, and this summary exists. See `docs/PLAN/03-IMPLEMENTATION-ROADMAP.md` for exit criteria.

---

## What Shipped

*The capability, in a sentence a non-engineer would understand. Then the list.*

| Task | Title | Record |
|---|---|---|

---

## What Deviated From `docs/`

*Every deviation, with its ADR. A documented deviation is a decision; an undocumented one is a bug nobody has found yet.*

| Deviation | ADR | Why |
|---|---|---|

**If this table is empty, check again.** A phase that implemented a specification with no surprises is unusual.

---

## Spike Results (Phase 0 and Phase 6)

*One row per spike: what was tried, on which Windows builds / DPI / monitor layouts, and the outcome, including failures.*

| Spike | Question | Environments | Outcome | ADR updated |
|---|---|---|---|---|

---

## What Was Deferred

| Item | Waiting on | Backlog entry |
|---|---|---|

---

## What Failed

*Approaches abandoned, tests that did not pass, budgets missed (memory, CPU), safety checks that did not fire.*

**This is the highest-value section.** A phase summary with an empty one is a summary nobody will trust.

---

## Desktop-Safety Outcomes

*Named tests and results. An assertion that a test passed is not evidence.*

| Control | Test | Result |
|---|---|---|
| Icons restored when the process is killed | *name the test* | |
| Icons restored after a forced reboot (`state.json`) | | |
| `--restore-desktop` works with the app not running | | |
| Window host survives an Explorer restart | | |
| Hook callback stays within its time/alloc budget | | |
| Config survives a crash mid-write | | |

**No safety finding may be waived.** If one was found, it was fixed before this phase closed, or this phase is not closed.

---

## Platform and Budget Outcomes

| Requirement | Evidence (named test / measurement) | Environment |
|---|---|---|
| Idle memory budget | | |
| Idle CPU budget | | |
| Mixed-DPI correctness | | |
| Windows 10 22H2 behaviour (minimum supported) | | |

---

## What Was Not Determined

*Things that could not be verified: an environment that was unavailable, a test that could not run, a claim that rests on inference.*

**Rounding this up is how a status report becomes untrustworthy.**

---

## Definition of Done

- [ ] Every task in the phase file is `DONE`
- [ ] Every `DONE` task has a record in `MEMORY/records/`
- [ ] The global Definition of Done is satisfied for each, or the waiver is recorded with **who agreed**
- [ ] This summary exists
- [ ] Safety outcomes are **named**, not asserted
- [ ] `TASKS/PROGRESS.md` reflects reality
- [ ] ADRs whose status depended on this phase's spikes are updated (Accepted or Superseded)

---

## What to Watch

*What might break as the next phase builds on this: fragile assumptions, behaviour that depends on one Windows build, anything held together by convention rather than mechanism.*

---

## Metrics

| | |
|---|---|
| Tasks completed | |
| ADRs written | |
| Specification gaps found | |
| Defects found **after** a task was marked done | |
| Test count and coverage of `Drape.Core` at phase close | |

A defect found after `DONE` is a gap in the Definition of Done, not just a bug. Ask what would have caught it.

---

## Next Phase

*What is unblocked now, what is still blocked and by what.*

**Never build a Phase N+1 feature while Phase N is incomplete.**
