# 04 — Risk Register

Project and platform risks. Risks are `PR-<n>`. Severity is the consequence if it happens; likelihood is a judgement, stated so it can be argued with. An early C# implementation exists, but no Rust migration evidence exists; each risk remains **open** unless recorded otherwise; a status changes only through a record in [`../../MEMORY/records/`](../../MEMORY/README.md).

## Critical

### PR-1 — The desktop window structure changes between Windows builds

**Severity:** critical · **Likelihood:** medium · **Status:** open — spike R0-04

Drape sits on the desktop layer by relying on the internal arrangement of `Progman` / `WorkerW` / `SHELLDLL_DefView`, which is undocumented and has changed across Windows 10 and 11 (including 24H2).

**Mitigation:** dynamic discovery instead of a hard-coded hierarchy; a layered host strategy with a fallback (strategy B, a bottom-of-z-order window) behind `DesktopHost` ([ADR-007](../../MEMORY/DECISIONS.md)); a degraded mode that shows Drapes as plain windows.

**Watch:** each Insider/GA build; any change in the six host criteria.

### PR-2 — Hiding native icons leaves the user stuck

**Severity:** critical · **Likelihood:** medium · **Status:** open — spike R0-05

The icon toggle is undocumented and is a *toggle*, not a setter. If the app dies while icons are hidden, the user sees an empty desktop.

**Mitigation:** read state before and after every change; record it in `state.json`; Guard process; `--restore-desktop`; tray command "Restore desktop icons"; restore on the next start; uninstall restores icons ([ADR-006](../../MEMORY/DECISIONS.md), [ADR-008](../../MEMORY/DECISIONS.md)). **This mitigation may not be cut for schedule.**

**Watch:** any new exit path; any code that calls the toggle, including quick-hide. Guard readiness and durable intent precede mutation; App/Guard/CLI serialise per-session generation ownership. Guard death while App survives triggers restoration/no rehide until rearmed. Explicit restore must not be undone by a live session. Missing hardware or a hosted runner is not a safety waiver.

### PR-3 — A global mouse hook is flagged by antivirus or abused

**Severity:** high · **Likelihood:** medium · **Status:** open — spike R0-08

A low-level hook looks like a keylogger-class technique to some products.

**Mitigation:** minimal callback; no recording of input; no keyboard hook; code signing; open documentation; a setting that removes the hook entirely ([`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md)).

**Watch:** every PR touching hook code gets a dedicated review.

## High

### PR-4 — Slint may not behave as a desktop-layer window

**Severity:** high · **Likelihood:** medium · **Status:** open — spikes R0-04, R0-07

Re-parenting a Slint top-level window's native handle into the desktop host, per-pixel transparency, acrylic/Mica and 20 simultaneous windows are all unproven together. The owner chose Rust + Slint for resource usage, performance, memory safety and size ([ADR-025](../../MEMORY/DECISIONS.md)); the cost of being wrong is rewriting the window layer.

**Mitigation:** early spikes on real hardware with 20 windows; `DesktopHost` and `BlurEffect` keep the platform specifics out of the UI; a native-window fallback for the host layer only.

### PR-5 — Memory, CPU and distribution budgets with Rust + Slint

**Severity:** high · **Likelihood:** medium · **Status:** open — R0-09 baseline

< 100 MB with 20 transparent windows and < 0.5% idle CPU are unproved for the selected Slint backend/renderer, caches, native resources and Guard; Rust alone does not establish either budget.

**Mitigation:** measure at R0-09 and every phase; bounded caches/queues/workers, shared assets and no idle redraw; compare renderers and measured LTO/stripping profiles in Phase 5. A change to one-window-per-Drape needs a new ADR.

### PR-6 — Virtual and redirected folders

**Severity:** medium · **Likelihood:** high · **Status:** open — spike R0-06

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

Windows 10 normal support ended on 14 October 2025. Extended Security Updates eligibility/duration depend on programme, edition, region and enrolment; verify current [Microsoft lifecycle information](https://learn.microsoft.com/lifecycle/products/windows-10-home-and-pro) before release. Do not infer patch coverage from Drape's supported minimum. The old fixed consumer-ESU end date is not a universal guarantee. Windows 10 lacks Windows 11 visual APIs (Mica/system backdrop), so effects need tested fallbacks.

**Mitigation:** both OSes are in every release gate; the Windows 10 paths are tested, not assumed; visual features fall back through `BlurEffect`; revisit the minimum by ADR when Windows 10 usage drops or a needed API is Windows 11-only. The owner set no sunset date.

**Watch:** Windows 11-only APIs, toolchain/backend OS minimums and current lifecycle/ESU terms. Drape's owner-set minimum has no sunset date and is not silently changed by upstream lifecycle policy.

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
| Test libraries that change to commercial licences | Check pinned dependency licences, including Slint attribution requirements |
| Returning platform image types from Core | Breaks the portability rule; return neutral pixel data |

### PR-10 — Rust native unsafe and recovery assumptions

Severity high; likelihood medium; open. Incorrect native pointer/handle lifetime, COM apartment use or FFI unwind can still corrupt memory or abort the app. Drop/panic hooks do not handle forced termination. Mitigate with narrow reviewed unsafe wrappers, correct RAII, no FFI unwinding and independently armed Guard/durable recovery before hiding (R0-05).

### PR-11 — Slint licence, accessibility and integration gaps

Severity high; likelihood medium; open. Slint is not MIT; exact attribution/dependency terms must be checked at pinning/distribution. Native UI does not prove Windows UIA, OLE drag/drop, transparent child-window compatibility or mobile support. R0-02/R0-04/R0-07 and Phase 6 spikes establish evidence; do not invent API equivalence to Avalonia.

### PR-12 — Migration is mistaken for completed Rust work

Severity high; likelihood medium; open. Legacy DONE cards, 98 reported tests and C# measurements can falsely imply a Rust baseline. Preserve dated history, fresh R0/R1 tasks, explicit parity mapping and new named tests. No schema break, old-code deletion, CI rewrite or implementation occurs in the documentation-only change.
