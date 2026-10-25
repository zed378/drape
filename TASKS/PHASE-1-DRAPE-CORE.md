# Phase 1 — Core Drape

**Status: TODO** — planning phase; no card in this file has started. The whole phase waits for P0-09 (ADR-007 final).

**Goal:** a user can create and arrange Drapes that hold items, and everything is saved and restored. This phase does not yet change the native desktop (that is Phase 2). The panel is called `Drape` in UI and code alike (ADR-020).

**Prerequisite:** P0-09. **Exit criteria:** see [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md) (Phase 1). Feature references: F-01..F-05 and F-13 in [`../docs/PLAN/02-FEATURE-SPECIFICATION.md`](../docs/PLAN/02-FEATURE-SPECIFICATION.md); data model in [`../docs/ARCHITECTURE/02-DATA-MODEL.md`](../docs/ARCHITECTURE/02-DATA-MODEL.md).

Conventions: [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md).

---

### P1-01 — Core domain model

| | |
|---|---|
| **Status** | DONE (2026-10-23) |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | core-engineer |
| **Depends on** | P0-09 |
| **Spec refs** | [docs/ARCHITECTURE/02-DATA-MODEL.md](../docs/ARCHITECTURE/02-DATA-MODEL.md) · ADR-011 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · ADR-016 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** Every other layer is written against these types. A model that allows invalid states pushes validation into UI code, where it gets skipped.

`Config`, `Page`, `Drape`, `DrapeItem`, `Bounds`, `Style` and `State` as immutable records, with validation and test builders. Names follow ADR-020 and the CS-10 namespace-clash rule. An item belongs to at most one Drape (ADR-011).

**Definition of Done**
- [ ] Every entity matches `docs/ARCHITECTURE/02-DATA-MODEL.md`; validation (minimum size, unique IDs, one item per Drape) is tested including the failing branches
- [ ] `Drape.Core` has no reference to Avalonia, Win32 or any platform API (the P0-01 architecture test stays green)

**Abuse cases**
- Records declared immutable but exposing mutable collections, so tests pass while callers mutate shared state
- Validation tests that only build valid objects and never assert a rejection

---

### P1-02 — Interface contracts in Core

| | |
|---|---|
| **Status** | DONE (2026-10-23) |
| **Priority** | P0 |
| **Estimate** | 1 day |
| **Role** | architect |
| **Depends on** | P0-09 |
| **Spec refs** | [docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md](../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) · ADR-015 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** The contracts are the seam between Core and the platform. A leaky contract (a UI-framework type, an OS check) costs every later platform port.

Define `IConfigStore`, `IDesktopItemSource`, `IDesktopHost`, `IDesktopIcons`, `IIconProvider`, `IDisplayMonitor`, `IClock`, `IFileSystem`, plus `IPlatformCapabilities`, `IDesktopLocations`, `IAutostart`, the `IconBitmap` type (BGRA pixels plus size, no UI-framework dependency) and `IBlurEffect`. Fakes for all of them live in `tests/Drape.TestSupport`.

**Definition of Done**
- [ ] All interfaces have XML documentation, including threading expectations and failure behaviour
- [ ] A fake exists for each interface and each fake is used by at least one named test
- [ ] No contract exposes an Avalonia, WPF or Win32 type
- [ ] `IDesktopIcons` exposes an explicit read of the current state; there is no blind toggle

**Abuse cases**
- Fakes written but never wired into a test, so "fake available" is true and useless
- A contract that returns an Avalonia `Bitmap` "for convenience" because `IconBitmap` was awkward

---

### P1-03 — ConfigStore: atomic save, backup, quarantine, migration

| | |
|---|---|
| **Status** | DONE (2026-10-25) — spec written, 50x50 save measured under 20 ms
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | core-engineer |
| **Depends on** | P1-01, P1-02 |
| **Spec refs** | [docs/ARCHITECTURE/02-DATA-MODEL.md](../docs/ARCHITECTURE/02-DATA-MODEL.md) · ADR-005 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | yes |

**Why:** A config that can be half-written is a config that will one day lose a user's layout.

Implement `IConfigStore` in `Drape.Infrastructure`: write to a temp file, flush, then replace; keep the last 10 backups; quarantine corrupt files; `schemaVersion` with migrations; `JsonExtensionData` so unknown fields survive.

**Definition of Done**
- [ ] A simulated crash in the middle of a write leaves the old config intact
- [ ] A corrupt config is quarantined and the newest valid backup is used
- [ ] A v1 fixture migrates; a config with a newer version opens read-only and is never overwritten
- [ ] Saving 50 Drapes × 50 items takes under 20 ms
- [ ] Feature spec written first in `MEMORY/specs/`
- [ ] Mutation check: remove the replace-step atomicity and watch the crash test fail

**Abuse cases**
- "Atomic" implemented as write-in-place plus a "writing" flag
- A migration test that round-trips the migrator's own output instead of a hand-written fixture per version, so a dropped field is never noticed

---

### P1-04 — DrapeWindow: basic Drape window

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P0-09, P1-02 |
| **Spec refs** | [docs/UI-UX/00-UI-UX-SPEC.md](../docs/UI-UX/00-UI-UX-SPEC.md) · ADR-007 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** This is the first thing the user sees. One Avalonia window per Drape is an architecture rule; bending it here would be copied everywhere.

`DrapeWindow` is an Avalonia window: borderless, with a title, an item area and a transparent background per the spec; placed through `IDesktopHost`; not in the taskbar or Alt+Tab; never takes focus when shown.

**Definition of Done**
- [ ] Shows in the desktop layer on Windows 11 and on Windows 10 22H2 (both required)
- [ ] Does not steal focus and does not appear in Alt+Tab
- [ ] Matches `docs/UI-UX/00-UI-UX-SPEC.md` at 100% and 150% DPI (screenshots attached)
- [ ] The view never calls native APIs; native handle work goes through `IDesktopHost`
- [ ] A headless UI test covers the window's view-model-bound content

**Abuse cases**
- Window "hidden from Alt+Tab" by moving it off-screen or making it 1×1 pixel instead of setting the proper style
- DPI checked only at 100% and reported as "multi-DPI verified"

---

### P1-05 — `IDesktopHost` implementation (from spike P0-03)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P0-09, P1-02 |
| **Spec refs** | [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §2 · ADR-007 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | yes |

**Why:** The host is the part most likely to break on the next Windows build. Production code must degrade, not crash.

Production version of the chosen strategy in `Drape.Shell.Windows`: `Attach`, `Reattach`, `Place`, `EnsureZOrder`, `Detach`, with a degraded mode. Native handles are wrapped in `SafeHandle` or `IDisposable`.

**Definition of Done**
- [ ] Meets the six ADR-007 criteria on every build in the test matrix (each build named)
- [ ] No leaked handle over 1,000 open/close cycles (how it was measured is stated)
- [ ] A host failure produces the degraded mode with a user notification, not a crash
- [ ] The desktop window hierarchy is discovered, not hard-coded to a fixed path
- [ ] Feature spec written first in `MEMORY/specs/`

**Abuse cases**
- The 1,000-cycle test run against the fake host, so it proves the consumer and says nothing about the real one
- The degraded mode tested only by throwing from a fake, never by making the real attach fail

---

### P1-06 — Move and resize a Drape

| | |
|---|---|
| **Status** | DONE for the Core maths; the 144 Hz check needs a view (P1-04) |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P1-04 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-02 |
| **Spec required** | no |

**Why:** Layout maths in the view is untestable. Keeping it in Core makes the clamp rules provable.

Drag by title, edge and corner; clamp to the working area; minimum size. The maths lives in `LayoutService` in Core.

**Definition of Done**
- [ ] `LayoutService` has unit tests for clamp, minimum size and the 40 DIP that must stay visible, including the failure branches
- [ ] Interaction stays smooth at 144 Hz without a CPU spike (measurement and method recorded)

**Abuse cases**
- Clamp logic duplicated in the view while `LayoutService` exists only to satisfy the test
- Smoothness judged by eye on a 60 Hz display

---

### P1-07 — Create, rename and delete a Drape

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P1-04, P1-03 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-01 · [docs/UI-UX/00-UI-UX-SPEC.md](../docs/UI-UX/00-UI-UX-SPEC.md) |
| **Spec required** | no |

**Why:** Deleting a Drape sounds like deleting files. The confirmation and the behaviour must make clear, and make true, that no file is touched.

Commands from the tray and the Drape menu; a unique default name; a delete confirmation that states files are not deleted.

**Definition of Done**
- [ ] A new Drape appears at a sensible position and is saved
- [ ] Deleting a Drape deletes no file; its items return to the ungrouped list (verified by a test that asserts the files still exist)
- [ ] The confirmation text matches `docs/UI-UX/00-UI-UX-SPEC.md`

**Abuse cases**
- The "no file deleted" test uses a fake file system with no delete method, so it cannot fail
- Confirmation text changed to pass review while the underlying delete path is unchanged

---

### P1-08 — Item display: grid, async icons, labels

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P1-04, P1-02 |
| **Spec refs** | [docs/UI-UX/00-UI-UX-SPEC.md](../docs/UI-UX/00-UI-UX-SPEC.md) · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §4 |
| **Spec required** | no |

**Why:** Icons decide whether the panel feels fast. Touching an Avalonia object off the UI thread causes intermittent crashes that are hard to reproduce.

An item grid with light virtualisation; icons loaded asynchronously as `IconBitmap` and converted to Avalonia bitmaps on `Dispatcher.UIThread` (following the P0-05 answer); placeholders; two-line labels; missing items shown dimmed.

**Definition of Done**
- [ ] 200 items in one Drape stay responsive (frame-time method recorded)
- [ ] Icon loading causes no layout jump
- [ ] Missing items are clearly marked
- [ ] No Avalonia object is touched off the UI thread (a test or analyser enforces it)

**Abuse cases**
- Icons loaded synchronously in a build with a warm cache, then reported as "async"
- The 200-item check run with 200 copies of one icon so the cache hides the real cost

---

### P1-09 — Drag-drop to add items; open and remove items

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P1-08 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-03, F-04 · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §6 |
| **Spec required** | no |

**Why:** This is where user files are first touched. Removing an item must never remove the file, and an elevated drop source must fail clearly rather than silently.

Accept `CF_HDROP` and Shell ID lists, reject duplicates, open with `ShellExecute`, and remove the reference without deleting the file.

**Definition of Done**
- [ ] Dropping from Explorer and from the desktop adds items
- [ ] Double-click or Enter opens; Delete removes from the Drape and the file stays on disk
- [ ] A drop from an elevated process fails with a clear message (UIPI); the app does not escalate
- [ ] A duplicate drop is rejected and an item already in another Drape is handled per ADR-011

**Abuse cases**
- "File stays" asserted by checking the in-memory list, never the file system
- The UIPI case skipped because no elevated test source was available, and reported as passing

---

### P1-10 — Multi-monitor and per-monitor DPI

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P1-06, P1-05 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-13 · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §8 |
| **Spec required** | no |

**Why:** Position drift under repeated resolution changes is silent and cumulative; users notice it weeks later.

`IDisplayMonitor`, monitor-relative position storage, relocation when a monitor disappears and returns, and scaling from the saved position (not from the last scaled one).

**Definition of Done**
- [ ] A Core test: 10 resolution-change cycles cause no drift
- [ ] Unplugging and re-plugging a monitor moves the Drape away and back
- [ ] Mixed 100%/150% DPI renders correctly (screenshots attached, setup named)

**Abuse cases**
- Drift test using integer-friendly resolutions that never expose rounding
- Monitor removal simulated only through the fake, never on real hardware or a virtual display

---

### P1-11 — End-to-end persistence and startup reconciliation

| | |
|---|---|
| **Status** | DONE (2026-10-23) |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | core-engineer |
| **Depends on** | P1-03, P1-07, P1-09 |
| **Spec refs** | [docs/PLAN/02-FEATURE-SPECIFICATION.md](../docs/PLAN/02-FEATURE-SPECIFICATION.md) F-05 · [docs/ARCHITECTURE/02-DATA-MODEL.md](../docs/ARCHITECTURE/02-DATA-MODEL.md) |
| **Spec required** | no |

**Why:** Debounced saving is where layouts get lost: a crash inside the debounce window. The shutdown path must flush.

Save with a 500 ms debounce, load at start, validate (orphan Drapes, duplicate items), and save synchronously on shutdown.

**Definition of Done**
- [ ] Closing and reopening the app restores every Drape and item
- [ ] A corrupt config is handled as described in the data model document
- [ ] A change made just before shutdown is persisted (named test)

**Abuse cases**
- Round-trip tested in a single process with the same in-memory objects, never through a real file and a restart
- Debounce tested with a fake clock that never advances, so the save path never runs

---

### P1-12 — Phase 1 testing: unit, integration, manual checklist, performance baseline

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | qa |
| **Depends on** | P1-11 |
| **Spec refs** | [docs/TESTING/00-TESTING-STRATEGY.md](../docs/TESTING/00-TESTING-STRATEGY.md) |
| **Spec required** | no |

**Why:** The Phase 1 exit criteria are claims; this card turns them into named evidence and a baseline Phase 5 can be measured against.

Complete the tests, run the manual checklist items for Phase 1, and measure CPU, memory and startup baselines.

**Definition of Done**
- [ ] Core coverage is at least 80% (report attached)
- [ ] The manual checklist report and the performance baseline are stored in `MEMORY/` or `docs/TESTING/`, naming the build, DPI and monitor setup
- [ ] Bugs found are in `BACKLOG.md` with a severity

**Abuse cases**
- Coverage reached with tests that execute code but assert nothing
- Bugs fixed silently or left unlogged so the backlog shows a clean phase
