# <Task ID> — <Title>

**Date:** YYYY-MM-DD
**Task:** <ID from TASKS/, or "—" if this is not task work>
**Author:** <who>
**Spec refs:** <docs/ documents this implements>

> Save to `MEMORY/records/YYYY-MM-DD-<task-id>-<slug>.md`. Fill in every section; "Not applicable" is valid, blank is not.

---

## Hook

*One or two sentences that tell a future reader whether this is the record they need. Not a summary: the surprising part.*

*Good: "Restoring icons after a killed process worked only when the Guard was started before the mutex was taken, so a second instance silently hid the icons for good."*

*Bad: "Implemented the Guard."*

---

## What Changed

*Files, projects, settings, config fields. Enough for someone to find it. Not a diff: git has the diff.*

---

## Why

*The reasoning. What problem this solves, and why this shape rather than another.*

*If `docs/` already decided this, link and say nothing more. If `docs/` left it open, the decision goes here and probably needs an ADR.*

---

## Alternatives Considered

*What was rejected, and why. This is the section a future reader needs most. "None" is valid only when there was genuinely one obvious way.*

---

## What Surprised Me

*The part that cost the most time: a trap, a wrong assumption, behaviour that differed from the documentation (including Windows behaviour that differed from what `docs/PLATFORM/` says). A record with this section empty is usually a record written a week late.*

---

## Tests

*Name them. An assertion that something was tested is not evidence.*

- Unit:
- Integration:
- UI / headless:
- E2E (FlaUI) / manual checklist items run:
- **Desktop-safety test** (required for anything that hides icons, hosts a window or installs a hook): which test, and the result
- **Mutation check** (did you break the thing and watch the right test fail?):

---

## Platform Matrix

*Where this was run. If it was not run on something, say so in "What Was Not Determined".*

| Windows build | DPI scale | Monitors | Result |
|---|---|---|---|
| e.g. 11 24H2 | 100% | 1 | |
| | 150% | 2, mixed DPI | |

*Other platforms (macOS, Linux, Android, iOS): "not applicable" until Phase 6.*

---

## Definition of Done

*Per `TASKS/00-TASK-CONVENTIONS.md`. Tick what passed. **Name what was waived and who agreed.***

- [ ] Meets the card's acceptance criteria
- [ ] Build with warnings as errors, `dotnet format --verify-no-changes` clean, tests green
- [ ] Architecture rules respected (Core free of platform references, native calls only in Shell, handles wrapped)
- [ ] Desktop-safety rules respected (see below)
- [ ] Tested on Windows 11 and at least one DPI other than 100%
- [ ] Documentation and `TASKS/PROGRESS.md` updated
- [ ] ADR added for any decision
- [ ] This record written

**Waived:** <which item, by whom, why, or "none">

---

## Desktop Safety

*Anything touching hiding or restoring icons, the host window, handles, hooks or `state.json`.*

- **Icon hide path** (what can leave icons hidden):
- **Restore paths exercised** (normal exit / unhandled exception / Guard / next start / `--restore-desktop`):
- **Handles** (every native handle wrapped in `SafeHandle` or `IDisposable`?):
- **Hook callback** (stays minimal: no large allocation, no I/O, no long lock?):
- **Capability gating** (new behaviour behind `IPlatformCapabilities`, not an OS check?):
- **Config** (schema version bump? migration and fixture added? atomic write unchanged?):
- **Privacy** (new network call, new data collected, anything written to logs that is a user path?):

*If nothing, write "no desktop-safety-relevant change", do not leave it blank.*

---

## What to Watch

*What might break because of this. What a future reader should check before assuming this still works. Examples: a new Windows build, mixed DPI, an Explorer restart, a second monitor being unplugged, an antivirus flagging the hook.*

---

## What Was Not Determined

*Stated plainly: something you could not verify, a test you could not run, an environment you had no access to (an ARM64 device, a Windows 10 machine, a second monitor).*

*Rounding this up is how a status report becomes untrustworthy.*

---

## Follow-Ups

*Anything left. Add each to `TASKS/BACKLOG.md` and link it here.*

---

## Links

- ADR:
- Changelog:
- Backlog:
- Related records:
