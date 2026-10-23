# Phase 0 — Foundation and Spikes

**Status: TODO** — planning phase; no card in this file has started and no code exists yet.

**Goal:** the repository is ready to work in, CI is green, and the biggest Windows risks are answered with evidence rather than assumption. Every spike is done on a separate branch or project under `spikes/`, and its result is written down as an ADR. Spike code does not automatically become production code.

**Exit criteria:** see [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md) (Phase 0).

Conventions (card anatomy, statuses, global Definition of Done): [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md). A spike result names the Windows build, DPI and monitor setup it was measured on; "works on Windows 11" is not evidence.

---

### P0-01 — Scaffold repo and Avalonia solution

| | |
|---|---|
| **Status** | WIP — solution, layering test and `IPlatformCapabilities` skeleton in place; whole-solution test run, `dotnet format` and NuGet version re-check outstanding ([record](../MEMORY/records/2026-10-23-P0-01-solution-scaffold-and-platform-capabilities.md)) |
| **Priority** | P0 |
| **Estimate** | 1 day |
| **Role** | architect |
| **Depends on** | — |
| **Spec refs** | [docs/ARCHITECTURE/01-TECH-STACK.md](../docs/ARCHITECTURE/01-TECH-STACK.md) · [docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md](../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) · [docs/ENGINEERING/00-CODING-STANDARDS.md](../docs/ENGINEERING/00-CODING-STANDARDS.md) · ADR-012 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · ADR-015 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** Every later card builds on this layout. If the dependency direction is wrong or unenforced here, it erodes quietly and is expensive to repair after Phase 1.

Git is already initialised (remote `origin`, owner-created) and a basic `.gitignore` exists; extend it for the .NET tooling in use, and add `.editorconfig`, `global.json`, `Directory.Build.props`, `Directory.Packages.props` and `Drape.sln`. Create the projects in the target structure: `src/Drape.Core`, `Drape.Infrastructure`, `Drape.Shell.Windows`, `Drape.UI` (Avalonia), `Drape.App.Desktop`, `Drape.Guard`; `tests/Drape.Core.Tests`, `Drape.Infrastructure.Tests`, `Drape.UI.Tests`, `Drape.E2E`, `Drape.TestSupport`; plus `spikes/` and `.github/`. Verify the latest LTS .NET and package versions on NuGet (not from memory). Add `app.manifest` (PerMonitorV2, longPathAware, asInvoker) to the app and Guard projects.

**Decision to make and record as an ADR in `MEMORY/DECISIONS.md`:** how `Drape.App.Desktop` references the platform Shell project. Options: (a) multi-target the app, (b) conditional `ProjectReference` selected by OS, (c) one thin head per OS. State the chosen option, the alternatives, and the cost of changing it when Phase 6 adds macOS and Linux.

**Define the `IPlatformCapabilities` skeleton** in `Drape.Core` (per ADR-015): a read-only description of what the current platform supports (for example desktop-layer hosting, icon hiding, global input hook, blur kinds), with a fake in `Drape.TestSupport`. No `OperatingSystem.IsWindows()` calls outside `Shell.*` and the composition root.

**Definition of Done**
- [ ] `dotnet build Drape.sln -c Release -warnaserror` and `dotnet test Drape.sln` pass with no warnings on a clean machine
- [ ] An architecture test (named in the PR) fails if `Drape.Core`, `Drape.UI` or `Drape.Infrastructure` references any `Drape.Shell.*` project, `System.Windows` or `Microsoft.Win32`; and fails if `Drape.Core` references Avalonia
- [ ] Mutation check: add a forbidden reference locally, watch the architecture test fail, then remove it
- [ ] The App-to-Shell referencing decision is recorded as an ADR with alternatives and the bad consequences
- [ ] `IPlatformCapabilities` skeleton exists in Core with a fake in `tests/Drape.TestSupport`
- [ ] Root `README.md` and the MIT `LICENSE` (both already created on 2026-10-10) are kept accurate; the repository, its `origin` remote and the first (documentation) commit already exist (ADR-022, Q-10)
- [ ] Package versions were checked on NuGet and the check is noted in the record

**Abuse cases**
- The architecture test asserts only that the assemblies load, or scans a hand-written allow-list that is never compared with the real project references
- Empty projects are added "to satisfy the structure" while forbidden references are suppressed with a warning exclusion

---

### P0-02 — CI on GitHub Actions

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 1 day |
| **Role** | release-engineer |
| **Depends on** | P0-01 |
| **Spec refs** | [docs/DEVOPS/00-BUILD-AND-RELEASE.md](../docs/DEVOPS/00-BUILD-AND-RELEASE.md) · [docs/TESTING/00-TESTING-STRATEGY.md](../docs/TESTING/00-TESTING-STRATEGY.md) |
| **Spec required** | no |

**Why:** A gate that nobody has seen fail is not a gate. CI must prove it rejects bad changes before anyone depends on it.

Workflow `ci.yml`: locked restore, build with `-warnaserror`, format check, tests with coverage, secret scanning and package vulnerability scan. Dependabot.

**Definition of Done**
- [ ] A PR triggers CI and passes on a Windows runner
- [ ] A deliberate format violation and a deliberate warning each make CI fail (both demonstrated, with the failed run linked)
- [ ] Coverage is published as an artifact
- [ ] No test or script in CI starts or kills `explorer.exe`

**Abuse cases**
- Steps marked `continue-on-error`, or the format and vulnerability steps run but never gate the job
- Coverage reported from an empty test set so the number looks healthy

---

### P0-03 — Spike: window host in the desktop layer

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P0-01 |
| **Spec refs** | [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §1–2 · ADR-007 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · [docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md](../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) |
| **Spec required** | yes |

**Why:** If a transparent Avalonia window cannot live at the desktop layer on current Windows builds, the product design changes. Everything in Phases 1 and 2 stands on this answer.

Prototype strategies A, B and C for placing a transparent **Avalonia** window in the desktop layer. Obtain the native handle with `TryGetPlatformHandle()` and do all re-parenting and z-order work in `Drape.Shell.Windows`, not in the view. Test on Windows 11 (23H2 and 24H2+) and Windows 10 22H2 (VM; the supported minimum, ADR-023, so a failure there is a failure).

**Definition of Done**
- [ ] A results table per strategy × Windows build, scored against the six criteria in ADR-007, each cell naming the build number, DPI and monitor setup
- [ ] Win+D, Explorer restart, 150% and 200% DPI, and two monitors with different DPI were each tested
- [ ] Recommended strategy plus a fallback mode, and a draft `IDesktopHost` contract
- [ ] Summary written to the record and `MEMORY/` lessons; spike code lives only in `spikes/`
- [ ] Feature spec written first in `MEMORY/specs/`

**Abuse cases**
- "Passes" claimed after a window renders on the developer's machine; rendering is not meeting the six criteria on every build in the matrix
- The Explorer-restart case tested by killing `explorer.exe` in a script on the host machine instead of manually in a VM, or skipped and reported as "not applicable"

---

### P0-04 — Spike: hide/show native icons and recovery

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P0-01 |
| **Spec refs** | [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §3 · ADR-006 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · ADR-008 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | yes |

**Why:** Hiding the user's desktop icons is the most dangerous thing this product does. A wrong state read flips the user's desktop; a missed restore strands them.

Find the most stable way to read the current state, hide and show desktop icons; prove recovery after the process is killed. Prototype `--restore-desktop`.

**Definition of Done**
- [ ] Read-state and set-state are reliable on every tested build and never flip the state by mistake (a read of the wrong state is tested explicitly)
- [ ] Kill the process, then `--restore-desktop` brings the icons back (procedure and build recorded)
- [ ] Alternatives (SW_HIDE on the list view, registry) evaluated and the reasons written down
- [ ] ADR-006 is amended with the chosen technique (by P0-09; this card supplies the evidence)
- [ ] Feature spec written first in `MEMORY/specs/`

**Abuse cases**
- Recovery "verified" only on the exit path where the process exits cleanly, never after a hard kill or a forced reboot
- The toggle treated as a setter: the spike flips the state without reading it first and works only because the starting state happened to be "shown"

---

### P0-05 — Spike: enumerate items and icons via the shell

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P0-01 |
| **Spec refs** | [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §4 · ADR-015 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** Phase 1 shows icons for up to hundreds of items. Slow or hydration-triggering icon reads would ruin the first impression and could download a user's cloud files.

Enumerate Desktop plus Public Desktop through the Known Folder API, and fetch icons/thumbnails with `IShellItemImageFactory`. Icons are returned as a neutral `IconBitmap` (BGRA pixels plus size), not a UI-framework image type. Measure speed with 500 items, quality, overlays, and OneDrive Files On-Demand behaviour (without triggering downloads). **Answer explicitly:** can Avalonia bitmaps be created off the UI thread, or must `IconBitmap` be converted on `Dispatcher.UIThread`?

**Definition of Done**
- [ ] 500 items load in under 1 second (icons asynchronous) without blocking the UI thread
- [ ] No cloud-file hydration is triggered (checked with a Files On-Demand placeholder)
- [ ] The Avalonia off-UI-thread bitmap question is answered with a test or documented source, not an opinion
- [ ] Cache decision (size, LRU) and overlay handling are recorded
- [ ] The Desktop path is resolved through the Known Folder API; `%USERPROFILE%\Desktop` is not used

**Abuse cases**
- The 500-item timing measured with a warm shell cache on a folder of identical `.txt` files, then presented as the real-world number
- Hydration checked by file size on disk rather than by observing whether a download actually started

---

### P0-06 — Spike: transparency and blur on Avalonia

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P0-01 |
| **Spec refs** | [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §9 · [docs/UI-UX/00-UI-UX-SPEC.md](../docs/UI-UX/00-UI-UX-SPEC.md) |
| **Spec required** | no |

**Why:** The panel look is the product. Blur is also the cheapest way to blow the idle-CPU budget, so the cost must be known before the style system is designed.

Compare Avalonia's transparency-level hint (transparent and blur-behind), acrylic or Mica, and the Windows 11 DWM backdrop. Measure CPU/GPU at idle and while dragging; test high-contrast mode.

**Definition of Done**
- [ ] Benchmark with 20 Drape windows; the numbers (and the machine) are recorded
- [ ] An `IBlurEffect` contract with up to three implementations (or a written reason for rejecting it) and a solid-colour fallback
- [ ] Rounded-corner behaviour on Windows 10 and 11 documented
- [ ] High-contrast mode behaviour checked and noted

**Abuse cases**
- The benchmark run with one window and extrapolated to 20
- The fallback "implemented" as the same blur code path with a different brush, so it is never exercised when blur is unavailable

---

### P0-07 — Spike: mouse hook (desktop double-click and right-drag)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P0-01 |
| **Spec refs** | [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §7 · [docs/SECURITY/00-SECURITY-AND-PRIVACY.md](../docs/SECURITY/00-SECURITY-AND-PRIVACY.md) · ADR-010 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | yes |

**Why:** A slow `WH_MOUSE_LL` callback gets the hook disabled by the OS and affects input system-wide. It is also what antivirus products look at first.

Prototype `WH_MOUSE_LL` on a dedicated thread, "empty desktop area" hit-testing, and right-drag without breaking the native context menu. The callback must be minimal: no large allocation, no I/O, no long lock; work is handed to another thread. No input is stored.

**Definition of Done**
- [ ] A double-click on empty desktop is detected; on a native icon, a Drape or another window it is not
- [ ] A plain right-click still shows the Windows menu
- [ ] Callback latency measured (average under 1 ms), the method stated; no input is stored or logged
- [ ] Antivirus risk note written
- [ ] Feature spec written first in `MEMORY/specs/`

**Abuse cases**
- Latency measured on the handler's inner work instead of end-to-end from the OS callback entry to return
- Debug logging of mouse coordinates left in the callback and described as "temporary"

---

### P0-08 — App skeleton: composition root, DI, logging, single instance, tray

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P0-01 |
| **Spec refs** | [docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md](../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) · [docs/ARCHITECTURE/01-TECH-STACK.md](../docs/ARCHITECTURE/01-TECH-STACK.md) · ADR-008 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · ADR-012 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) |
| **Spec required** | no |

**Why:** This is the composition root every later card plugs into. Baseline CPU and memory measured now are the only way to notice later regressions (PR-5).

`Drape.App.Desktop` as the composition root: Generic Host together with the Avalonia application lifetime, an Avalonia `TrayIcon` (Exit, placeholder "New Drape"), Serilog to a rolling file, single-instance mutex, `--restore-desktop` flag parsing, and global exception handlers (`AppDomain.UnhandledException` plus the Avalonia UI-thread unhandled-exception hook).

**Definition of Done**
- [ ] A second instance exits and signals the first
- [ ] Logs are written to `%LOCALAPPDATA%\Drape\logs`
- [ ] `--restore-desktop` is parsed and dispatched (the real behaviour is P2-03)
- [ ] An intentional exception is caught by both handlers and logged
- [ ] Baseline idle CPU and memory are measured and recorded (machine and build named), for PR-5
- [ ] The process runs non-elevated (`asInvoker`)

**Abuse cases**
- Single-instance implemented with a process-name check instead of a named mutex, so it fails under a renamed binary or a second user session
- Baseline recorded from a debug build or with the tray hidden, so later Release comparisons look like regressions or hide real ones

---

### P0-09 — Record spike results as ADRs and update documents

| | |
|---|---|
| **Status** | BLOCKED (waiting for P0-03, P0-04, P0-05, P0-06, P0-07) |
| **Priority** | P0 |
| **Estimate** | 1 day |
| **Role** | architect |
| **Depends on** | P0-03, P0-04, P0-05, P0-06, P0-07 |
| **Spec refs** | ADR-006 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · ADR-007 in [MEMORY/DECISIONS.md](../MEMORY/DECISIONS.md) · [docs/PLATFORM/00-WINDOWS-INTEGRATION.md](../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) · [docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md](../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) |
| **Spec required** | no |

**Why:** Phase 1 is blocked on an accepted host decision. Unrecorded spike results are lost, and unproven claims in `docs/` mislead everyone downstream.

Finalise ADR-007 (host layer) and ADR-006 (icon strategy) in `MEMORY/DECISIONS.md` using the spike evidence. Update `docs/PLATFORM/00-WINDOWS-INTEGRATION.md` and `docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md` with the real results, and review the Phase 1 and Phase 2 estimates.

**Definition of Done**
- [ ] ADR-007 is `Accepted` (or replaced by a new ADR), with alternatives and bad consequences
- [ ] ADR-006 is complete with the chosen hide/show technique
- [ ] The Windows integration and architecture documents contain actual results; claims that were not proven are removed or marked unverified
- [ ] Phase 1 and Phase 2 estimates reviewed by the orchestrator; changes noted in the record
- [ ] Every ADR cites the specific spike result and Windows build it rests on

**Abuse cases**
- ADR-007 marked `Accepted` while one of the six criteria failed on a tested build, with the failure left out of the table
- Unproven claims left in the Windows integration document because deleting them makes the document look thin

---

### P0-10 — Custom subagent definitions (optional)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 0.5 day |
| **Role** | orchestrator |
| **Depends on** | P0-01 |
| **Spec refs** | [AGENTS.md](../AGENTS.md) |
| **Spec required** | no |

**Why:** Role boundaries that exist only in prose are crossed by accident. A definition that states ownership makes a violation visible.

Create `.claude/agents/*.md` for the roles in `AGENTS.md` (shell-engineer-windows, core-engineer, ui-engineer, qa) if the workflow needs them.

**Definition of Done**
- [ ] Each definition states directory ownership and links to the documents that must be read first
- [ ] Each definition was tried on one real task and the outcome is noted in the record

**Abuse cases**
- Definitions copied from a template without the real directory ownership of this repository
- "Tried on a real task" satisfied by a run whose result was never reviewed
