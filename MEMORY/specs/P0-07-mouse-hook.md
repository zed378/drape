# Feature Spec — P0-07 Spike: Mouse Hook (Desktop Double-Click and Right-Drag)

**Written:** 2026-10-25 — **before** spike code
**Task:** P0-07
**Author:** code agent
**Spec refs:** [`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](../../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §7 · [`docs/SECURITY/00-SECURITY-AND-PRIVACY.md`](../../docs/SECURITY/00-SECURITY-AND-PRIVACY.md) · ADR-010 in [`MEMORY/DECISIONS.md`](../DECISIONS.md)

---

## Problem

Quick-hide (double-click empty desktop) and right-drag ("create Drape here") need a global mouse hook. A slow `WH_MOUSE_LL` callback gets disabled by the OS and affects input system-wide; it is also what antivirus products flag. The spike must prove the callback is fast enough and the hit-testing works without disturbing the native menu.

## What `docs/` Already Decides

| Decision | Source |
|---|---|
| `WH_MOUSE_LL` on a dedicated thread with a message loop | PLATFORM §7 |
| Callback only: read `MSLLHOOKSTRUCT`, decide coarse event type, push to `Channel<T>`, `CallNextHookEx` | PLATFORM §7 |
| Empty-desktop decision outside the callback: `WindowFromPoint` → confirm SysListView32 → `LVM_HITTEST` or UI Automation | PLATFORM §7 |
| Double-click timing follows `GetDoubleClickTime()` + `SM_CXDOUBLECLK` | PLATFORM §7 |
| No logging/storing of coordinates beyond the feature need | PLATFORM §7, SECURITY |
| Hook failure is non-fatal: features disabled with a message via `IPlatformCapabilities` | PLATFORM §7 |
| No input stored (ADR-010) | ADR-010 |

**No gap found.** The question is empirical: latency and hit-test reliability.

---

## Layering and Interfaces

- [x] Spike code lives only in `spikes/p0-07-mouse-hook/`.
- [x] No Core interface yet — the spike informs `IInputHook` design (Phase 2).
- [x] No input stored or logged. Coordinates are evaluated and discarded.

---

## Desktop Safety

- [x] **Icon hide path:** none.
- [x] **Hook minimalism:** callback does no allocation to speak of, no I/O, no long lock; hands work to another thread. Budget: average under 1 ms end-to-end (OS entry to return).
- [x] Plain right-click must still show the Windows menu — verified manually.
- [x] Antivirus note written after running in the VM.

---

## Data and Config

Not applicable.

---

## Behaviour

| Rule | Enforcement point |
|---|---|
| Double-click on empty desktop detected; on icon/Drape/other window it is not | Spike + manual test |
| Plain right-click still shows the Windows menu | Manual test |
| Callback latency average < 1 ms, measured end-to-end | Spike timestamps |
| No coordinates stored or logged | Code review (no log statement in callback) |

---

## Platform Matrix

| Target | Applies? | Notes / how verified |
|---|---|---|
| Windows 11 | Yes | Owner VM |
| Windows 10 22H2 | Yes | Owner VM |
| DPI 100%/150% | Yes | Hit-testing must work at both |
| macOS / Linux | not applicable until Phase 6 | |

---

## Security and Privacy

- [x] No network call.
- [x] No input stored. No coordinates logged (temporary debug logging in the callback is forbidden, not "temporary").
- [x] No elevation.

---

## UI

Not applicable.

---

## Tests

- Spike harness: `spikes/p0-07-mouse-hook/` — installs hook, prints coarse events + latency stats, exits on Esc.
- Manual: double-click empty desktop / on icon / on spike window; right-click (menu appears); right-drag (no menu breakage).
- Negative: hook install failure → clean message, no crash.
- Latency: end-to-end per-callback microseconds, average + max over 1000 events.

---

## What Was Not Determined

- Whether `LVM_HITTEST` cross-process works reliably or UI Automation is needed — measured in the VM.
- Antivirus reaction — noted after running, not predicted.

---

## Open Questions

| Question | Default assumption | Backlog entry |
|---|---|---|
| None | — | — |
