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

**Status:** Accepted atomic-storage intent; durability/compatibility limits clarified by ADR-026. Earlier text remains historical.

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

**Status:** Accepted non-destructive principle; technique remains unverified. Rust recovery/ownership constraints clarified by ADR-025/026; new evidence comes from R0-05, not the legacy P0-04 card.

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

**Status:** Proposed host technique pending Rust R0-04/R0-07 evidence; one-window-per-Drape principle retained by ADR-025. Earlier Avalonia/spike references remain historical.

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

**Status:** Accepted independent-Guard intent; launch ordering, liveness and App/Guard/CLI ownership protocol superseded/clarified by ADR-025/026. Earlier body remains historical.

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

**Status:** Superseded by ADR-025 (Rust + Slint target, 2026-11-25); retained as historical C# decision.

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

**Status:** Accepted in intent; Rust names/ports and native lifetime boundaries updated by ADR-025.

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

---

## ADR-024: The Desktop App References the Platform Shell Directly (One Thin Head per OS)

**Date:** 2026-10-23 | **Task:** P0-01 | **Related:** ADR-012, ADR-015, ADR-023

**Decision:** `Drape.App.Desktop` holds an unconditional `ProjectReference` to `Drape.Shell.Windows`. When Phase 6 adds macOS and Linux, each gets its own app head (`Drape.App.Mac`, `Drape.App.Linux`) with the same shape: a composition root that references exactly one `Drape.Shell.<Platform>` project and registers its implementations of the Core interfaces.

**Rationale:** The composition root is the only place allowed to know which platform it is on (CS-21). A thin head per OS keeps that knowledge in one file per platform, keeps `Drape.UI` and `Drape.Core` provably free of shell references, and makes the architecture test (`LayeringTests.AppDesktop_IsTheOnlyProjectReferencingTheWindowsShell`) express the rule directly rather than approximately.

**Alternatives Considered:**
- **(a) Multi-target the app** (`net10.0-windows`, `net10.0-macos`) with conditional `ProjectReference` items selected by TFM. Rejected: the app's `Main`, manifest, installer and packaging differ per platform far more than the TFM does; the conditionals would spread through the project file and the CI matrix, and a mistake would fail at runtime instead of at restore.
- **(b) Conditional `ProjectReference` selected by `$(OS)` or a property.** Rejected: it makes the build machine's OS part of the build definition, which breaks cross-compilation and reproducible builds from a tag, and it hides the dependency from static inspection.
- **(c) Thin head per OS.** Chosen.

**Implications:**
- Phase 6 adds two projects to the solution and two release pipelines; the shared code is `Drape.Core` + `Drape.Infrastructure` + `Drape.UI`, so the duplication is a composition root, not a UI.
- `Drape.App.Desktop` cannot be built on macOS or Linux. That is intended: it is the Windows head.
- The architecture test must be extended in Phase 6 to assert that each head references exactly one shell project; the rule is written down here so it is not reinvented.
- Cost of changing later: medium. Moving to multi-targeting would touch the app project, CI and the packaging pipeline but not Core, UI or the shell projects.

**Status:** Superseded in implementation detail by ADR-025; App-to-platform-adapter composition remains, C# project-reference choice is historical.


## ADR-025: Rust + Slint Target Architecture; Documentation Before Implementation

**Date:** 2026-11-25 | **Source:** owner decision in conversation: Rust for RAM, performance, memory safety and application size; Slint for native UI without WebView/web engine; owner then requested documentation updates only.

**Status:** Accepted target decision. Documentation delivery requires separate review; no implementation acceptance or benchmark result is implied.

**Supersedes:** ADR-012 stack/tooling; ADR-024 C# project-reference implementation detail; ADR-004 where still relevant. **Updates in part:** ADR-007 (Slint replaces Avalonia, one window per Drape retained; host strategy still unproved), ADR-015 (Rust traits/capability data), ADR-022 packaging preference requires Rust revalidation, ADR-023 non-Windows minimums require new evidence. Other product/safety decisions remain.

### Decision

Use Rust stable with compiled Slint components and native OS adapters. No WebView/browser engine, JavaScript UI runtime or .NET runtime in the future Rust product. Platform-free Core, I/O Infrastructure, isolated shell adapters, shared UI, explicit App composition and independent minimal Guard form a Cargo workspace under crates/. Exact toolchain/dependency versions/backend/renderer are selected in later tasks, not invented now.

Update active reference docs, task plan, agent instructions and continuity records now. Preserve existing C# source/build files/CI/harnesses and original historical records; no Rust code, Cargo files, source removal or desktop experiment is authorised in this change. New R0/R1 tasks replace historical P0/P1 as active Rust work; P2–P6 retain future product scope. No language-only schema bump: version 1 JSON compatibility, unknown fields/values and old recovery records need tests.

### Rationale and alternatives

The owner's goals favour a native-compiled non-WebView architecture with controlled allocations and narrowly reviewed native code. Remaining on C#/Avalonia preserves current investment but does not meet the chosen stack direction. Tauri introduces the excluded WebView. iced/egui are native alternatives but Slint was explicitly selected; Flutter/Rust adds a separate UI language/runtime. None of these comparisons constitutes measured RAM or performance evidence.

### Safety and consequences, including costs

Rust ownership reduces classes of memory errors in safe code; native unsafe/COM/FFI still needs lifetime/apartment/ABI review, matching RAII releases and no unwinding across callbacks. Slint-owned native handles are borrowed by Shell only within registration lifetime. Shared Core/UI forbid unsafe and OS checks; capabilities remain neutral data.

Guard readiness plus durable ownership intent precede hiding. Guard failure keeps icons visible; config/log failure cannot prevent restoration. Drop/panic hooks do not cover abort/kill/reboot. Independent Guard, next-start recovery and standalone --restore-desktop remain unwaivable. This clarifies unsafe ordering in earlier startup prose; it is not permission to weaken recovery.

Costs: rewrite/parity effort, new Rust/Slint expertise, repeated host/rendering/drop/UIA spikes, new test/CI/packaging tooling, possible backend integration gaps, and revalidation of Phase 6 support. Native compilation does not imply OS-native controls or guaranteed accessibility. Existing C# test counts/results are not Rust evidence. Old estimates are withdrawn pending Rust spikes. No destructive legacy retirement is authorised.

### Licensing and performance

Drape remains MIT. Explicit dependency-policy exception: Slint's royalty-free desktop/mobile/web application licence, with AboutSlint attribution planned through top-level Settings/About and notices retained. The exact pinned release/licence/dependencies must be verified at R0-02 and before distribution. Do not describe Slint as MIT or silently switch licence options.

Preserve existing ceilings: combined App + Guard working set <100 MB at 20 panels ×20 items after 30 minutes, idle CPU average <0.5%, startup median <1.5 s on SSD. Record private bytes, GPU memory, per-panel scaling and installed/download/binary/prerequisite sizes separately. Benefits remain unmeasured; Q-20 sets numeric size/tighter memory budgets after baseline. Q-19 blur MVP priority, Q-21 backend/renderer/handle/drop/UIA integration and Q-22 packaging remain open. Blur stays in planned scope until owner decides otherwise.

### References and follow-through

- [Owner-supplied Gemini discussion](https://share.gemini.google/UwyvsCzSs0aM), read 2026-11-25: brainstorming only, not validated code, version pins, automatic desktop fallback or RAM/timeline evidence.
- [Slint official site](https://slint.dev), [documentation](https://docs.slint.dev), [royalty-free licence 2.0](https://github.com/slint-ui/slint/blob/master/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md), licence text read 2026-11-25. Reverify at version pinning.
- [Migration specification](../docs/ARCHITECTURE/03-RUST-MIGRATION.md), [task cards](../TASKS/RUST-MIGRATION.md) and [record](./records/2026-11-25-R0-01-rust-slint-documentation.md).

No Rust build/test/desktop validation was performed for this decision. R0-02–R0-10 and R1-01–R1-06 establish future implementation evidence and parity before release or any separately authorised source retirement.


## ADR-026: Reviewed Recovery, Durability and Acceptance Contracts

**Date:** 2026-11-25 | **Source:** owner explicitly requested multiple specialist agents to audit all documentation and patch/remove unsupported or irrelevant content. Architecture, Windows/security and QA/release/UI reviewers contributed independent findings.

**Status:** Accepted documentation clarification within the existing safety/product requirements. No Rust implementation, native experiment or release acceptance is implied.

**Clarifies:** ADR-005 atomic storage, ADR-006 non-destructive icon ownership, ADR-007 host evidence, ADR-008 independent Guard, ADR-015 platform seams and ADR-025 migration. Existing stack, Windows minimum, feature scope, MIT licence, offline defaults and open owner decisions remain.

### Decision and rationale

Serialize App/Guard/standalone CLI recovery by user/session ownership and validated process/generation identity before mutating icons. A second instance must not restore a live owner's changes. Use authenticated bounded IPC, reject stale/replayed state and acknowledge/revalidate Guard before hide. Monitor Guard health after readiness: if lost while App survives, promptly restore owned changes and prevent rehide until rearmed. Unknown original intent preserves observed state, reports uncertainty and offers explicit restoration; never blind-toggle or force user-hidden icons visible. Test each non-atomic crash boundary and retain independent/next-start recovery instead of claiming all concurrent failure windows disappear.

Quick-hide that mutates native icons follows the same ownership/recovery protocol. Native context-menu destructive verbs require explicit confirmation/reviewed invocation; Windows or third-party handlers cannot be assumed to prompt. Threads/timeouts protect responsiveness, not process integrity or safe cancellation. No new process-isolation feature is added; unsupported destructive/native invocation must not bypass the non-destructive contract.

Native FileSystem operations belong to platform adapters selected/injected by App; Infrastructure coordinates platform-neutral persistence without importing Shell. Preserve literal path data and filesystem comparison rules, native handle/COM lifetimes and stale response rejection. Core/shared UI stay platform-free and safe Rust.

Atomic config saving protects the last successfully committed valid configuration; the 500 ms debounce means unsaved recent edits can be lost. Flush/replace guarantees are bounded by filesystem/storage behaviour. Require single-writer sequencing, failure injection, unknown-field/value preservation and compatible legacy records; never promise absolute power-loss immunity.

Keyboard-only entry to tray/Settings/Drape chooser and panel move/resize must not depend on optional global hotkeys or assume Tab crosses native windows. Verify UIA/keyboard/high contrast and the promised DPI through 300%/one-to-four-monitor range. Capability matrices must distinguish planned integration from measured Rust support.

Performance measures App + Guard, machine-normalised CPU, summed process working sets (shared pages can be counted twice), private bytes/GPU memory, cold/warm startup and installed/download/prerequisite footprint. Thresholds are retained, not claimed achieved. Installer/update/uninstall cover Windows minimum, signed compatible App/Guard pairs, trust roots, staged failure/rollback and restoration before removal. Internal unsigned experiments do not waive signing for formal public beta/stable releases.

### Alternatives and costs

Rejected: treating readiness as permanent protection, restoring before checking live ownership, assuming Drop/panic hooks handle kills, treating native thread timeouts as containment, assuming OS confirmation, or retaining inaccurate zero-loss/support claims. These simplify prose at the cost of unsafe or unverifiable implementation guidance.

The clarified contracts require more explicit state-machine, failure-injection, native-lifetime and manual-matrix tests. They do not choose unverified APIs, thresholds, an updater implementation, a new schema or a final desktop-host strategy. Q-19–Q-22 remain open. Q-23 records missing pin/page/tab/portal representation contracts as feature-spec prerequisites, with scope retained.

### Documentation relevance and review evidence

Remove generic plugin-dashboard/savings/footer requirements and mandates to invoke unavailable context/graph/skill commands from agent instructions. Keep RTK, available graph-first code discovery, bounded document reads, primary sources and durable MEMORY checkpoints. Saving a checkpoint is not manual conversation compaction; this session has no callable manual compaction tool.

Keep historical ADR/record bodies and legacy source intact; update status metadata or add archive notices rather than manufacture Rust results. Reviewers approved planning contracts subject to cross-file reconciliation, which is recorded in the [audit record](./records/2026-11-25-R0-01-specialist-documentation-review.md). Global Markdown checks follow there. Reviewed local work remains REVIEW until actual merge; no commit/PR/merge is authorised implicitly.


## ADR-027: Begin the Reviewed Rust Workspace Baseline

**Date:** 2026-11-25. **Status:** accepted task execution decision; implementation in progress, review/merge pending.

**Authority:** owner request ?lanjutkan implementasi? after the specialist-reviewed R0-01 baseline. This supersedes the documentation-only execution restriction for the next migration task. R0-01 remains REVIEW until merge; the reviewed local baseline is the explicit prerequisite for R0-02 continuation, without manufacturing a DONE status. No legacy source retirement, CI conversion, native icon mutation or host-strategy change is authorised by this baseline.

**Decision:** create the seven Windows-v1 workspace members; pin stable Rust 1.90.0 and Slint 1.13.1 with defaults disabled and winit/software/accessibility enabled. These reproducible baseline pins are not claims of latest versions or final Q-21 selection. The downloaded Slint and slint-build manifests declare Rust 1.85; target-filtered dependency metadata declares no minimum above 1.90. The royalty-free 2.0 licence included in the pinned package is selected under ADR-025, with AboutSlint attribution in the baseline window. Record the actual Cargo graph and preserve the lockfile. Native Windows support still requires the named Win10/Win11 matrix.

**Contracts:** only neutral capability data and an injected clock enter Core now. More domain/ports belong to R1-01. Restore CLI is a renderer-independent route that explicitly fails unavailable until R0-05; Guard cannot announce readiness. A compiled scaffold is never presented as verified icon recovery or a usable desktop organizer.

**Alternatives/costs:** waiting for an unrequested merge would block the authorised continuation; marking R0-01 DONE would falsify the board. Taking the newest release or default Slint backend without validation would introduce unreviewed features. Pinning an older stable baseline requires future security/update review; software rendering may fail later host/performance gates and remains provisional. Required different-role implementation review remains pending. No desktop-safety exception is made.


## ADR-028: Isolate Slint-generated unsafe glue

Date: 2026-11-25. Status: implementation decision pending review.

Evidence: the pinned Slint 1.13.1 generated ItemTreeVTable_static macros fail E0453 under crate-level forbid(unsafe_code). Hand-written Core and UI remain safe-only. The UI root uses deny(unsafe_code), a private generated module alone allows the compiler output, and presentation.rs unconditionally forbids unsafe. No hand-written native bindings or unsafe blocks are authorised.

Alternative: forbidding every generated macro prevents compiling the selected toolkit. Globally allowing unsafe would weaken the shared-code boundary. A separate generated crate adds a workspace member without improving this narrow trust boundary. Cost: compiler output and pinned toolkit internals remain trusted; architecture tests verify the exception remains confined. This refines the literal UI crate-wide prohibition in ADR-025, without changing toolkit, host, schema or product scope. Review and runtime safety evidence remain separate.


## ADR-029: Parallel local continuation with one validation owner

Date: 2026-11-25. Status: accepted execution decision under explicit owner authorisation.

Authority: the owner requested parallel implementation with several agents, and exactly one testing agent executing a sequential queue.

Decision: release-engineer continues R0-03 and core-engineer starts R1-01 against the R0-02 local REVIEW baseline. A single QA agent owns all validation execution and independently reviews the baseline before dependent acceptance. The orchestrator owns shared task/continuity files and acts as architect for proposed Core interfaces. Implementation agents author tests but never execute validation. Existing dirty work and branch are retained; no commit, merge, legacy retirement or desktop mutation is implied. R1-01 contracts precede implementations; native ports remain subject to their spike evidence.

Rationale: this carries out authorised parallel work without overlapping test processes or falsely marking prerequisites DONE. Disjoint implementation ownership prevents file conflicts.

Alternatives/costs: serial implementation wastes the expressly requested parallel capacity; concurrent testers violate the owner instruction. Local dependent work risks rework if baseline review finds defects, and the shared branch requires later task-scoped commit preparation. Tasks remain REVIEW or WIP until their actual acceptance evidence exists; merging remains separate.


## ADR-030: Express configuration commit uncertainty without discarding evidence

Date: 2026-11-25. Status: interface design decision; independent QA review and native adapter evidence pending.

Authority: owner-authorised parallel implementation continuation under ADR-027 and ADR-029. This refines the approved config-persistence contract only; it changes no schema, data location, product scope or desktop-recovery protocol.

Decision: the platform-free `FileSystem` port in `crates/drape-core/src/ports.rs` uses adapter/root-bound file, revision, writer and temporary tokens rather than arbitrary caller paths. Reads and backup discovery are explicitly bounded; missing, access-denied and oversize inputs stay distinct. Exclusive create, complete write and sync are separate operations. Initial publication cannot overwrite an unexpected destination; replacement checks the observed revision and preserves the previous validated bytes in a collision-safe backup. Only owned temporaries and recognized managed backups can be removed.

A commit reports `NotCommitted`, `Committed` or `Indeterminate`. The first guarantees that the destination and existing backups remain unchanged. The second acknowledges actual namespace publication and reports storage-durability confirmation separately from subsequent cleanup. The third represents an adapter fault whose publication result has not been established: preserve config, backup and temporary evidence, stop further saves and forbid automatic retry/prune/cleanup until reconciliation. It never satisfies successful durability acceptance. The existing requirement to recover the last successfully committed valid configuration remains; the eventual native adapter must demonstrate it at supported failure points, rather than infer it from a generic OS error.

Quarantine uses the same verified-moved / verified-unchanged / indeterminate distinction. An indeterminate move stops backup recovery, overwrite and automatic retry until reconciliation establishes source/quarantine identities. Writer exclusion applies to cooperating Drape processes: checking a revision and then replacing is not atomic compare-and-swap against arbitrary external editors. Native adapters document those residual windows, detect conflicts where possible and reject unsupported stronger guarantees before mutation.

Cancellation is observed before mutation. Once commit begins, the adapter must establish/report its actual result; cancellation after publication cannot become a false claim that the save did not happen. Shutdown stops new work and waits for completion on the persistence worker, independently of desktop restoration. No parser defaults, asynchronous runtime, JSON dependency or ConfigStore service implementation is selected here.

Alternatives considered: a single `Result<(), Error>` hides whether bytes were published before a later sync/cleanup failure and invites unsafe retries. Treating every platform error as proof that the namespace is unchanged is an unverified native guarantee. Treating indeterminate state as success, deleting temporary evidence or retrying blindly could destroy the only recoverable bytes. Waiting for the complete parser/native adapter before writing the port would unnecessarily prevent deterministic failure-injection work.

Costs and limitations: implementers must validate opaque token provenance, revision consistency and temporary eligibility, and consumers must handle an explicit stopped-save fault state. Indeterminate publication requires later native reconciliation design and tests; no real filesystem or process-failure experiment is claimed. Numeric parser budgets, alias-conflict handling and lossless wire representation remain prerequisites. Native acceptance, supported Windows builds and power-loss limits are not established by interface compilation or an in-memory fake.

Architecture-check maintenance in the same continuation recognizes the already pinned Windows binding only as a direct dependency of `drape-shell-windows` and its enumerated transitive packages in Guard's closure. It does not allow direct native dependencies in Core, UI, Infrastructure or Guard. Actual-metadata mutation tests retain those boundaries; no broad `windows-*` exception is introduced.

## ADR-031: Lossless schema-1 persistence in Infrastructure

Date: 2026-11-26. Status: accepted for implementation; native adapter evidence pending.

Authority: the owner asked for R1-02 to start while the native R1-01 ports wait for spike evidence. The R1-02 dependency is narrowed to the reviewed `ConfigStore` and `FileSystem` contracts. No schema, data location or product scope change.

Decision: `drape-infrastructure` parses with `serde_json` 1.0.154, already locked through the Slint build graph, with `preserve_order` and `arbitrary_precision`. The stored JSON value is the lossless document; a save rewrites only known fields whose domain value changed, then re-reads its own output before publishing. A separate bounded scan enforces limits and detects duplicate properties and unpaired surrogates before the parser runs. Core gains one port, `PathKeys`. The architecture check allows `serde_json` for Infrastructure and `drape-test-support` as an Infrastructure development dependency only. Accepted forms, classification and limits are specified in [the R1-02 spec](./specs/R1-02-durable-config.md). New documents use the legacy writer's flat `quickHideEnabled` and unhyphenated label modes, because that writer is the only implementation that has produced files.

Alternatives considered: serializing domain structs directly loses unknown values and precision. Typed wire structs with flattened extension maps keep unknown properties but still rewrite accepted spellings and timestamp precision. A hand-written JSON parser adds untested code at the trust boundary.

Costs and limitations: the whole document is held twice in memory during a save. Comparing field by field needs a mapping that must be kept in step with the domain model; the re-read check turns a missed field into a refused save rather than silent loss. `preserve_order` and `arbitrary_precision` apply wherever Cargo unifies this `serde_json` build. Numbers beyond `f64` in known fields are read as invalid.

## ADR-032: Restore after an unmonitored gap only a verified hide

Date: 2026-11-26. Status: accepted owner decision; implementation and native evidence pending.

Authority: the owner answered the R0-05 question raised during the [spec review](./records/2026-11-26-R0-05-spec-review.md): restore if verified.

Decision: when no process monitored the desktop without a gap, for example after both App and Guard died or after a reboot, Drape shows the icons again only if the durable recovery record proves that its own hide was verified, no user change or supersession is recorded, and the icons are currently hidden. When the record shows only an intent or an unverified hide, Drape does not change visibility; it reports the ambiguity and gives manual guidance. Recovery by a process that monitored without a gap is unaffected.

Rationale: a verified hide is the strongest ownership evidence available after a gap, and leaving a desktop hidden is a real loss for the user. An unverified command does not establish that Drape caused the current state.

Alternatives considered: restoring on intent alone never strands the desktop but may override a user's choice on weak evidence. Never restoring automatically cannot override a choice but leaves icons hidden after every double failure.

This applies to every recovery entry, including standalone restore and uninstall. The Security document's uninstall line was reworded to match it and the standing rule that exit paths restore owned changes.

Costs and limitations: a user who hid the icons themselves during the gap will see them shown. A crash between the hide command and its verification can leave icons hidden until the user restores them manually.
