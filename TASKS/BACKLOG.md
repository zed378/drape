# Backlog

Open questions, specification gaps and deliberate deferrals.

**A task that would need a decision `docs/` does not contain is not a task** — it is an entry here, raised with the project owner.

Every item either has a task, a trigger, or an owner question attached; items with no consequence anywhere are noise. Each open question carries a **default assumption** so work is not blocked while it waits.

---

## Open Questions — need an owner decision

**Q-19–Q-23 are open as of 2026-11-25.** Rust + Slint is accepted by ADR-025. Earlier Q-01–Q-18 answers remain dated history except where superseded below; packaging and support assumptions need Rust-specific verification.

| # | Question | Default assumption | Impact if different | Decider | Status |
|---|---|---|---|---|---|
| Q-01 | Stack: is .NET + WPF acceptable, or Rust/Tauri/Electron/other? | — | — | Owner | **Closed — ADR-025: Rust + Slint, owner selected; supersedes ADR-012 C#/.NET/Avalonia** |
| Q-02 | Licence and distribution model: open source (MIT/Apache-2.0) or proprietary/commercial? | MIT | `docs/PLAN/05`, dependencies, signing, release | Owner | **Closed — ADR-019: MIT, free** |
| Q-03 | Final product name other than "Drape"? | Drape (working) | Namespaces, branding, trademark search | Owner | **Closed — ADR-019: "Drape" is final** |
| Q-04 | Distribution: direct (installer + winget) or Microsoft Store (MSIX)? | Direct with Velopack | `docs/DEVOPS/00`, Phase 5; MSIX may restrict the global hook and desktop access | Owner | **Channel decision retained: direct GitHub Releases + per-user installer + portable zip + winget; Velopack implementation preference requires Rust validation under Q-22/ADR-025; Store/MSIX deferred** |
| Q-05 | Minimum OS: Windows 11 only, or keep Windows 10 22H2? | Windows 11 primary, Windows 10 best effort | Test matrix, compatibility burden | Owner | **Closed — ADR-023 (owner decision): Windows 10 22H2 is the minimum and fully supported; Windows 11 is the primary dev target** |
| Q-06 | ARM64 in v1? | No; x64 first | Build, tests, native dependencies | Owner | **Closed — ADR-022: x64 only in v1.0 (x64-on-ARM compatibility remains unverified); native ARM64 after v1.0 (B-02)** |
| Q-07 | UI languages besides id-ID and en-US? | Only id-ID and en-US | Resources, QA | Owner | **Closed — ADR-022: id-ID and en-US only; translations by contribution later** |
| Q-08 | Is `Drape.Guard` required from the MVP? | Yes | If not: recovery only via `state.json` and the tray | Owner / architect | **Closed — ADR-022: Guard is required from the MVP** |
| Q-09 | May one item live in several Drapes? | No (one Drape per item, ADR-011) | Data model, UI, rules | Owner | **Closed — ADR-022: one Drape per item in v1.0 (ADR-011); B-03 stays deferred** |
| Q-10 | Repo and CI: GitHub + GitHub Actions? | Yes | `docs/DEVOPS/00`, P0-02 | Owner | **Closed — ADR-022: GitHub + GitHub Actions** |
| Q-11 | Must virtual items (This PC, Recycle Bin) exist in v1? | No, after v1 | Adds shell complexity (PIDL) in Phases 3/4 | Owner | **Closed — ADR-022: no virtual items in v1.0 (B-01)** |
| Q-12 | How much visual resemblance to the original product versus a distinct look? | Own look, equivalent function | Legal and UI | Owner | **Closed — ADR-022: own look, equivalent function** |
| Q-13 | Will other agents/people work in parallel, or a single agent? | Several agents in parallel per `AGENTS.md` | Role split and WIP limit | Owner | **Closed — ADR-022: several agents in parallel under AGENTS.md; WIP 3; at most 4 concurrent subagents** |
| Q-14 | Is an `AGENTS.md` (common convention) needed besides `CLAUDE.md`? | Yes — adopted, replaces the old `AGENT.md` | — | Owner | **Closed — ADR-018** |
| Q-15 | Scope of v1.0 across platforms: Windows only, or Windows + macOS + Linux? | Windows only; others in Phase 6 (ADR-014) | Roadmap length, CI matrix, staffing | Owner | **Closed — ADR-014 confirmed by the owner: Windows first** |
| Q-16 | What is a Drape *item* on Android/iOS, and is mobile in scope at all? | Undefined; design card P6-09 decides | Whether Phase 6 mobile tasks exist | Owner | **Closed — ADR-021: companion shortcut groups, decided by the assistant on the owner's delegation** |
| Q-17 | Keep the internal identifier `Fence*`, or rename (e.g. `Panel*`) before public release? | Keep (ADR-016) | Every namespace, type and config key | Owner | **Closed — ADR-020: everything is called Drape** |
| Q-18 | Are periodic update checks acceptable at all, given "no network by default"? | Opt-in only, off by default (ADR-017) | Updater design, privacy statement | Owner | **Closed — ADR-022: updates off by default; manual check button and an opt-in periodic check** |
| Q-19 | Must blur/acrylic be enabled in the MVP, or is translucent colour sufficient initially? | Keep planned blur capability/fallback and existing feature phases; no scope removal. MVP priority unconfirmed. | UI scope/performance settings | Owner | Open; architecture docs proceed without deciding this |
| Q-20 | What tighter RAM target and numeric installed/download-size ceiling should Rust meet? | Retain existing App + Guard <100 MB ceiling and measure full footprint before proposing tighter targets | Performance acceptance and renderer/profile choices | Owner with R0-09 measurements | Open; no invented target or achieved saving |
| Q-21 | Which Slint backend/renderer and supported native-handle/drop/UIA integration meet the Windows matrix? | Minimal eligible native deployment without WebView/Qt; leave icons visible if safe hosting cannot be established | Host/lifetime/accessibility/drag-drop/rendering | Architect after R0-04/R0-07 evidence | Open technical decision; R0-10 records result |
| Q-22 | Can the existing Velopack preference package/update native Rust App + Guard within requirements? | Keep channels, evaluate actual Rust integration/footprint/rollback/uninstall before selecting implementation | Packaging, runtime size, code signing, recovery | Release engineer / architect in P5-03; owner for material scope changes | Open validation; replacement needs ADR |
| Q-23 | How do pinned panels, tabs and portal projections map to durable identity, membership and schema? | Preserve feature scope; keep single-owner membership distinct from read-only folder projections; no invented schema fields or language-only version bump | P4-01/P4-04/P4-07 specs, future config migrations, snapshots/import compatibility | Architect and core/UI owners; owner decides material behaviour/scope changes | Open specification prerequisite before these features are implemented |


## Deferrals

| # | Idea | Origin | Notes |
|---|---|---|---|
| B-01 | Virtual items (This PC, Recycle Bin, Network) | Q-11 | After v1; needs PIDL |
| B-02 | Full ARM64 support | Q-06 | After x64 is stable |
| B-03 | One item in several Drapes | Q-09 | If requested |
| B-10 | Flatpak/Snap/.deb packaging for Linux | ADR-023 | Sandboxing likely conflicts with desktop-layer features |
| B-11 | Mac App Store distribution | ADR-023 | Sandbox probably forbids the needed behaviour; unverified |
| B-09 | Microsoft Store / MSIX distribution | Q-04 | Needs `startupTask` instead of the HKCU Run key, and a Store-policy check for global hooks; both unverified |
| B-04 | Community themes / sharing styles | roadmap | After v1 |
| B-05 | "Drape per project" tied to a working folder | roadmap | Idea |
| B-06 | Third-party plugins | architecture | After v1 |
| B-07 | User documentation and website | Phase 5 | After features stabilise |
| B-08 | Widgets (clock, weather) | PRD | After v1 |

## Unverified Claims

Things the documents state but nobody has proven. Each is settled by a card; until then treat it as a hypothesis, not a fact.

| Claim | Settled by |
|---|---|
| Winget accepts a Velopack-built installer `.exe` (needs silent switches in the manifest) | P5-03 |
| A packaged (MSIX) app's HKCU writes are redirected, so an HKCU `Run` entry would not start it at logon | B-09, if ever pursued |
| x64 build runs correctly on Windows 11 ARM, including the desktop-layer and hook features | before announcing ARM support (B-02) |
| An actual Slint window can be re-parented safely and remain transparent | R0-04, R0-07 |
| `WM_COMMAND 0x7402` toggles desktop icons reliably on Windows 11 24H2+ | R0-05 |
| 20 Slint windows plus Guard fit CPU/RAM/footprint budgets | R0-07, R0-09 |
| `WH_MOUSE_LL` plus hit-testing can find "empty desktop" without breaking the context menu | R0-08 |
| Owned pixel extraction and Slint UI-thread image conversion meet throughput and lifetime requirements | R0-06, R0-07 |
| macOS can show a panel at desktop-icon level and hide Finder icons safely | P6-01, P6-02 |
| Linux X11/Wayland desktop-layer windows are feasible per compositor | P6-03, P6-04 |

## Migration assumptions (ADR-025)

No code retirement, schema change or product feature removal is implied by the documentation request. Exact toolchain/crate versions and backend/rendering APIs are intentionally unselected until R0-02/spike evidence. Slint is an explicit royalty-free dependency-policy exception requiring attribution; Drape remains MIT. Preserve old C# records and use fresh R0/R1 tasks. Former Phase 0/1 status contradictions are preserved as historical snapshots, not counted as active WIP. Old OS minimums outside Windows remain planning targets requiring Rust/Slint validation.

## R0-06 enumeration integration prerequisites — 2026-11-25

The [passive enumeration continuation](../MEMORY/records/2026-11-25-enumeration-persistence-contracts.md) retains raw literal paths rather than inventing Windows comparison keys. Before implementing `DesktopItemSource`, review case-sensitive directories, extended/UNC paths, duplicate private/public roots and verified Explorer visibility settings. Caller-supplied enumeration budgets are probe inputs, not selected production defaults. No native hydration/timing acceptance is inferred from metadata-only source code.

## R1-02 compatibility prerequisites — 2026-11-25

Source review found that the legacy writer's flat quick-hide property and enum spellings differ from the abridged documentation example. Default assumption: retain both supported representations losslessly, preserve timestamp precision, and refuse ambiguous/conflicting aliases until their handling is specified. No schema change is implied. The architect and Core/Infrastructure implementers must settle these cases, filesystem operation signatures and parser resource budgets in [the R1-02 spec](../MEMORY/specs/R1-02-durable-config.md) before persistence implementation. Limits need hand-written sanitized fixtures for both the 500-desktop-item matrix and the legacy 2,500-reference serialization profile; no numeric cap is inferred from either profile. See the [continuation record](../MEMORY/records/2026-11-25-icon-cache-validation-continuation.md).

## Parallel continuation assumption — 2026-11-25

The owner explicitly authorised parallel implementation and one sequential testing agent. R0-03 and R1-01 continue locally against the R0-02 REVIEW baseline; QA independently reviews that baseline before dependent acceptance. Work remains on the existing feat/R0-03-rust-ci branch to preserve the dirty shared workspace; no commit, merge or DONE is implied. Native desktop spikes retain their VM/safety prerequisites.

The next continuation develops R1-03 over the reviewed R1-01 data subset and R0-04 as a read-only native-handle diagnostic. Architect-approved routine layout choices: exact opaque monitor identity; duplicate matches use the unique primary; absent/multiple primaries when fallback is required return an error; snap distance ties prefer work-area edges, then supplied neighbor order, then grid. No default native-host strategy is selected by these choices. Full prerequisite acceptance and native Windows matrix evidence remain pending.
