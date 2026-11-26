# 00 — Task Conventions

How to work in this repository. Read before starting any task.

---

## Task IDs

```
P<phase>-<sequence>          P0-01 (legacy), P2-04, P6-03
R<phase>-<sequence>          R0-01, R1-01 (Rust replacement foundation/core)
```

IDs are never reused. ADR-025 reserves R0/R1 for Rust foundation/core replacement; legacy P0/P1 retain their historical statuses and are not active claims. See [Rust migration](./RUST-MIGRATION.md). A cancelled task keeps its ID with status `DROPPED` and a reason. Phases are described in [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md).

## Statuses

| Status | Meaning |
|---|---|
| `TODO` | not started; dependencies may or may not be met |
| `BLOCKED` | dependencies not met, or waiting on an owner decision (name the `Q-n` or task) |
| `WIP` | claimed work in progress; record the branch when one is created |
| `REVIEW` | concrete changes ready for different-role review; link the PR if one was requested/created |
| `DONE` | merged **and** its `MEMORY/records/` entry exists |
| `DROPPED` | cancelled, with the reason written on the card |

**`DONE` requires the record.** A task without one is not done, however finished the code looks.

**WIP limit: 3 tasks at the same time for the whole team.** One owner per task; a `WIP` task is not taken by another agent.

## Anatomy of a Task Card

```markdown
### R1-02 — ConfigStore: atomic save, backup, quarantine, migration

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | core-engineer |
| **Depends on** | R1-01 |
| **Spec refs** | docs/ARCHITECTURE/02-DATA-MODEL.md · ADR-005 |
| **Spec required** | yes |

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

feat/R1-04-drape-window
fix/P2-02-state-recovery
docs/R0-10-record-spike-results
```

Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`. One task, one branch, one PR. `main` stays buildable and green.

## Commits

```
feat(core): R1-01 add domain structs

- Config, Page, Drape, DrapeItem, Bounds, Style, State as owned structs/enums
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
- [ ] Architecture rules in [`../CLAUDE.md`](../CLAUDE.md) hold (Core is platform-free; native calls only in `shell-*`; capabilities, not OS checks)

### Desktop Safety — unwaivable

- [ ] Every path that hides native icons has a matching restore on **every** exit path (normal, exception, Guard, next start, `--restore-desktop`)
- [ ] No automated script or test kills `explorer.exe`; Explorer-restart tests are manual, in a VM or test account
- [ ] No user file is moved, renamed or deleted without explicit confirmation
- [ ] No handle, PIDL or COM object leaks (wrapped in RAII wrappers with the matching native release API)
- [ ] Hook callbacks stay minimal and store no input

### Testing

- [ ] Unit tests for new Core logic, **including the error branches** (written first for Core)
- [ ] Negative cases — the positive case alone proves nothing
- [ ] An architecture/headless/E2E test where the layer boundary or UI behaviour changed
- [ ] Tests pass at the coverage gate (Core ≥ 80%)
- [ ] Where a test guards something load-bearing, a **mutation check**: break the thing, watch the right test fail

### Build

Rust gates below become valid only after R0-02. Docs-only work checks Markdown links, terminology, whitespace and scope; it does not claim application tests.

- [ ] `rtk cargo build --workspace --release --locked` passes with no warnings
- [ ] `rtk cargo fmt --all -- --check` is clean
- [ ] `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` passes
- [ ] `rtk cargo test --workspace --locked` is green
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
- **A mock proves the client, not the contract.** A fake `DesktopHost` proves the consumer; only the real implementation on a real Windows build proves the host.
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
| `cfg(target_os)` or OS checks in UI/Core | Bypasses the capability model; use `PlatformCapabilities` |
| Touching a Slint object off the UI thread | Intermittent crashes; marshal with `slint::invoke_from_event_loop` |
| A slow hook callback | The OS disables the hook; input is affected system-wide |
| Dropping from an elevated process | Blocked by UIPI; document, never escalate |
| Reading icons that trigger OneDrive hydration | Unwanted downloads |

## Before Opening a PR

```bash
rtk cargo build --workspace --release --locked
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
```

A green run of these is not a green release: E2E and the manual checklist are separate.

## PR

**Title:** `feat(core): R1-01 add domain structs`

**Body:** spec refs, what changed and why, **named tests**, DoD checklist, anything waived and who agreed, anything **not determined**.

The last item is not optional. A PR that omits what it could not verify will be trusted more than it should be.

## Rust safety traps

Slint 1.13.1 generated glue requires a private compiler-output unsafe exception (ADR-028); hand-written UI/Core still forbid unsafe. A crate-wide forbid fails E0453. See the [R0-02 record](../MEMORY/records/2026-11-25-R0-02-rust-workspace.md).

Drop/panic hooks do not recover from abort/kill/reboot. Guard readiness and durable intent precede icon hiding; Guard health loss while App lives requires immediate best-effort restoration and no rehide until rearmed. App/Guard/CLI serialize recovery ownership; duplicate startup must not restore a live owner's changes. Restore must not depend on config/log success. Slint owns its HWNDs; shell registrations borrow them and expire before destruction. C# test counts and old measurements never count as Rust evidence.

## Native namespace uncertainty trap

A native replace/rename error does not by itself prove that no namespace mutation occurred. Commit and quarantine contracts must distinguish verified non-mutation from indeterminate outcomes, retain all recovery evidence and prevent automatic retry, backup pruning or temporary cleanup until reconciliation. A revision check is not an OS compare-and-swap against uncooperative external writers. See [ADR-030](../MEMORY/DECISIONS.md) and the [interface review record](../MEMORY/records/2026-11-25-enumeration-persistence-contracts.md); native adapter acceptance remains pending.

## Security scanner compatibility trap

A pinned scanner can become incompatible with its current advisory database before checking any dependency. R0-03 reproduced cargo-deny 0.18.3 rejecting CVSS 4.0 advisory data. Update to an explicitly pinned compatible helper and rerun the complete audit; never delete an advisory or add an ignore solely to make parsing pass. See the [R0-03 record](../MEMORY/records/2026-11-25-R0-03-rust-ci.md).
