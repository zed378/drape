# Architectural Decision Records (ADRs)

Central record of architectural and project decisions for **Drape**, with rationale and trade-offs. This file replaces the old `MEMORY/decisions-log.md`, the old Indonesian `MEMORY/DECISIONS.md` and the folder `docs/adr/` (see ADR-018).

## Format

Each ADR follows this pattern:

- **Decision** — what was decided
- **Rationale** — why this choice
- **Alternatives Considered** — what was rejected and why
- **Implications** — consequences and trade-offs, **including the bad ones**
- **Status** — Accepted | Proposed | Superseded by ADR-nnn | Deprecated

plus a date and a **Source** line naming the old decision ID (`D-nnn`) or old ADR file it came from, where there is one.

## How to Add an ADR

1. Take the next number (`ADR-019`, ...). Numbers are never reused.
2. Add it at the end of Part II (or a new Part if a new phase of the project begins) and add a line to [`MEMORY-INDEX.md`](./MEMORY-INDEX.md).
3. If it replaces an earlier ADR, change only that ADR's **Status** line to `Superseded by ADR-nnn` and say in the new ADR which one it supersedes. **Do not rewrite the old text** — an ADR records what was decided at a moment, and rewriting it destroys the evidence that the decision changed.
4. A decision that is only an assumption says so in its Status (for example `Accepted (assumption; Q-05)`) and names the open question in [`../TASKS/BACKLOG.md`](../TASKS/BACKLOG.md) that would confirm or overturn it.

**Read Part II first.** Part I was written before anything was built and several entries were overtaken (the WPF stack, the Indonesian-language rule). Part II says what changed.

**Deviation protocol.** If implementation needs something a `docs/` document or an Accepted ADR does not allow, do not just do it. Raise it per [`../TASKS/00-TASK-CONVENTIONS.md`](../TASKS/00-TASK-CONVENTIONS.md): write the ADR that records the change, update the affected `docs/` document, and only then merge the code. A documented deviation is a decision; an undocumented one is a bug nobody has found yet.

---

# Part I — Planning-Phase Decisions (2026-10-09)

Written before any code existed. Everything here is a decision on paper; none has been built or measured.

---

## ADR-001: Working Product Name "Drape"

**Date:** 2026-10-09 | **Source:** old D-002

**Decision:** The working name of the product is **Drape**. *Update 2026-10-10: confirmed as the final name by the owner, see ADR-019.*

**Rationale (corrected on 2026-10-10):**
- The goal is a name that is independent of the Stardock trademark, so that no name, icon, UI text or asset of the original product is reused ([`../docs/PLAN/05-LEGAL-AND-NAMING.md`](../docs/PLAN/05-LEGAL-AND-NAMING.md)).
- **The original rationale was wrong.** It said "Drape = fence in Bahasa Indonesia". "Drape" is an English word, not the Indonesian word for fence. The name appears to have replaced an earlier working name, "Pagar" (Indonesian for fence), which left a trace in the config field `iconsHiddenByPagar`, now renamed `iconsHiddenByDrape`.
- The name therefore stands on "independent working name", not on a translation.

**Alternatives Considered:**
- Keep "Pagar" — rejected earlier; it was the discarded working name.
- Any name containing "Fences" — rejected: infringes the trademark.

**Implications:**
- The name is baked into the root namespace (`Drape.*`), the folder `%LOCALAPPDATA%\Drape` and the config. Renaming later is expensive.
- No trademark search has been done. "Drape" may collide with an existing product; this is untested.

**Status:** Accepted (confirmed final by ADR-019)

---

## ADR-002: Project Working Structure

**Date:** 2026-10-09 | **Source:** old D-001 (owner request)

**Decision:** The repository is organised as `docs/` (reference), `MEMORY/` (record), `TASKS/` (plan), plus `CLAUDE.md` and an agent-instructions file (`AGENT.md`, renamed `AGENTS.md` by ADR-018).

**Rationale:** The owner asked for a working pattern suited to Claude Code and several agents working in parallel. Agents keep no memory between sessions, so continuity has to live in files.

**Alternatives Considered:**
- Only a README and issues — rejected: nothing for agents to read at session start.

**Implications:**
- Documentation has to be kept in step with the code; a stale document produces confidently wrong work (the 2026-10-10 record shows it happened here).
- The detailed layout of each folder was reworked by ADR-018; the three-folder idea itself stands.

**Status:** Accepted

---

## ADR-003: Documentation in Indonesian, Code in English

**Date:** 2026-10-09 | **Source:** old D-003

**Decision:** Documentation and communication in Indonesian; code, identifiers, code comments and commit messages in English.

**Rationale:** The owner writes in Indonesian; the .NET ecosystem is English.

**Alternatives Considered:**
- Everything in English — chosen later, see ADR-013.

**Implications:**
- Documentation was written in a language that most contributors, tools and the reference layout do not use.

**Status:** Superseded by ADR-013

---

## ADR-004: Stack C# / .NET LTS / WPF / CsWin32

**Date:** 2026-10-09 | **Source:** old D-004; old `docs/adr/0001-tech-stack-dotnet-wpf.md`

**Decision:** C# on .NET LTS with WPF for the UI and CsWin32 for Win32 interop.

**Rationale:** Best control over windows and Win32 interop for a light desktop-layer application, assuming Windows only.

**Alternatives Considered:**
- WinUI 3 — weaker window-handle control, packaging constraints.
- Rust + Tauri — different language, WebView limits native window control.
- Electron — memory use too high, poor window-layer control.
- C++/Win32 — highest control, highest cost and risk.

**Implications:**
- Windows only. This became wrong when the owner chose a cross-platform target; see ADR-012.

**Status:** Superseded by ADR-012

---

## ADR-005: Config Storage — One Atomic JSON File

**Date:** 2026-10-09 | **Source:** old D-006; old `docs/adr/0002-config-storage-json.md`

**Decision:** A single `config.json` in `%LOCALAPPDATA%\Drape\`, written atomically (temporary file, flush, replace), with `schemaVersion`, code-based migration, the last 10 backups, and quarantine for a corrupt file. Snapshots are separate files. Details: [`../docs/ARCHITECTURE/02-DATA-MODEL.md`](../docs/ARCHITECTURE/02-DATA-MODEL.md).

**Rationale:** The data is small (hundreds of KB), changes rarely, must survive a crash and be easy to back up and export. No complex queries and no multi-process writes.

**Alternatives Considered:**
- SQLite — native dependency, excessive for the size, not human-readable.
- Windows registry — poor for structured data, hard to move and back up.
- One file per Drape — cross-file consistency (item membership) is hard; more error paths.

**Implications:**
- The whole file is rewritten on each change (small; softened by debounce).
- A migration library and a fixture per schema version are needed in tests.
- Unknown properties are preserved for backward compatibility.
- Portal items (folders with many entries) must not be stored in the config, only the folder path, or the file grows.

**Status:** Accepted

---

## ADR-006: Desktop Icon Strategy — Non-Destructive

**Date:** 2026-10-09 | **Source:** old D-005; old `docs/adr/0003-desktop-icon-strategy.md`

**Decision:** Drape hides the native desktop icons and renders its own from the contents of the Desktop folder. It stores references (path + file ID); it does not move, rename or delete user files without explicit confirmation. Derived rules:
1. References only.
2. Native icons are hidden only while Drape is active and the setting allows it. Restoration is mandatory on every exit path (normal, exception, Guard, next start, `--restore-desktop`).
3. No injection into other processes.
4. New items on the Desktop go to an Inbox or follow a rule.
5. The hiding technique sits behind `IDesktopIcons` so it can be replaced.

**Rationale:** User files must stay safe, and removing the app must return the desktop to normal.

**Alternatives Considered:**
- Move files into an app-owned folder — no need to hide icons, but risk of lost or confused files and broken shortcuts that point at the original path.
- Modify Explorer's `SysListView32` by injection or cross-process subclassing — deepest integration, but fragile, a stability and antivirus risk; not accepted.

**Implications:**
- Needs `Drape.Guard`, `state.json` and `--restore-desktop` (ADR-008).
- Virtual items (This PC, Recycle Bin) are not real files and are unavailable at first (B-01; no virtual items in v1.0, ADR-022).
- Shell behaviour that must be imitated (context menu, overlays, drag and drop) adds work.
- A user who disables "hide desktop icons" will see duplicate icons; this has to be documented.
- The hiding technique relies on undocumented behaviour that may change between Windows builds.

**Status:** Accepted (principle); technique Proposed — pending spike P0-04

---

## ADR-007: One Window per Drape and Host-Layer Strategy

**Date:** 2026-10-09 | **Source:** old D-007; old `docs/adr/0004-window-per-fence-and-host-layer.md`

**Decision, part A:** One window per Drape (now an Avalonia window instead of a WPF one, per ADR-012).

**Decision, part B (pending spike):** The host-layer strategy sits behind `IDesktopHost`:
- **A** — make the window a child of `Progman`/`WorkerW`.
- **B** — top-level window at the bottom of the z-order (`HWND_BOTTOM`, `WS_EX_TOOLWINDOW`, not activated).
- **C** — hybrid: try A, fall back to B when verification fails. Provisional recommendation.

Contract in Core: `IDesktopHost` with `Attach`, `Reattach`, `Place(window)`, `EnsureZOrder`, `Detach`, and a `FakeDesktopHost` for tests. Degraded mode (a normal window at the bottom of the z-order) is provided if the host fails.

Acceptance criteria for a strategy (all must pass on Windows 11 23H2/24H2+ and Windows 10 22H2):
1. Shown above the wallpaper and below application windows.
2. Visible after Win+D and peek.
3. Does not steal focus; absent from the taskbar and Alt+Tab.
4. Recovers after an Explorer restart.
5. Transparency/blur correct on mixed DPI.
6. Idle CPU/GPU cost within budget.

**Rationale:** Per-window hit-testing is natural, areas outside a Drape do not capture clicks, moving and resizing are simple, and a rendering crash is isolated. The Explorer desktop window structure is undocumented and changes between builds, hence the spike.

**Alternatives Considered:**
- One full-screen window drawing all Drapes — captures all desktop clicks or needs complicated regions and hit-testing; wasteful on large screens; blocks native interaction.

**Implications:**
- Many windows (small memory cost each, layered transparency). Budgets: PR-5. Host structure may change between Windows builds: PR-1.
- Whether an Avalonia window can be re-parented into the desktop host and stay transparent is unproven (risk PR-4; spike P0-03, P0-06).
- Spike P0-03 gates Phase 1; this ADR becomes Accepted or is superseded when the spike result is attached.

**Status:** Proposed (pending spike P0-03)

---

## ADR-008: Single Non-Elevated Process plus Guard

**Date:** 2026-10-09 | **Source:** old D-008; old `docs/adr/0005-single-process-non-elevated-guard.md`

**Decision:** One non-elevated main process plus `Drape.Guard`, a small UI-less process that the app launches when icons are hidden. The Guard waits on the main process handle and restores the desktop icons however the main process ends. `state.json` records that Drape hid the icons so the next start and `--restore-desktop` can recover after a forced reboot. Single instance via a named mutex; a second instance signals the first and exits.

**Rationale:** The app must be safe (never leave the desktop hidden), must not ask for admin rights, and must stay simple to maintain.

**Alternatives Considered:**
- Single process only — simple, but no recovery if killed.
- Windows service plus UI — strong recovery, but needs admin to install, large attack surface, excessive.
- Elevated app — rejected: security and UX risk; breaks drag and drop from a normal Explorer.

**Implications:**
- Drops from an elevated application are not supported (UIPI); must be documented.
- Two binaries; the Guard must stay tiny, with minimal dependencies, and be signed with the app.
- If the Guard fails to launch, the app keeps running and warns; `state.json` and the tray button remain as fallbacks. The Guard is required from the MVP (ADR-022, Q-08 closed).

**Status:** Accepted

---

## ADR-009: Windows 11 Primary, Windows 10 Best Effort

**Date:** 2026-10-09 | **Source:** old D-009

**Decision:** Windows 11 is the primary target; Windows 10 22H2 is best effort. x64 first.

**Rationale:** Windows 10 reached end of support on 14 October 2025, but Microsoft's consumer Extended Security Updates now run to 12 October 2027, and StatCounter shows Windows 10 at 27.83% of worldwide Windows desktops in September 2026 (Windows 11: 71.44%). It is too common to ignore and too old to be primary; Avalonia lists Windows 10 22H2 x64 as Tier 2. Details in ADR-022.

**Alternatives Considered:**
- Windows 11 only — smaller test matrix, excludes users still on Windows 10.
- Full Windows 10 support — larger test and compatibility burden for an unsupported OS.

**Implications:**
- Defects found only on Windows 10 may be closed as won't-fix.
- Confirmed with research by ADR-022 (Q-05 closed); revisit when consumer ESU ends on 12 October 2027.

**Status:** Superseded in part by ADR-023 (Windows 10 is now a fully supported minimum, not best effort)

---

## ADR-010: No Telemetry and No Network by Default

**Date:** 2026-10-09 | **Source:** old D-010

**Decision:** The application collects no telemetry and makes no network requests by default.

**Rationale:** Privacy; there is no product need for either. Details: [`../docs/SECURITY/00-SECURITY-AND-PRIVACY.md`](../docs/SECURITY/00-SECURITY-AND-PRIVACY.md).

**Alternatives Considered:**
- Opt-out crash reporting — rejected for v1; it would make the default "network on".

**Implications:**
- No field data on crashes or performance; defects depend on user reports and local logs.
- Update checking needed a separate rule; see ADR-017.

**Status:** Accepted

---

## ADR-011: One Item Belongs to Exactly One Drape in v1

**Date:** 2026-10-09 | **Source:** old D-011

**Decision:** In v1 an item can be in only one Drape.

**Rationale:** Simplifies the data model and the UI.

**Alternatives Considered:**
- An item in several Drapes — needs reference semantics, removal rules and conflicting-rule resolution.

**Implications:**
- Users coming from tools that allow duplicates may object (deferral B-03).
- Feature F-03 contradicted this in the old documents; fixed on 2026-10-10 (see the record).

**Status:** Accepted (Q-09 closed by ADR-022)

---

# Part II — What Changed or Was Added (2026-10-10)

Each entry says which earlier ADR it supersedes, if any.

---

## ADR-012: Stack C# / .NET 10 LTS / Avalonia UI, Cross-Platform

**Date:** 2026-10-10 | **Source:** old `docs/adr/0001-tech-stack-dotnet-avalonia.md` | **Supersedes:** ADR-004 | **Closes:** Q-01

**Decision:** C# on .NET 10 LTS with **Avalonia UI** as the UI framework, chosen by the project owner for cross-platform reach (Windows, macOS, Linux, Android, iOS). `Drape.Core` stays free of platform dependencies. The shell is split per desktop platform: `Drape.Shell.Windows`, `Drape.Shell.Mac`, `Drape.Shell.Linux`. Platform interfaces (`IDesktopHost`, `IDesktopIcons`, `IInputHook`, ...) are defined in Core and chosen through DI at start-up. **Mobile has no Shell project.** Details: [`../docs/ARCHITECTURE/01-TECH-STACK.md`](../docs/ARCHITECTURE/01-TECH-STACK.md), [`../docs/PLATFORM/01-PLATFORM-SUPPORT.md`](../docs/PLATFORM/01-PLATFORM-SUPPORT.md).

**Rationale:** WPF runs only on Windows and cannot meet the owner's cross-platform target. Avalonia has a WPF-like XAML, runs on all five platforms, gives access to native handles (`HWND`/`NSWindow`/window handles) through `IPlatformHandle`, has an active ecosystem and an MIT licence.

**Alternatives Considered:**
- .NET MAUI — no Linux desktop, low-level window control weak; not suited to a resident desktop-layer app.
- Uno Platform — wide coverage, but complex, large binaries, smaller community for native desktop cases.
- Tauri (Rust + WebView) — very light, but a different language and the WebView limits native window control.
- Electron — memory use too high, poor window-layer control.

**Implications:**
- The core feature, a panel above the wallpaper, depends on OS APIs that do **not** exist on every platform: Windows, macOS and Linux may support it (each needing a spike), Android and iOS cannot; mobile ships a plain shortcut-group manager at most (see ADR-014).
- New risks: PR-4 (Avalonia may not behave as a desktop-layer window) and PR-5 (memory and CPU budgets with .NET + Avalonia). Both are unproven until Phase 0 spikes.
- Avalonia is less common than WPF; some platform APIs need manual bridges; mobile support is still maturing.
- The old documents still said WPF and Windows-only after this choice; that mismatch is part of the 2026-10-10 documentation rework.

**Status:** Accepted

---

## ADR-013: English Documentation, Code and Commits

**Date:** 2026-10-10 | **Source:** owner request | **Supersedes:** ADR-003

**Decision:** All documentation, code, identifiers, comments and commit messages are in English. Conversation with the owner stays in the owner's language (Indonesian). The application UI ships `id-ID` and `en-US`.

**Rationale:** The reference layout, the tooling and the wider .NET ecosystem are English; a single language removes the translation burden between documents and code. The owner asked for the change.

**Alternatives Considered:**
- Keep Indonesian documentation — rejected by the owner.
- Bilingual documents — double maintenance and drift.

**Implications:**
- All earlier Indonesian documents had to be translated (see the 2026-10-10 record); translation can introduce errors that have not been reviewed by the owner.
- The owner reads Indonesian more easily than the documents are now written; explanations in conversation remain Indonesian.

**Status:** Accepted

---

## ADR-014: v1.0 Ships Windows Only

**Date:** 2026-10-10 | **Source:** new | **Related:** ADR-012

**Decision:** v1.0 ships for Windows only. macOS, Linux, Android and iOS are Phase 6 (post-v1); each needs a spike and its own ADR before commitment. The architecture stays cross-platform so those ports are possible.

**Rationale:** The desktop-layer feature is unproven on every platform, including Windows. Committing five platforms before one works would multiply risk and cost.

**Alternatives Considered:**
- Windows, macOS and Linux together in v1.0 — roadmap and CI matrix grow several-fold with no spike evidence.
- Windows only and a Windows-only architecture — cheaper now, expensive to undo.

**Implications:**
- Keeping the architecture cross-platform costs discipline now (interfaces, capability flags, no platform types in Core) for benefit that arrives later or never.
- Owners expecting five platforms at launch will not get them.

**Status:** Accepted (confirmed by the owner on 2026-10-10; Q-15 closed)

---

## ADR-015: Platform Seams

**Date:** 2026-10-10 | **Source:** found while reviewing the old documents | **Related:** ADR-012

**Decision:**
1. **`IPlatformCapabilities`** exposes capability flags (can host a desktop layer, can hide native icons, supports blur, ...). Code asks for a capability, not "is this Windows".
2. **`IDesktopLocations`** and **`IAutostart`** live in the Shell projects, not Infrastructure, because they are native or registry based.
3. **Core returns a neutral `IconBitmap`** (BGRA pixels plus size), not UI image types; the UI layer turns it into an Avalonia bitmap.
4. **`IBlurEffect`** abstracts the blur/acrylic backdrop.
5. **Mobile apps register no desktop-layer Shell.**

**Rationale:** Fixes architecture-rule violations found in the old documents: an `ImageSource` (a WPF type) returned from Core, and Known Folder and autostart (native/registry) placed in Infrastructure.

**Alternatives Considered:**
- OS checks (`OperatingSystem.IsWindows()`) in shared code — scatters platform knowledge and defeats testing.
- Let Core reference the UI bitmap type — breaks the "Core has no platform or UI reference" rule.

**Implications:**
- One extra conversion step for icons (pixels to bitmap) in the UI layer.
- A capability flag set must be kept honest: a flag that claims support without a spike result misleads.

**Status:** Accepted

---

## ADR-016: Terminology — "Drape" in the Product, `Fence` in Code

**Date:** 2026-10-10 | **Source:** new

**Decision:** The product and UI term is "Drape"; code identifiers used `Fence`. **Superseded by ADR-020 the same day**, when the owner asked for everything to be called Drape.

**Rationale:** At the time: a judgement that the generic word "fence" in internal code was a low trademark risk, and that a `Drape` type inside a `Drape` namespace reads badly.

**Alternatives Considered:**
- `Drape`-based identifiers throughout — chosen later by the owner (ADR-020).
- A neutral term such as `Panel`.

**Implications:** Two words for one concept, which is why the owner removed it.

**Status:** Superseded by ADR-020

---

## ADR-017: Update Checks Are Opt-In

**Date:** 2026-10-10 | **Source:** inconsistency found in the old build document | **Related:** ADR-010

**Decision:** Update checks are opt-in and off by default, so "no network by default" holds. Setting `updateCheck`: `off` (default) | `manual` | `periodic`.

**Rationale:** The old build/release document specified periodic update checks while the security document and ADR-010 said no network by default. They cannot both be true; the privacy promise wins.

**Alternatives Considered:**
- Periodic checks on by default — contradicts ADR-010.
- No update checking at all — leaves users on old, possibly unsafe versions with no signal.

**Implications:**
- Many users will never turn it on and will run old versions.
- The config schema gains a setting and a migration; the privacy statement must describe exactly what a check sends.

**Status:** Accepted (Q-18 closed by ADR-022)

---

## ADR-018: Documentation Layout Adopted from the Reference Repository

**Date:** 2026-10-10 | **Source:** owner request | **Related:** ADR-002, ADR-013; closes Q-14

**Decision:** Adopt the layout of the sibling "Callibrator" repository:
- `docs/` by uppercase category (`PLAN`, `ARCHITECTURE`, `PLATFORM`, `UI-UX`, `ENGINEERING`, `TESTING`, `DEVOPS`, `SECURITY`) with `NN-` numbered files. ADRs are **not** in `docs/`: they live in this file, and `docs/adr/` is removed.
- `TASKS/` = `00-TASK-CONVENTIONS.md` + `PROGRESS.md` + `BACKLOG.md` + `PHASE-n` files, with task IDs `Pn-nn`. This replaces `Mn-nn`, `BOARD.md`, `DONE.md` and `open-questions.md` (questions become `Q-n` in `BACKLOG.md`).
- `MEMORY/` = `DECISIONS.md`, `records/`, `specs/`, `templates/`, `MEMORY-INDEX.md`, `CHANGELOG.md`. The old `decisions-log.md` and the duplicate `DECISIONS.md` are merged here.
- `AGENT.md` is renamed `AGENTS.md` (closes Q-14).
- `project-context.md`, `lessons-learned.md` and `user-preferences.md` are folded into `docs/PLAN`, the traps table in the task conventions, and `CLAUDE.md`.
- Commits now carry the task ID.

**Rationale:** The owner asked for the same structure as the reference project, which has a working record-keeping routine; two parallel layouts would drift.

**Alternatives Considered:**
- Keep the flat numbered `docs/00..14-*.md` — works for one person, scales poorly to many agents and many documents.
- Keep Indonesian `MEMORY/` files — rejected with ADR-013.

**Implications:**
- Every cross-reference in the old documents had to be rewritten; relative link targets were machine-checked afterwards, see the record.
- Old IDs (`D-nnn`, `Mn-nn`, `docs/adr/000n`) appear in history and in this file's Source lines; a reader must map them.
- The reference layout was built for a larger, multi-tenant project; some of its machinery (phase summaries, many spec files) is heavy for a pre-code project and may sit unused.

**Status:** Accepted

---

## ADR-019: Licence MIT and Final Name "Drape"

**Date:** 2026-10-10 | **Source:** owner decision | **Closes:** Q-02, Q-03 | **Related:** ADR-001

**Decision:** The source is released under the **MIT** licence and is free of charge. The product name **Drape** is final.

**Rationale:** The owner chose a free, permissive, open-source model and the existing working name.

**Alternatives Considered:**
- Apache-2.0 — adds a patent grant; not chosen.
- Proprietary/commercial — not chosen.
- Another name — not chosen.

**Implications:**
- P0-01 creates `LICENSE` (MIT) and P5-06 produces `THIRD_PARTY_NOTICES.md`; every dependency must be compatible with MIT distribution.
- MIT gives no patent protection and no trademark protection.
- "Final" is the owner's choice, **not a cleared name**: no trademark search has been done, and P5-06 still runs one. If it fails, a rename touches the namespace, folders and config path.
- Free distribution removes any paid-store assumption; the distribution channel is settled in ADR-022.

**Status:** Accepted

---

## ADR-020: Rename `Fence` to `Drape` in All Identifiers

**Date:** 2026-10-10 | **Source:** owner decision | **Supersedes:** ADR-016 | **Closes:** Q-17

**Decision:** The panel concept is called `Drape` everywhere: UI, docs and code (`Drape`, `DrapeItem`, `DrapeManager`, `DrapeWindow`, `DrapeViewModel`, `DrapeBuilder`; config keys `drapes`, `drapeId`, `targetDrapeId`; branch slugs `drape-window`). The word "fence" appears in no identifier; it is used only when naming the competing product.

**Rationale:** One word for one concept; removes a trademark-adjacent identifier and the mapping readers had to learn.

**Alternatives Considered:**
- Keep `Fence` in code (ADR-016) — two names for one thing.
- `Panel` — collides with Avalonia's `Panel` type.

**Implications:**
- The entity `Drape` has the same name as the root namespace `Drape`. Simple-name resolution inside `Drape.*` namespaces then finds the namespace first. Rule CS-10: import the model namespace explicitly, use an alias where ambiguous, suppress CA1724 for that type.
- No code or shipped config exists yet, so no schema migration is needed; schema v1 uses the `drapes` key from the start.
- All documents were rewritten accordingly on 2026-10-10.

**Status:** Accepted

---

## ADR-021: Mobile Scope — Companion Shortcut Groups, Last in the Queue

**Date:** 2026-10-10 | **Source:** owner delegated the decision to the assistant (Q-16: "decide yourself") | **Closes:** Q-16 | **Related:** ADR-014, ADR-015

**Decision:** Android and iOS stay in the plan but come **last** (after macOS and Linux) and are a **companion**, not a port of the desktop layer. A Drape on mobile is a named group of launchable references, with no panel over a wallpaper, no widgets and no sync in the first mobile release. An item on mobile is one of:
- **Android:** an app the user picks (launch intent), a URL, or a file/folder picked through the system document picker (persisted URI permission).
- **iOS:** a URL or URL-scheme link, a Shortcuts shortcut by name, or a file the user picks (security-scoped bookmark). iOS does not allow listing installed apps, so there is no app item.

Data stays local in the app sandbox. `Drape.Core`, `Drape.Infrastructure` and `Drape.UI` are reused; mobile registers no desktop-layer Shell project (ADR-015). Mobile-only item kinds (`app`, `url`) are added to the data model when Phase 6 reaches P6-09.

**Rationale:** The core promise (organising the desktop) has no mobile equivalent, so the honest mobile product is small. The decision lets Phase 6 proceed without inventing features, while Windows stays first.

**Alternatives Considered:**
- Drop mobile entirely — loses the cross-platform promise the owner chose Avalonia for.
- Port desktop features (widgets, home-screen overlay) — impossible or store-rejected on both platforms.
- Sync groups between desktop and phone — needs accounts or a network, against ADR-010.

**Implications:**
- This is the assistant's decision on the owner's delegation, a working decision; P6-09 may refine or drop it. App-store policies (Android package visibility, iOS restrictions) are unverified.
- P6-10 and P6-11 may still be `DROPPED` if P6-09 finds the product not worthwhile.

**Status:** Accepted (assumption, owner-delegated; confirm when Phase 6 starts)

---

## ADR-022: Answers to the Remaining Open Questions (Q-04 … Q-13, Q-18)

**Date:** 2026-10-10 | **Source:** owner instruction "answer all remaining questions; research if needed" | **Closes:** Q-04, Q-05, Q-06, Q-07, Q-08, Q-09, Q-10, Q-11, Q-12, Q-13, Q-18 | **Related:** ADR-006, ADR-009, ADR-010, ADR-011, ADR-017

**Decision:**

| Q | Answer |
|---|---|
| Q-04 distribution | **Direct.** GitHub Releases with a Velopack per-user installer, a portable zip and checksums, plus a winget manifest. No Microsoft Store/MSIX in v1.0 (deferred as B-09). |
| Q-05 minimum OS | Windows 11 22H2+ is primary (24H2 is Avalonia Tier 1). Windows 10 22H2 x64 stays best effort **while consumer ESU runs (to 12 October 2027)** *(overturned by ADR-023: it is the supported minimum)*; after that date it leaves the test matrix unless usage still justifies it. Windows 10-only defects never justify a workaround that harms Windows 11. |
| Q-06 ARM64 | v1.0 ships **win-x64 only**; Windows 11 on ARM runs it through emulation (unverified for the desktop layer and hook). A native win-arm64 build follows v1.0 (B-02). |
| Q-07 languages | **id-ID and en-US only**, in resource files so contributors can add languages later. |
| Q-08 Guard | **Required from the MVP** (Phase 2). It is the desktop-safety control (ADR-006, ADR-008). |
| Q-09 one item, several Drapes | **No** in v1.0 (ADR-011). B-03 keeps the idea. |
| Q-10 repo and CI | **GitHub and GitHub Actions.** The repository was initialised by the owner with remote `origin` = `https://github.com/zed378/drape.git`; the first commit is the documentation set. |
| Q-11 virtual items | **No** in v1.0 (B-01). |
| Q-12 look | **Own visual design**, equivalent function; never a pixel copy (legal doc, rule 5). |
| Q-13 parallel work | **Several agents in parallel** under `AGENTS.md`. WIP stays **3 tasks**; separately, the orchestrator runs **at most 4 subagents concurrently** (owner instruction, 2026-10-10). |
| Q-18 update checks | **Off by default.** The first run explains the option. Settings offer a manual "Check for updates" button and an opt-in periodic check. Auto-apply on startup is disabled; an update is applied only on the user's action. |

**Rationale (evidence):**
- *Q-04.* Microsoft's MSIX documentation says packaged full-trust apps keep running as normal processes, but **all writes under HKCU are copied on write to a private per-user, per-app location** and the package files are read-only. Inference, not tested: a `Run` value written to HKCU would not be seen by Windows at logon, so autostart (F-12) would need the manifest's startup-task mechanism instead. Whether a global low-level mouse hook or desktop-window access is allowed under the Store is **not stated** on that page, and a search found nothing. Together with MIT/free distribution (no store revenue needed) and Velopack's delta updates, direct distribution is the lower-risk path. Velopack's update API (`UpdateManager.CheckForUpdatesAsync`) lets the application decide when to check, download and apply, and can read GitHub Releases as its source.
- *Q-05.* Microsoft's consumer ESU page: the program ends 12 October 2027 (a Windows Latest report says the end date was extended by a year from October 2026). StatCounter, September 2026: Windows 11 71.44%, Windows 10 27.83%. Avalonia support tiers: Windows 11 24H2 Tier 1, 22H2 Tier 2, Windows 10 22H2 x64 Tier 2.
- *Q-06.* Avalonia lists Windows 11 ARM64 as supported (Tier 1 on 24H2); .NET 10 targets `win-arm64`. A native build is therefore feasible, but testing it needs ARM hardware or a VM, which is not in the v1.0 plan.

**Alternatives Considered:**
- Microsoft Store/MSIX first — rejected for v1.0 (autostart and policy unknowns above).
- Windows 11 only — smaller matrix, but drops roughly a quarter of desktops by StatCounter.
- ARM64 in v1.0 — adds a test platform with no hardware plan.
- Periodic update checks on by default — contradicts ADR-010.

**Implications:**
- These are the assistant's decisions on the owner's delegation; they are working decisions, not owner confirmations, and each can be overturned by a new ADR.
- P0-02 creates the GitHub Actions workflow.
- Winget acceptance of a Velopack-built installer is unverified and is checked in P5-03. Whether x64-on-ARM emulation supports the hook and desktop layer is unverified.
- Windows 10 support needs a decision before 12 October 2027.
- Research caveat: statistics and ESU dates come from web pages read on 2026-10-10; StatCounter figures are a sampling estimate, not a census.

**Sources:**
- [MSIX: how packaged desktop apps run](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes)
- [Windows 10 consumer ESU](https://www.microsoft.com/en-us/windows/extended-security-updates)
- [Windows Latest on the ESU extension](https://www.windowslatest.com/2026/06/25/windows-10-support-quietly-extended-until-oct-2027-as-users-reject-windows-11/)
- [StatCounter Windows versions](https://gs.statcounter.com/os-version-market-share/windows/desktop/worldwide)
- [Velopack integration overview](https://docs.velopack.io/integrating/overview)
- [Avalonia supported platforms](https://docs.avaloniaui.net/docs/supported-platforms)

**Status:** Accepted (assumption, delegated by the owner; each answer can be overturned by a new ADR)

---

## ADR-023: Windows 10 22H2 Is the Minimum Supported OS; Minimums and Channels for Other Platforms

**Date:** 2026-10-10 | **Source:** owner decision (Windows); other platforms decided by the assistant on the owner's delegation | **Supersedes in part:** ADR-009, and the Q-05 answer in ADR-022 | **Related:** ADR-014, ADR-021

**Decision:**
- **Windows:** the minimum supported OS is **Windows 10 22H2 (build 19045), x64**. It is fully supported: it is in the manual matrix and the release gate, and its defects are fixed. Windows 11 (24H2) remains the primary *development* target. Older Windows 10 builds are unsupported. The owner set no sunset date. The winget manifest declares `MinimumOSVersion: 10.0.19045.0`.
- **Other platforms (assistant's decision):** the minimum is the oldest version Avalonia lists at Tier 2 or better — macOS 14 Sonoma; Ubuntu 22.04 LTS and Debian 12 families on x64 with X11 guaranteed and Wayland by capability; Android 12 (API 31); iOS 18. Channels: macOS notarised direct download; Linux AppImage; Android Google Play plus signed APK; iOS App Store/TestFlight — details in [`../docs/PLATFORM/01-PLATFORM-SUPPORT.md`](../docs/PLATFORM/01-PLATFORM-SUPPORT.md).

**Rationale:** The owner wants the widest Windows reach: Windows 10 is still about 28% of Windows desktops (StatCounter, September 2026). The same "lowest supported version" rule is applied to the other platforms so the policy is uniform and rests on the framework's own support tiers rather than guesses.

**Alternatives Considered:**
- Windows 11 only — rejected by the owner.
- Windows 10 as best effort (ADR-009/022) — replaced by this decision.
- Newer minimums for the other platforms (macOS 15/26, Android 16, iOS 26) — smaller matrix, but not what the owner's Windows choice implies.

**Implications:**
- Every Phase 0 spike and every phase gate must pass on **both** Windows 10 22H2 and Windows 11; VMs for both are mandatory.
- Windows 10 lacks Mica/system backdrop and uses a clip geometry for rounded corners, so visual features need two paths (`IBlurEffect`). This enlarges P0-06 and P3-02/P3-03.
- After 12 October 2027 un-enrolled Windows 10 users run an unpatched OS while Drape still claims support; recorded as risk PR-9 in the risk register. A sunset needs a new ADR.
- Distribution channels for macOS, iOS and Android need paid developer accounts; none of the store policies is verified.
- Untested claim: Avalonia's tiering is read from its documentation, not exercised.

**Status:** Accepted (Windows part: owner decision; other platforms: assumption, owner-delegated)
