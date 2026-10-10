# 04 — Risk Register

Project and platform risks. Risks are `PR-<n>`. Severity is the consequence if it happens; likelihood is a judgement, stated so it can be argued with. Nothing has been built yet, so every status below is **open** unless stated; a status changes only through a record in [`../../MEMORY/records/`](../../MEMORY/README.md).

## Critical

### PR-1 — The desktop window structure changes between Windows builds

**Severity:** critical · **Likelihood:** medium · **Status:** open — spike P0-03

Drape sits on the desktop layer by relying on the internal arrangement of `Progman` / `WorkerW` / `SHELLDLL_DefView`, which is undocumented and has changed across Windows 10 and 11 (including 24H2).

**Mitigation:** dynamic discovery instead of a hard-coded hierarchy; a layered host strategy with a fallback (strategy B, a bottom-of-z-order window) behind `IDesktopHost` ([ADR-007](../../MEMORY/DECISIONS.md)); a degraded mode that shows Drapes as plain windows.

**Watch:** each Insider/GA build; any change in the six host criteria.

### PR-2 — Hiding native icons leaves the user stuck

**Severity:** critical · **Likelihood:** medium · **Status:** open — spike P0-04

The icon toggle is undocumented and is a *toggle*, not a setter. If the app dies while icons are hidden, the user sees an empty desktop.

**Mitigation:** read state before and after every change; record it in `state.json`; Guard process; `--restore-desktop`; tray command "Restore desktop icons"; restore on the next start; uninstall restores icons ([ADR-006](../../MEMORY/DECISIONS.md), [ADR-008](../../MEMORY/DECISIONS.md)). **This mitigation may not be cut for schedule.**

**Watch:** any new exit path; any code that calls the toggle.

### PR-3 — A global mouse hook is flagged by antivirus or abused

**Severity:** high · **Likelihood:** medium · **Status:** open — spike P0-07

A low-level hook looks like a keylogger-class technique to some products.

**Mitigation:** minimal callback; no recording of input; no keyboard hook; code signing; open documentation; a setting that removes the hook entirely ([`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md)).

**Watch:** every PR touching hook code gets a dedicated review.

## High

### PR-4 — Avalonia may not behave as a desktop-layer window

**Severity:** high · **Likelihood:** medium · **Status:** open — spikes P0-03, P0-06

Re-parenting an Avalonia top-level window's native handle into the desktop host, per-pixel transparency, acrylic/Mica and 20 simultaneous windows are all unproven together. The owner chose Avalonia for cross-platform reach ([ADR-012](../../MEMORY/DECISIONS.md)); the cost of being wrong is rewriting the window layer.

**Mitigation:** early spikes on real hardware with 20 windows; `IDesktopHost` and `IBlurEffect` keep the platform specifics out of the UI; a native-window fallback for the host layer only.

### PR-5 — Memory and CPU budgets with .NET + Avalonia

**Severity:** high · **Likelihood:** medium · **Status:** open — P0-08 baseline

< 100 MB with 20 transparent windows and < 0.5% idle CPU are tight for a managed runtime and a GPU-composited UI.

**Mitigation:** measure at P0-08 and every phase; GC settings, shared resources, icon cache limits; consider fewer layered windows or a trimmed build (Phase 5).

### PR-6 — Virtual and redirected folders

**Severity:** medium · **Likelihood:** high · **Status:** open — spike P0-05

Virtual icons (This PC, Recycle Bin) and OneDrive-redirected Desktop folders cause missing or duplicated items. Reading icons must not trigger OneDrive file hydration.

**Mitigation:** Known Folder API for locations; OneDrive KFM and Files On-Demand in the test matrix; virtual items deferred.

### PR-7 — Cross-platform scope outruns the plan

**Severity:** medium · **Likelihood:** high · **Status:** open

macOS, Linux and mobile each have a different (or missing) desktop-layer story. Promising them before spikes risks a design shaped around a platform that cannot deliver.

**Mitigation:** v1.0 ships Windows only ([ADR-014](../../MEMORY/DECISIONS.md)); capability flags rather than OS checks ([ADR-015](../../MEMORY/DECISIONS.md)); each platform needs a spike and an ADR first (Phase 6).

### PR-8 — Resemblance to a trademarked product

**Severity:** medium · **Likelihood:** low · **Status:** open

**Mitigation:** own name, assets and text; clean-room implementation ([`05-LEGAL-AND-NAMING.md`](./05-LEGAL-AND-NAMING.md)). The name "Drape" is chosen by the owner but not yet trademark-searched ([ADR-019](../../MEMORY/DECISIONS.md)); the search is a Phase 5 task (P5-06). No identifier uses the word "fence" ([ADR-020](../../MEMORY/DECISIONS.md)).

### PR-9 — Windows 10 is the supported minimum but is leaving Microsoft support

**Severity:** medium · **Likelihood:** high · **Status:** open — owner decision ADR-023

Microsoft ended Windows 10 support on 14 October 2025; consumer Extended Security Updates run to 12 October 2027 (Windows 10 22H2 only). After that, users who did not enrol run an unpatched OS, yet Drape promises to support it. Windows 10 also lacks Windows 11 visual APIs (Mica/system backdrop), so every visual feature needs two paths.

**Mitigation:** both OSes are in every release gate; the Windows 10 paths are tested, not assumed; visual features fall back through `IBlurEffect`; revisit the minimum by ADR when Windows 10 usage drops or a needed API is Windows 11-only. The owner set no sunset date.

**Watch:** a feature that requires a Windows 11-only API; StatCounter share of Windows 10; the 12 October 2027 ESU end.

## Traps Worth Remembering

Carried over from the planning phase (formerly "lessons learned"); they are structural, not careless.

| Trap | Consequence |
|---|---|
| Hard-coding the desktop window hierarchy | Breaks on the next Windows build |
| Treating the icon command as a setter | Wrong read of state flips the user's desktop |
| Letting the app die with icons hidden | The user is stranded; a schedule cut here is not acceptable |
| Using `%USERPROFILE%\Desktop` | Wrong on OneDrive-redirected machines; use the Known Folder API |
| Dropping from an elevated process into a non-elevated app | Blocked by UIPI; document it, never escalate |
| A slow global mouse-hook callback | The OS disables the hook and input is affected system-wide |
| Test libraries that change to commercial licences | Check every package licence; Shouldly is used for assertions |
| Returning platform image types from Core | Breaks the portability rule; return neutral pixel data |
