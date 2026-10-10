# 00 — Task Conventions

How to work in this repository. Read before starting any task.

---

## Task IDs

```
P<phase>-<sequence>          P0-01, P2-04, P6-03
```

IDs are never reused. A cancelled task keeps its ID with status `DROPPED` and a reason. Phases are described in [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md).

## Statuses

| Status | Meaning |
|---|---|
| `TODO` | not started; dependencies may or may not be met |
| `BLOCKED` | dependencies not met, or waiting on an owner decision (name the `Q-n` or task) |
| `WIP` | in progress, on a branch |
| `REVIEW` | PR open |
| `DONE` | merged **and** its `MEMORY/records/` entry exists |
| `DROPPED` | cancelled, with the reason written on the card |

**`DONE` requires the record.** A task without one is not done, however finished the code looks.

**WIP limit: 3 tasks at the same time for the whole team.** One owner per task; a `WIP` task is not taken by another agent.

## Anatomy of a Task Card

```markdown
### P1-03 — ConfigStore: atomic save, backup, quarantine, migration

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | core-engineer |
| **Depends on** | P1-01, P1-02 |
| **Spec refs** | docs/ARCHITECTURE/02-DATA-MODEL.md · ADR-005 |
| **Spec required** | no |

**Why:** A config that can be half-written is a config that will one day lose a user's layout.

**Definition of Done**
- [ ] a simulated crash mid-write leaves the old config intact
- [ ] a corrupt config is quarantined and the newest valid backup is used

**Abuse cases**
- "Atomic" implemented as write-in-place plus a flag
```

- **Spec refs are mandatory.** A task with none is a task nobody can check.
- **Priority:** `P0` blocks the phase or concerns the safety of the user's desktop; `P1` is needed for the phase exit criteria; `P2` improves quality and may slip to the next phase. (Priority is separate from the phase number in the ID.)
- **Tasks are small:** aim for ≤ 3 working days. A card estimated above that (some spikes and platform implementations are 4–5 days) is split into sub-cards (`P3-06a`, `P3-06b`) at the moment it is taken to `WIP`, so no single card runs unreviewed for more than about 3 days.
- **Abuse cases** state how the task could be satisfied dishonestly. They exist because most of them happen somewhere.
- **Spec required: yes** means a feature spec is written first in `MEMORY/specs/<task-id>-<slug>.md` (template in `MEMORY/templates/`). Required for the host, icon hiding, Guard, hook, config store, rule engine and shell context menu cards.

## Roles

Roles and ownership boundaries are defined in [`../AGENTS.md`](../AGENTS.md): `orchestrator`, `architect`, `shell-engineer-windows`, `shell-engineer-mac`, `shell-engineer-linux`, `core-engineer`, `ui-engineer`, `mobile-engineer`, `qa`, `release-engineer`, `docs-keeper`.

## Branches

```
{type}/{task-id}-{kebab-description}

feat/P1-04-drape-window
fix/P2-02-state-recovery
docs/P0-09-record-spike-results
```

Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`. One task, one branch, one PR. `main` stays buildable and green.

## Commits

```
feat(core): P1-01 add domain records

- Config, Page, Drape, DrapeItem, Bounds, Style, State as immutable records
- Validation covers minimum size and unique IDs
- Closes the model gap named in docs/ARCHITECTURE/02-DATA-MODEL.md
```

Conventional Commits with the project scope and the task ID in the subject. The body explains **why**, not what — git has the what. Scopes: `core`, `shell`, `ui`, `app`, `infra`, `guard`, `docs`, `ci`. Commit messages are English.

## The Global Definition of Done

Every task inherits this. A task-specific DoD **adds** to it.

### Code

- [ ] Implements the spec in `docs/`
- [ ] Follows [`../docs/ENGINEERING/00-CODING-STANDARDS.md`](../docs/ENGINEERING/00-CODING-STANDARDS.md)
- [ ] No dead code, no commented-out code, no stray debug output
- [ ] Architecture rules in [`../CLAUDE.md`](../CLAUDE.md) hold (Core is platform-free; native calls only in `Shell.*`; capabilities, not OS checks)

### Desktop Safety — unwaivable

- [ ] Every path that hides native icons has a matching restore on **every** exit path (normal, exception, Guard, next start, `--restore-desktop`)
- [ ] No automated script or test kills `explorer.exe`; Explorer-restart tests are manual, in a VM or test account
- [ ] No user file is moved, renamed or deleted without explicit confirmation
- [ ] No handle, PIDL or COM object leaks (wrapped in `SafeHandle`/`IDisposable`)
- [ ] Hook callbacks stay minimal and store no input

### Testing

- [ ] Unit tests for new Core logic, **including the error branches** (written first for Core)
- [ ] Negative cases — the positive case alone proves nothing
- [ ] An architecture/headless/E2E test where the layer boundary or UI behaviour changed
- [ ] Tests pass at the coverage gate (Core ≥ 80%)
- [ ] Where a test guards something load-bearing, a **mutation check**: break the thing, watch the right test fail

### Build

- [ ] `dotnet build -c Release -warnaserror` passes with no warnings
- [ ] `dotnet format --verify-no-changes` is clean
- [ ] `dotnet test` is green
- [ ] Shell/UI changes: the relevant manual checklist in [`../docs/TESTING/00-TESTING-STRATEGY.md`](../docs/TESTING/00-TESTING-STRATEGY.md) was run, on Windows 11 and Windows 10 22H2, with at least one non-100% DPI

### Documentation

- [ ] `docs/` amended **if and only if** reality changed — through the deviation protocol
- [ ] An ADR if a decision was made or a `docs/` document deviated from

### Record

- [ ] `MEMORY/records/YYYY-MM-DD-<task-id>-<slug>.md` written
- [ ] A line added to `MEMORY/MEMORY-INDEX.md`
- [ ] A `MEMORY/CHANGELOG.md` entry if user-visible or operationally significant
- [ ] `TASKS/PROGRESS.md` updated **in the same commit**

## What May Be Waived, and What May Not

**Desktop-safety items are not waivable by anyone.** Not for a deadline, not for a demo, not with a follow-up ticket.

Everything else is negotiable with a recorded decision **naming who agreed**, written into the change record.

## The Deviation Protocol

When implementation reveals `docs/` is wrong, incomplete or contradictory:

1. **Stop.** Do not quietly implement something different.
2. Write an **ADR** in `MEMORY/DECISIONS.md` — decision, rationale, **alternatives considered**, implications **including the bad ones**.
3. Amend the `docs/` document, referencing the ADR.
4. Note both in the change record.

**A documented deviation is a decision. An undocumented one is a bug nobody has found yet.**

If the deviation would need a decision the owner has not made, it is an Open Question in [`BACKLOG.md`](./BACKLOG.md), not a judgement call.

## Evidence

> **An assertion that a test passed is not evidence. Name the test.**

"Layout tested, all good" with no test named is **worse than saying nothing**, because it stops anyone looking again.

Related rules:

- **A test generated from the code it tests verifies consistency, never correctness.** A migration test that round-trips the migrator's own output cannot catch a field being dropped; use hand-written fixtures per version.
- **A mock proves the client, not the contract.** A fake `IDesktopHost` proves the consumer; only the real implementation on a real Windows build proves the host.
- **A spike result names the build.** "Works on Windows 11" means nothing; "works on 24H2 build 26100.xxxx, 150% DPI, two monitors" is evidence.

## Distinguish "Renders" From "Works"

A window that shows on the developer's machine *renders*. It *works* when it meets the six host criteria on every build in the matrix. State which one you mean. Do not round up.

## Traps Worth Checking Before You Finish

Each is structural rather than careless. More in [`../docs/PLAN/04-RISK-REGISTER.md`](../docs/PLAN/04-RISK-REGISTER.md).

| Trap | Consequence |
|---|---|
| Hard-coding the desktop window hierarchy | Breaks on the next Windows build |
| Treating the icon toggle as a setter | A wrong state read flips the user's desktop |
| An exit path that does not restore icons | The user is stranded |
| `%USERPROFILE%\Desktop` | Wrong on OneDrive-redirected machines |
| Returning a UI-framework image type from Core | Breaks portability; return `IconBitmap` |
| `OperatingSystem.IsWindows()` in UI/Core | Bypasses the capability model; use `IPlatformCapabilities` |
| Touching an Avalonia object off the UI thread | Intermittent crashes; marshal with `Dispatcher.UIThread` |
| A slow hook callback | The OS disables the hook; input is affected system-wide |
| Dropping from an elevated process | Blocked by UIPI; document, never escalate |
| Reading icons that trigger OneDrive hydration | Unwanted downloads |

## Before Opening a PR

```bash
dotnet build Drape.sln -c Release -warnaserror
dotnet format --verify-no-changes
dotnet test Drape.sln
```

A green run of these is not a green release: E2E and the manual checklist are separate.

## PR

**Title:** `feat(core): P1-01 add domain records`

**Body:** spec refs, what changed and why, **named tests**, DoD checklist, anything waived and who agreed, anything **not determined**.

The last item is not optional. A PR that omits what it could not verify will be trusted more than it should be.
