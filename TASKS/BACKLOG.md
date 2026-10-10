# Backlog

Open questions, specification gaps and deliberate deferrals.

**A task that would need a decision `docs/` does not contain is not a task** — it is an entry here, raised with the project owner.

Every item either has a task, a trigger, or an owner question attached; items with no consequence anywhere are noise. Each open question carries a **default assumption** so work is not blocked while it waits.

---

## Open Questions — need an owner decision

**None open as of 2026-10-10.** Q-01 … Q-18 are all closed; Q-04 … Q-13 and Q-18 were answered by the assistant on the owner's instruction to "answer every remaining question", with research ([ADR-022](../MEMORY/DECISIONS.md)). They are working decisions the owner may overturn. New questions are appended below with the next number (Q-19).

| # | Question | Default assumption | Impact if different | Decider | Status |
|---|---|---|---|---|---|
| Q-01 | Stack: is .NET + WPF acceptable, or Rust/Tauri/Electron/other? | — | — | Owner | **Closed — ADR-012** (C# / .NET 10 / Avalonia, chosen by the owner for cross-platform reach) |
| Q-02 | Licence and distribution model: open source (MIT/Apache-2.0) or proprietary/commercial? | MIT | `docs/PLAN/05`, dependencies, signing, release | Owner | **Closed — ADR-019: MIT, free** |
| Q-03 | Final product name other than "Drape"? | Drape (working) | Namespaces, branding, trademark search | Owner | **Closed — ADR-019: "Drape" is final** |
| Q-04 | Distribution: direct (installer + winget) or Microsoft Store (MSIX)? | Direct with Velopack | `docs/DEVOPS/00`, Phase 5; MSIX may restrict the global hook and desktop access | Owner | **Closed — ADR-022: direct distribution (GitHub Releases + Velopack installer + winget); Microsoft Store/MSIX not in v1.0** |
| Q-05 | Minimum OS: Windows 11 only, or keep Windows 10 22H2? | Windows 11 primary, Windows 10 best effort | Test matrix, compatibility burden | Owner | **Closed — ADR-023 (owner decision): Windows 10 22H2 is the minimum and fully supported; Windows 11 is the primary dev target** |
| Q-06 | ARM64 in v1? | No; x64 first | Build, tests, native dependencies | Owner | **Closed — ADR-022: x64 only in v1.0 (runs on Windows 11 ARM by emulation); native ARM64 after v1.0 (B-02)** |
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
| An Avalonia window can be re-parented into the desktop host and stay transparent | P0-03, P0-06 |
| `WM_COMMAND 0x7402` toggles desktop icons reliably on Windows 11 24H2+ | P0-04 |
| 20 transparent/acrylic Avalonia windows fit the CPU and memory budgets | P0-06, P0-08 |
| `WH_MOUSE_LL` plus hit-testing can find "empty desktop" without breaking the context menu | P0-07 |
| Avalonia bitmaps can be created off the UI thread | P0-05 |
| macOS can show a panel at desktop-icon level and hide Finder icons safely | P6-01, P6-02 |
| Linux X11/Wayland desktop-layer windows are feasible per compositor | P6-03, P6-04 |
