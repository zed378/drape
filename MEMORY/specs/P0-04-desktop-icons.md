# Feature Spec — P0-04 Spike: Hide/Show Native Icons and Recovery

**Written:** 2026-10-25 — **before** spike code
**Task:** P0-04
**Author:** code agent
**Spec refs:** [`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](../../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §3 · ADR-006 in [`MEMORY/DECISIONS.md`](../DECISIONS.md) · ADR-008 in [`MEMORY/DECISIONS.md`](../DECISIONS.md)

---

## Problem

Hiding the user's desktop icons is the most dangerous thing this product does. A wrong state read flips the user's desktop; a missed restore strands them. The spike must prove the technique and the recovery before any production code hides anything.

## What `docs/` Already Decides

| Decision | Source |
|---|---|
| Primary technique: `WM_COMMAND` `0x7402` to `SHELLDLL_DefView` (same as the "Show desktop icons" menu item) | PLATFORM §3 |
| The command is a toggle — read state first via `IsWindowVisible(SysListView32)` | PLATFORM §3 |
| Read state before and after; record in `state.json` | PLATFORM §3 |
| Restore on every exit path: normal, exception, Guard, `--restore-desktop`, next start | ARCHITECTURE, ADR-008 |
| Alternatives to evaluate: `ShowWindow(SW_HIDE)` on SysListView32, registry `HideIcons` + broadcast | PLATFORM §3 |
| Never kill `explorer.exe` to hide icons | PLATFORM §3 |

**No gap found.** The question is empirical: which technique is stable across builds.

---

## Layering and Interfaces

- [x] Spike code lives only in `spikes/p0-04-desktop-icons/`.
- [x] Draft `IDesktopIcons` already exists in Core (`AreHidden`, `Hide`, `Restore` — no toggle). The spike validates it.
- [x] No platform type crosses into Core.

---

## Desktop Safety

- [x] **Icon hide path:** the spike itself hides icons — in the VM only, with the owner watching. Checkpoint the VM first.
- [x] **Restore paths tested:** clean exit, hard kill (`taskkill /F`), `--restore-desktop` with the app absent. Each must bring icons back.
- [x] Non-destructive: hiding icons changes no file. The registry alternative is evaluated but must not persistently change user settings.
- [x] Explorer restart (manual, in VM): state re-read, no flip.
- [x] Failure mode: a failed read reports unknown and refuses to send the toggle — never guess.

---

## Data and Config

Not applicable — the spike writes no config. Production will record state in `state.json` (P2-03).

---

## Behaviour

| Rule | Enforcement point |
|---|---|
| Read `IsWindowVisible(SysListView32)` before sending `0x7402`; skip the send when already in the desired state | Spike code |
| Read again after; report mismatch as failure, not success | Spike code + manual check |
| Kill the spike process mid-hidden (`taskkill /F`), then run with `--restore` → icons return | Manual procedure in VM |
| `SW_HIDE` alternative: hide, restart Explorer manually, check whether Explorer re-shows it | Manual check in VM |
| Registry alternative: evaluated for stability only; rejected if it persistently changes settings | Record |

---

## Platform Matrix

| Target | Applies? | Notes / how verified |
|---|---|---|
| Windows 11 (23H2, 24H2+) | Yes | Owner VM |
| Windows 10 22H2 (minimum) | Yes | Owner VM — failure here is a failure |
| DPI / monitors | Partial | State read must work at 150%/200%; multi-monitor noted |
| macOS / Linux | not applicable until Phase 6 | |

---

## Security and Privacy

- [x] No network call.
- [x] No data written to disk.
- [x] No elevation. No injection — `SendMessage` to the shell's own window only.

---

## UI

Not applicable.

---

## Tests

- Spike harness: `spikes/p0-04-desktop-icons/` — `read`, `hide`, `show`, `--restore` commands.
- Manual (owner in VM): hide → verify → show → verify; hide → `taskkill /F` → `--restore` → verify; hide → Explorer restart (manual) → verify state in sync.
- Negative: read failure → refuse to toggle; double-hide without read → must not flip.
- Mutation check: not applicable (spike).

---

## What Was Not Determined

- Whether `SW_HIDE` survives an Explorer restart — measured in the VM.
- Whether the registry path is needed at all — only if both other techniques fail on some build.

---

## Open Questions

| Question | Default assumption | Backlog entry |
|---|---|---|
| None | — | — |
