# Feature Spec — P0-03 Spike: Window Host in the Desktop Layer

**Written:** 2026-10-25 — **before** spike code
**Task:** P0-03
**Author:** code agent
**Spec refs:** [`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](../../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §1–2 · ADR-007 in [`MEMORY/DECISIONS.md`](../DECISIONS.md) · [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](../../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md)

---

## Problem

If a transparent Avalonia window cannot live at the desktop layer on current Windows builds, the product design changes. Everything in Phases 1 and 2 stands on this answer. The owner has a Windows VM ready for testing.

## What `docs/` Already Decides

| Decision | Source |
|---|---|
| Three strategies: A (SetParent to Progman/WorkerW), B (bottom z-order top-level), C (hybrid) | PLATFORM §2 |
| Six acceptance criteria (wallpaper-above, Win+D, no focus, Explorer restart, mixed DPI, transparency) | PLATFORM §2, ADR-007 |
| Never hard-code the hierarchy; discover via FindWindow + EnumWindows | PLATFORM §1 |
| Interim recommendation C behind `IDesktopHost` | PLATFORM §2 |
| One Avalonia window per Drape; handle via `TryGetPlatformHandle()`; re-parent in Shell, not view | PLATFORM §2 |

**No gap found.** The question is empirical, not a spec gap: which strategy passes on real builds.

---

## Layering and Interfaces

- [x] Spike code lives only in `spikes/p0-03-desktop-host/` — never in `src/`.
- [x] Draft `IDesktopHost` contract already exists in Core (`Attach`, `Reattach`, `Place`, `EnsureZOrder`, `Detach`, `Degraded` status). The spike validates it, not redesigns it.
- [x] No platform type crosses into Core. Native handles stay in the spike.
- [x] No capability change: `SupportsDesktopLevelWindow` stays `false` until ADR-007 is Accepted (P0-09).

---

## Desktop Safety

- [x] **Icon hide path:** none — the spike never hides icons.
- [x] **Restore paths:** not applicable.
- [x] Non-destructive: the spike creates a test window and reparents it; it moves no user file.
- [x] Explorer restart: tested manually in the VM (never by killing explorer.exe in a script on the host).
- [x] Failure mode: if attach fails, the spike reports Degraded — it does not crash.

---

## Data and Config

Not applicable — the spike reads no config and writes none.

---

## Behaviour

| Rule | Enforcement point |
|---|---|
| Strategy A: discover host via FindWindow("Progman") → SHELLDLL_DefView child check → else EnumWindows for WorkerW with SHELLDLL_DefView child; send 0x052C to Progman if no WorkerW exists | Spike code + manual verification in Spy++ |
| Strategy B: WS_EX_TOOLWINDOW, SetWindowPos(HWND_BOTTOM, NOACTIVATE), re-assert on location change | Spike code + manual Win+D test |
| Strategy C: try A, verify (IsWindow + parent check), fall back to B | Spike code |
| Each strategy × each build scored against the 6 ADR-007 criteria | Results table in the record |

---

## Platform Matrix

| Target | Applies? | Notes / how verified |
|---|---|---|
| Windows 11 (23H2, 24H2+) | Yes | Owner VM — build number recorded in results |
| Windows 10 22H2 (minimum) | Yes | Owner VM or second VM — failure here is a failure |
| DPI 100% and 150%/200% | Yes | Each tested, screenshots |
| Multiple monitors, mixed DPI, monitor unplug | Yes | If the VM supports it; otherwise stated as not tested |
| macOS / Linux / Android / iOS | not applicable until Phase 6 | |

---

## Security and Privacy

- [x] No network call.
- [x] No new data written to disk.
- [x] No process/DLL injection — `SetParent` reparents the spike's own window only.
- [x] No elevation.

---

## UI

Not applicable — the spike window is a plain test surface (solid colour + label), not the Drape UI.

---

## Tests

- Spike harness (console): `spikes/p0-03-desktop-host/` — strategies A/B/C against a test Avalonia window.
- Manual checklist (owner runs in VM, results recorded):
  - Win+D, Win+, (peek): window stays visible
  - Explorer restart (manual, in VM): window recovers
  - 150%/200% DPI, 2 monitors mixed DPI
  - Focus check: opening the window does not steal focus
  - Alt+Tab: window absent
- Negative cases: attach to a missing host → Degraded, not crash; Explorer restart mid-attach → Reattach recovers.
- Mutation check: not applicable (spike, not production).

---

## What Was Not Determined

- Whether Avalonia's compositor stays transparent when re-parented (risk PR-4) — this is exactly what the spike measures.
- Per-pixel transparency on a child window may misbehave — measured, not assumed.
- Idle CPU/GPU with 20 windows — rough number only in the spike; the real budget is Phase 5.

---

## Open Questions

| Question | Default assumption | Backlog entry |
|---|---|---|
| None — the spike answers the open question | — | — |
