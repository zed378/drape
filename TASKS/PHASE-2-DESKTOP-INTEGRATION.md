# Phase 2 — Desktop Integration (MVP)

> Estimates in retained cards are provisional legacy planning allowances, not Rust delivery commitments. Re-estimate/split at claim time after the Rust spikes; follow actual per-card prerequisites.

> Rust + Slint target (ADR-025). R0/R1 dependencies refer to [Rust migration cards](./RUST-MIGRATION.md), not historical P0/P1 results. These implementation tasks remain unstarted; no code is authorised by the documentation update.

**Status: TODO** — Rust target work not started. The phase waits for R1-06 and R0-10 (plus per-card prerequisites).

**Goal:** Drape replaces the desktop icons safely: native icons are hidden and always restored, the desktop stays in sync, and quick-hide works.

**Exit criteria:** see [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md) (Phase 2). Feature references: F-06..F-12 in [`../docs/PLAN/02-FEATURE-SPECIFICATION.md`](../docs/PLAN/02-FEATURE-SPECIFICATION.md); ADR-006 and ADR-008 in [`../MEMORY/DECISIONS.md`](../MEMORY/DECISIONS.md); [`../docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md).

> **The desktop-safety rules in `CLAUDE.md` and [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md) apply in full and are unwaivable. Do not mark Phase 2 done before every icon-restore path (normal exit, exception, kill, Guard, next start, `--restore-desktop`, forced reboot) has been tested and the test is named in a record.** No test or script may kill `explorer.exe`; Explorer-restart tests are manual, in a VM or test account.

---

### P2-01 — Production `DesktopIcons`: read state, hide, show

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | shell-engineer-windows |
| **Depends on** | R0-10 |
| **Spec refs** | ADR-006 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §3 |
| **Spec required** | yes |

**Why:** A wrong state read flips the user's desktop. This is the call that can strand a user, so it must never behave like a blind toggle.

Implement the technique chosen in R0-05 / ADR-006 with state read before and after, detection of changes made by the user, and no accidental inversion.

**Definition of Done**
- [ ] Idempotent: calling `Hide` twice does not flip the state
- [ ] Detects when the user changes "Show desktop icons" themselves
- [ ] Tested on Windows 11 and on Windows 10 22H2 (builds named); icon restore proven on both
- [ ] The state is read back after every change and a mismatch is reported
- [ ] Feature spec written first in `MEMORY/specs/`
- [ ] Mutation check: replace the read with a blind toggle and watch the idempotency test fail

**Abuse cases**
- Idempotency tested only against a fake that stores a boolean, never against the real shell state
- Detection of user changes implemented as polling the app's own last-written value rather than the real state

---

### P2-02 — `state.json` and recovery at next start

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 1 day |
| **Role** | core-engineer |
| **Depends on** | P2-01 |
| **Spec refs** | [docs/ARCHITECTURE/02-DATA-MODEL.md](../docs/ARCHITECTURE/02-DATA-MODEL.md) · ADR-008 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** If the app dies with icons hidden, the next start is the second line of defence. A marker that is never cleared would restore icons the user chose to hide.

Version and atomically persist original visibility, change ownership/intent and validated OS process identity in state.json before mutation. Arm and acknowledge Guard before hiding; verify state afterwards. On next start, recover owned unfinished changes, including compatible legacy records. PID alone is insufficient. Clear recovery state only after verified restoration.

**Definition of Done**
- [ ] A forced reboot while icons are hidden: the next start restores them
- [ ] Recovery ownership is cleared only after restoration succeeds; failure retains retryable state
- [ ] A stale PID that now belongs to another process is not mistaken for a live Drape (tested)
- [ ] `state.json` is written atomically (same mechanism as the config store)

**Abuse cases**
- Reboot recovery simulated by deleting the process from a fake list instead of an actual forced reboot or kill
- The flag cleared at the start of shutdown, before the icons are actually restored

---

### P2-03 — Production `--restore-desktop`

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 1 day |
| **Role** | shell-engineer-windows |
| **Depends on** | P2-01 |
| **Spec refs** | ADR-006 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · ADR-008 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §3 |
| **Spec required** | no |

**Why:** This is the user's manual escape hatch. It must work when nothing else does, including when called by the installer or uninstaller.

A renderer-independent CLI mode acquires/coordinates the per-session recovery protocol, verifies active generation/process identity, asks a live App to suspend rehide and restores owned changes idempotently. It does not blindly flip unknown state or require a healthy Slint startup.

**Definition of Done**
- [ ] Runs without the main app running
- [ ] Has no effect when the icons are already shown (tested)
- [ ] Exit code is clear (0 success, non-zero failure) and the run is logged
- [ ] Works non-elevated
- [ ] Verified after a hard kill of the main app (procedure and build named)

**Abuse cases**
- The command works only because the main app is still running and holds a handle
- Exit code always 0, so a failed restore looks like success to the installer

---

### P2-04 — `drape-guard`

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P2-01, P2-02 |
| **Spec refs** | ADR-008 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · [docs/SECURITY/00-SECURITY-AND-PRIVACY.md](../docs/SECURITY/00-SECURITY-AND-PRIVACY.md) · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) |
| **Spec required** | yes |

**Why:** In-process handlers die with the process. The Guard is the only recovery path that survives a hard kill.

A small UI-less process that waits on the validated main process handle and restores owned icon changes when it ends. Durably record intent and launch/acknowledge Guard before hiding icons. Guard has no Slint/renderer dependency and is non-elevated, like App.

**Definition of Done**
- [ ] Kill `drape-app` and the icons come back in under 2 seconds (measured; method stated)
- [ ] Guard exits only after verified restoration or explicit disarm following successful App restoration; failures retain retryable state
- [ ] Guard launch/readiness failure keeps native icons visible; App may continue with a clear warning
- [ ] Guard preserves icons already hidden by the user and never toggles icons already visible; runtime user changes are reconciled without fighting them
- [ ] Guard death while App is alive triggers prompt owned-state restore and disables rehide until a new Guard is authenticated/ready
- [ ] Test readiness-before-hide race, stale/replayed IPC, simultaneous startup/restore/shutdown, corrupt state and hung/missing Explorer; no blind toggle or false success
- [ ] Feature spec written first in `MEMORY/specs/`

**Abuse cases**
- The Guard is a thread or task inside the main process, so a kill takes it down too
- The 2-second claim measured with a graceful close instead of a process kill

---

### P2-05 — Crash/exception recovery path and tray button

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 1 day |
| **Role** | ui-engineer |
| **Depends on** | P2-01 |
| **Spec refs** | ADR-008 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · [docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md](../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) |
| **Spec required** | no |

**Why:** An unhandled exception is the most common way an app ends unexpectedly. It must not leave the user's desktop empty.

Controlled error/panic paths attempt best-effort restoration; independent Guard handles abort/kill, durable state handles next start, and standalone --restore-desktop bypasses renderer startup. The tray exposes restoration. Never unwind across FFI or assume Drop/panic hooks run on every exit.

**Definition of Done**
- [ ] A controlled Rust UI error/panic has a documented best-effort restore path; if in-process recovery cannot complete, Guard performs recovery within the measured acceptance window
- [ ] A worker error/panic is surfaced to lifecycle handling; abort/forced kill is recovered independently by Guard, not assumed to unwind
- [ ] The tray button restores the icons and gives visible feedback
- [ ] Recovery failure returns an explicit result, cannot unwind across FFI, and retains Guard/durable retry responsibility (tested)

**Abuse cases**
- Only controlled UI errors are covered; worker panic, abort and forced kill are untested
- The restore runs after the process has started tearing down, and "worked on my machine" because timing was lucky

---

### P2-06 — Desktop watching and reconciliation

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | core-engineer |
| **Depends on** | R1-05, P2-13 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-07 · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §4 · ADR-006 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** The desktop changes constantly and from outside. Without batching and rename tracking, a Drape either thrashes or loses its members.

Production `DesktopItemSource` over Desktop plus Public Desktop (locations from `DesktopLocations`), a watcher with a 250 ms debounce, batching, rename tracking by file ID, and a grace period for missing items.

**Definition of Done**
- [ ] 1,000 consecutive changes produce only a few batches
- [ ] A rename within the same volume keeps Drape membership
- [ ] A OneDrive-redirected Desktop is tested (setup named)
- [ ] Locations come from `DesktopLocations`, never from a hard-coded path

**Abuse cases**
- Debounce tested with a fake clock and a fake watcher, never with a real directory under load
- Rename tracking tested by renaming within the fake item source rather than on a real volume

---

### P2-07 — Inbox and placement of new items

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | core-engineer |
| **Depends on** | P2-06 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-08 |
| **Spec required** | no |

**Why:** With native icons hidden, a new file that lands nowhere is invisible to the user. The Inbox is what keeps new files findable.

An Inbox Drape per page; new items land in the Inbox (or in a Drape chosen by rules later); an option to turn the Inbox off.

**Definition of Done**
- [ ] A new file on the desktop appears in the Inbox in under 2 seconds
- [ ] The Inbox cannot be deleted while sync is active
- [ ] With the Inbox turned off, new items remain reachable (spec behaviour tested)

**Abuse cases**
- The 2-second figure measured from the watcher event rather than from the file being created
- The Inbox hidden from the delete menu in the UI only, while the Core model still allows deleting it

---

### P2-08 — Quick-hide (double-click on the desktop)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-windows |
| **Depends on** | R0-08, P2-01 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-09 · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §7 · [docs/SECURITY/00-SECURITY-AND-PRIVACY.md](../docs/SECURITY/00-SECURITY-AND-PRIVACY.md) · ADR-010 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | yes |

**Why:** A global hook that is slow, or that leaks input, harms the whole system and is exactly what users distrust.

Production `InputHook` on a dedicated thread; empty-desktop hit-test; toggle Drape visibility; per-Drape exclusion; 150 ms fade that respects the system animation setting. The hook callback stays minimal and stores no input.

**Definition of Done**
- [ ] Does not trigger over a native item, a Drape, or another window
- [ ] The normal Windows right-click menu still works
- [ ] The hook is fully removed when the feature is turned off (verified by checking the handle is released)
- [ ] The callback performs no allocation of note, no I/O and no long lock; latency measured end to end
- [ ] Feature spec written first in `MEMORY/specs/`

**Abuse cases**
- The feature is "off" in settings but the hook stays installed and ignores events
- Hit-testing validated only on an empty wallpaper, never over icons, Drapes or a maximised window

---

### P2-09 — Create a Drape by right-drag

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P2-08 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-10 |
| **Spec required** | no |

**Why:** If the gesture steals the right-click, the Windows context menu breaks for everyone. The gesture has to be additive.

A rectangle preview while right-dragging; on release a "Create Drape here" menu; a plain right-click is untouched.

**Definition of Done**
- [ ] The Drape is created exactly on the drawn area
- [ ] A right-click without dragging still opens the Windows menu
- [ ] A very short drag (below the threshold) is treated as a click

**Abuse cases**
- The drag threshold set so high that the gesture rarely triggers, which makes the "click unaffected" check trivially pass
- Menu suppression implemented globally rather than only during an actual drag

---

### P2-10 — Session resilience: Explorer restart, Win+D, sleep/resume, lock, display change

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-windows |
| **Depends on** | R1-03, P2-01 |
| **Spec refs** | [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §10 · ADR-007 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · ADR-006 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** Explorer restarts and session changes happen to every user. Each one can drop the host or reset the icon state.

Handle verified native session/power/display notifications behind the shell adapter: registered `TaskbarCreated`, WTS session notifications, `WM_POWERBROADCAST` and display/DPI messages as supported by pinned bindings. These are not .NET `SessionSwitch`/`PowerModeChanged` callbacks. Invalidate stale host generations and verify ownership/icon state before reattachment or any rehide.

**Definition of Done**
- [ ] Restarting `explorer.exe` (manually, in a VM or test account) restores the Drapes and the icon state
- [ ] Win+D does not hide the Drapes
- [ ] After sleep/resume and unlock, layout and icon state are correct
- [ ] No script or automated test kills `explorer.exe`
- [ ] Each event is covered by a named manual-checklist entry with the build used

**Abuse cases**
- The Explorer restart is simulated by sending the `TaskbarCreated` message, which does not prove recovery from a real restart
- Only one monitor and one DPI were used for the display-change case

---

### P2-11 — Autostart and basic settings

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | R0-09, P2-13 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-12 · [docs/UI-UX/00-UI-UX-SPEC.md](../docs/UI-UX/00-UI-UX-SPEC.md) |
| **Spec required** | no |

**Why:** Autostart written to the wrong place, or one that needs admin, is a support burden and a trust problem.

Autostart via `Autostart`; settings for "Hide desktop icons", Inbox, quick-hide and language; a minimal settings window.

**Definition of Done**
- [ ] Autostart can be turned on and off, needs no admin, and goes through `Autostart` only
- [ ] Settings changes are saved and apply immediately
- [ ] The settings UI does not call any native or registry API directly
- [ ] Turning off "Hide desktop icons" restores the icons

**Abuse cases**
- The view-model writes the registry key directly because `Autostart` was "not ready yet"
- "Applies immediately" shown only for the toggle's own state, not for the behaviour it controls

---

### P2-12 — MVP testing and internal alpha release

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | qa |
| **Depends on** | P2-01, P2-02, P2-03, P2-04, P2-05, P2-06, P2-07, P2-08, P2-09, P2-10, P2-11, P2-13 |
| **Spec refs** | [docs/TESTING/00-TESTING-STRATEGY.md](../docs/TESTING/00-TESTING-STRATEGY.md) · [docs/PLAN/01-PRODUCT-REQUIREMENTS.md](../docs/PLAN/01-PRODUCT-REQUIREMENTS.md) (MVP criteria) |
| **Spec required** | no |

**Why:** The MVP is only done when someone tried to break it. The recovery paths are the part most likely to be claimed and least likely to be tried.

Run the full manual checklist (kill, forced reboot, Explorer restart, display changes, OneDrive), performance tests, and verify the five MVP acceptance criteria.

**Definition of Done**
- [ ] All five MVP criteria in `docs/PLAN/01-PRODUCT-REQUIREMENTS.md` are met and documented with named tests or checklist entries
- [ ] Every icon-restore path (normal exit, exception, kill, Guard, next start, `--restore-desktop`, forced reboot) has a recorded test result
- [ ] No blocker or major bug open in safety, data durability, accessibility or any required MVP functionality, regardless of component label
- [ ] An internal alpha build exists; known limitations are listed
- [ ] The results name the Windows builds, DPI settings and monitor setups used

**Abuse cases**
- A restore path left out of the report because it "obviously works"
- The alpha built and handed out while a major Shell bug sits in the backlog marked as minor

---

### P2-13 — Windows `DesktopLocations` and `Autostart`

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 1 day |
| **Role** | shell-engineer-windows |
| **Depends on** | R1-01 |
| **Spec refs** | ADR-015 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §4 · [docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md](../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) |
| **Spec required** | no |

**Why:** Desktop paths and autostart are platform facts. Putting them behind interfaces keeps Core and UI portable, and using the Known Folder API keeps them right on machines where the Desktop is redirected.

Implement the Windows versions of `DesktopLocations` (Known Folder API: `FOLDERID_Desktop` and `FOLDERID_PublicDesktop`) and `Autostart` (the `HKCU` Run key) in `drape-shell-windows`, behind the Core interfaces.

**Definition of Done**
- [ ] `DesktopLocations` returns the correct Desktop on a OneDrive-redirected machine (test or manual run, setup named)
- [ ] `Autostart` enables, disables and reports the state through `HKCU` only; no admin is required (tested non-elevated)
- [ ] COM and native memory are released (no leak; RAII wrappers with the matching native release API used)
- [ ] The Windows implementations are registered only by the composition root; no other project references them directly

**Abuse cases**
- Hard-coding `%USERPROFILE%\Desktop` and testing only on a machine without OneDrive redirection
- `Autostart` writing to `HKLM` or a Startup shortcut folder with elevated rights during testing, so "no admin" is never actually exercised
