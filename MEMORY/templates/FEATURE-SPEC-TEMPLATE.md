# Feature Spec — <Task ID> <Title>

**Written:** YYYY-MM-DD — **before** implementation
**Task:** <ID>
**Author:** <who>
**Spec refs:** <docs/ documents this implements>

> Save to `MEMORY/specs/<task-id>-<slug>.md`.
>
> Required for any task marked `Spec required: yes` in `TASKS/`. Written **before** code; its purpose is to surface decisions and traps while they are still cheap. Reading `docs/` carefully first has repeatedly found contradictions that would otherwise have become defects.

---

## Problem

*What is not possible today, and for whom. Name the persona from `docs/PLAN/01-PRODUCT-REQUIREMENTS.md` where one applies.*

---

## What `docs/` Already Decides

*Read every document in Spec refs and list what is already settled. Do not re-decide it.*

| Decision | Source |
|---|---|
| | |

**If `docs/` is silent, contradictory or wrong about something you need, stop and say so here.** That is a specification gap; it belongs in `TASKS/BACKLOG.md` and probably in an ADR before any code is written.

---

## Layering and Interfaces

- [ ] Which project owns each piece (`Drape.Core`, `Infrastructure`, `Shell.Windows`, `UI`, `App.Desktop`, `Guard`)?
- [ ] New or changed interfaces in Core (name, members). Core stays free of platform, UI and direct I/O references.
- [ ] Does anything cross a boundary with a platform type (`HWND`, `ImageSource`)? It must not.
- [ ] Is any native handle created? Wrapped in `SafeHandle` / `IDisposable`, and who disposes it?
- [ ] Is behaviour gated by `IPlatformCapabilities` rather than an OS check?

---

## Desktop Safety

- [ ] **Icon hide path:** can this feature leave native icons hidden? Under what failure?
- [ ] **Restore paths:** normal exit / unhandled exception / `Drape.Guard` / next start via `state.json` / `--restore-desktop`. Which does this feature touch, and how is each tested?
- [ ] Non-destructive: does anything move, rename or delete a user file? It needs explicit confirmation or must not exist.
- [ ] **Hook minimalism** (if a global hook is involved): the callback does no large allocation, no I/O and takes no long lock; the work moves to another thread. Time budget?
- [ ] Explorer restart, Win+D / peek, display change, session lock: what happens to this feature?
- [ ] Failure mode: when the native call fails, does the app degrade (degraded host mode, warning) rather than crash or leave a half-state?

---

## Data and Config

| Field | Type | Default | Notes |
|---|---|---|---|

- [ ] Does it change the config schema? Schema version bump, migration step and a fixture per old version?
- [ ] Writes stay atomic (temporary file, flush, replace); unknown properties preserved?
- [ ] Is anything derived stored where it can disagree with its inputs?
- [ ] Does it touch `state.json`?
- [ ] Is a Drape item still in exactly one Drape (ADR-011)?

---

## Behaviour

*Exact states and transitions, including illegal ones and what happens on them. For each rule, its **enforcement point**: type system, validation in Core, test, or convention. A rule with no enforcement point is an aspiration.*

---

## Platform Matrix

| Target | Applies? | Notes / how verified |
|---|---|---|
| Windows 11 (23H2, 24H2+) | | |
| Windows 10 22H2 (minimum) | | |
| DPI 100% and one of 125/150/200% | | |
| Multiple monitors, mixed DPI, monitor unplug | | |
| macOS / Linux / Android / iOS | not applicable until Phase 6 | |

---

## Security and Privacy

- [ ] Any network call? Default is none (ADR-010); update checks are opt-in (ADR-017).
- [ ] Any new data written to disk or logs; does it include user file paths or names?
- [ ] Any process, DLL or injection into another process? Not allowed (ADR-006).
- [ ] Any elevation needed? The app stays non-elevated (ADR-008).

---

## UI

*Screens and states; accessibility (keyboard, high contrast, screen reader names); DPI behaviour; id-ID and en-US strings.*

- [ ] Loading, empty and failed states?
- [ ] Works with the keyboard alone and at high contrast?
- [ ] All strings in resources, both languages?

---

## Tests

*Name them, before writing them.*

- Unit (Core, written first):
- Integration (Infrastructure, with a fake file system / `FakeDesktopHost`):
- UI / headless:
- E2E (FlaUI) or manual checklist items from `docs/TESTING/00-TESTING-STRATEGY.md`:
- Negative cases (the positive case alone proves little):
- Mutation check planned (which test should fail if the safety rule is broken):

---

## What Was Not Determined

*What this spec cannot settle before code or a spike: unknown Windows behaviour, a performance budget with no measurement, an owner question. Each links to a `Q-n` or a spike task.*

---

## Open Questions

*Raised with the owner via `TASKS/BACKLOG.md`. Each carries a default assumption so work is not blocked.*

| Question | Default assumption | Backlog entry |
|---|---|---|
