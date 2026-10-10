# Changelog

Significant changes to the project, newest first. Coarser than [`MEMORY-INDEX.md`](./MEMORY-INDEX.md); every entry links to a record or an ADR.

Format loosely follows Keep a Changelog. Dates are absolute. There is no released version: Drape is in the planning phase and has no code.

---

## Unreleased

### 2026-10-10 — Windows 10 22H2 is the minimum supported OS ([record](./records/2026-10-10-windows-10-minimum-and-platform-minimums.md))
- **Decided (ADR-023):** owner: Windows 10 22H2 (build 19045) is the minimum and fully supported; Windows 11 is the primary dev target. Assistant: macOS 14, Ubuntu 22.04/Debian 12, Android 12, iOS 18, with a distribution channel per platform.
- **Added:** risk PR-9, backlog B-10/B-11, a minimum-OS-and-distribution table in the platform document.
- **Changed:** Windows 10 is now a release gate, not best effort.

### 2026-10-10 — Every remaining open question answered; README and LICENSE added ([record](./records/2026-10-10-remaining-questions-readme-licence.md))
- **Decided (ADR-022):** direct distribution (GitHub Releases + Velopack + winget), Windows 10 best effort until 12 Oct 2027, x64 only in v1.0, Guard required from the MVP, update checks off by default, id-ID/en-US only, GitHub Actions, one Drape per item, no virtual items, own look.
- **Added:** root `README.md`, `LICENSE` (MIT).
- **Changed:** mobile minimums now follow Avalonia support tiers (Android 12+, iOS 18+).
- **Not verified:** research from web pages only; nothing run.

### 2026-10-10 — Owner decisions: MIT, final name, Windows first, `Fence` renamed to `Drape` ([record](./records/2026-10-10-owner-decisions-licence-name-rename.md))
- **Decided:** MIT licence, free (ADR-019); name "Drape" final but not trademark-searched; Windows first (ADR-014 confirmed); mobile is a companion shortcut-group app, last in the queue (ADR-021).
- **Changed:** every `Fence*` identifier is now `Drape*` (ADR-020, supersedes ADR-016); naming rule CS-10 handles the namespace clash.
- **Closed:** Q-02, Q-03, Q-15, Q-16, Q-17.
- **Not verified:** documents only; nothing compiled.

### 2026-10-10 — Documentation restructured and translated to English ([record](./records/2026-10-10-docs-restructure-and-translation.md))
- **Changed:** all documentation moved from Indonesian to English (ADR-013) and into the layout of the reference repository (ADR-018): `docs/` by category, ADRs in `MEMORY/DECISIONS.md`, `TASKS/` as `PROGRESS` + `BACKLOG` + `PHASE-n`, task IDs `Pn-nn`, `AGENT.md` renamed `AGENTS.md`.
- **Changed:** the stack is recorded as C# / .NET 10 / Avalonia, cross-platform (ADR-012); v1.0 is Windows only and other platforms are Phase 6 (ADR-014).
- **Added:** `docs/PLATFORM/01-PLATFORM-SUPPORT.md`, Phase 6, task P2-13, ADR-012 to ADR-018, open questions Q-15 to Q-18.
- **Fixed (documents only):** several contradictions and layer violations, listed in the record. The name rationale for "Drape" was corrected (ADR-001).
- **Removed:** `docs/adr/`, `TASKS/BOARD.md`, `TASKS/DONE.md`, the M0-M5 task files and the old Indonesian `MEMORY/` files (content merged).
- **Not verified:** nothing was built or run, and no platform claim was tested.

### 2026-10-09 — Planning phase begins ([record](./records/2026-10-09-project-setup.md))
- **Added:** project structure (`docs/`, `MEMORY/`, `TASKS/`, `CLAUDE.md`, agent file), the first set of documents (in Indonesian), five ADR files, task files M0-M5, decisions D-001 to D-011 (now ADR-001 to ADR-011) and open questions Q-01 to Q-14.
- No code, no repository history, no solution.
