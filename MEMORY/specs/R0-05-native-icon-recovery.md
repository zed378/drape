# Feature Spec - R0-05 Native icon state and independent recovery

**Written:** 2026-11-26 - **before** implementation  
**Task:** R0-05  
**Author:** orchestrator  
**Spec refs:** `docs/PLATFORM/00-WINDOWS-INTEGRATION.md`, `docs/SECURITY/00-SECURITY-AND-PRIVACY.md`, `docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`, `docs/ARCHITECTURE/03-RUST-MIGRATION.md`, ADR-006, ADR-008, ADR-025

## Problem

The Rust baseline has no verified way to hide or restore Explorer desktop icons. A recovery implementation must survive normal exit, process failure, Guard loss, restart and standalone `--restore-desktop` without toggling an unknown state or restoring over a user's later choice.

## What `docs/` Already Decides

| Decision | Source |
|---|---|
| Discover the current desktop host dynamically; never guess a Progman/WorkerW hierarchy | Windows integration sections 1 and 3 |
| Read icon state before and after any toggle and preserve user-hidden state | Windows integration section 3; ADR-006 |
| Durable ownership intent and Guard readiness must precede hiding | System architecture startup/shutdown; Rust migration |
| App, Guard and CLI coordinate per user/session with generation and OS process identity | System architecture; R0-05 task card |
| Guard death while App lives restores owned changes and disables rehide until rearmed | System architecture; R0-05 task card |
| Restore does not depend on config/log success; Drop and panic hooks are best effort | CLAUDE.md and system architecture |
| Explorer termination and destructive registry/system operations are never automated | AGENTS.md and security spec |

## Layering and Interfaces

- `drape-core` owns neutral recovery records, typed outcomes and clock/serialization contracts only; it has no OS, process, filesystem or UI dependency.
- `drape-shell-windows` owns desktop-state discovery, icon read/toggle/verify, process identity and authenticated local coordination, with RAII for every native resource.
- `drape-guard` owns the minimal restore loop and must not start UI, read user config, or claim readiness without a validated protocol record.
- `drape-app` owns startup/shutdown orchestration and the standalone CLI route; UI is not involved in `--restore-desktop`.
- Any native HWND/PIDL/handle is borrowed or wrapped with its matching releaser. No native handle crosses Core or is stored as a PID-only identity.

## Desktop Safety

The transaction is: acquire per-session ownership; read and classify current icon state; durably write an intent with generation and original state; launch Guard and authenticate readiness; revalidate owner, Guard and host; toggle only when state is known and currently visible; read back and record active ownership. Any failed precondition leaves icons unchanged. Already-hidden icons are never claimed.

Restore is idempotent and verifies the resulting state. It runs on normal shutdown, error/panic best-effort paths, Guard supervision, next startup and `--restore-desktop`. Guard is disarmed only after verified restoration. Indeterminate writes, corrupt/future records, stale generations, ambiguous ownership and Explorer absence retain evidence and fail closed; they never authorize a blind toggle.

Guard loss while App remains alive triggers immediate restoration and disables all further icon mutation until a new authenticated Guard is armed. Standalone restore first asks a validated live owner to restore and otherwise uses only a non-ambiguous durable record. No path moves, renames or deletes user files.

## Data and Config

The recovery record is separate from `config.json`, versioned, bounded and stored below the app-owned data root. It contains schema version, user/session scope, generation, authenticated process identity, original/observed icon state, transaction phase and retry metadata. Unknown or newer records are preserved and reported as unsafe; they are never migrated by guessing. Recovery records are not subject to R1-02 config backup/quarantine semantics.

## Behaviour

| State/event | Required result |
|---|---|
| Unknown host or icon state | Return unavailable; do not toggle |
| Guard launch/readiness failure | Keep icons visible/unchanged; no active ownership |
| Crash after intent before hide | Next start reconciles observation and does not invert state |
| Crash after hide before active record | Guard/next start restores only the recorded owned change after revalidation |
| User changes visibility while active | Drop ownership and do not restore over the user's choice |
| Guard dies while App lives | Restore owned change, mark rearm required, suppress rehide |
| Restore/exit/startup race | Per-session coordinator serializes it; stale generation is rejected |
| Explorer unavailable during restore | Retain retryable state and return pending/failed, never claim success |

## Platform Matrix

| Target | Applies? | Notes / how verified |
|---|---|---|
| Windows 11 23H2/24H2+ | yes | interactive VM, exact build recorded |
| Windows 10 22H2 | yes | release minimum, interactive VM, exact build recorded |
| DPI/monitors | indirect | host discovery must be revalidated on mixed-DPI two-monitor setup |
| macOS/Linux/mobile | no | Phase 6 |

## Security and Privacy

No network, elevation, registry write or Explorer termination. IPC is same-user/session, bounded, authenticated to the current process generation, rejects replay/stale acknowledgements and redacts paths/identifiers from normal logs. Recovery state contains no input history and no user file contents.

## UI

No UI is implemented by R0-05. Callers receive typed restored/already-safe/pending/failed/unavailable results suitable for later accessible settings and emergency-restore surfaces.

## Tests

- Core: deterministic state-machine tests for every transaction phase, stale generation, user override, idempotent restore and malformed/future records.
- Shell/Guard integration: fake coordinator and failure injection around intent, readiness, hide, verify, disarm, Guard loss and Explorer absence.
- CLI: `--restore-desktop` bypasses renderer startup and reports typed outcomes.
- Native manual: Windows 10 22H2 and Windows 11 exact builds; normal exit, App kill, Guard loss, hard reboot/next start, Explorer restart and concurrent restore. Explorer restart/termination is manual only.
- Mutation checks: removing read-before-toggle, readiness revalidation or generation validation must fail named tests.

## What Was Not Determined

The authoritative Windows icon state read and toggle mechanism, IPC transport, process-identity API details and bounded retry timings require the R0-05 spike and Windows VM evidence. No production adapter is authorized until those results are recorded and reviewed.

## Open Questions

| Question | Default assumption | Backlog entry |
|---|---|---|
| Which verified state-read/toggle technique survives supported Windows builds? | Evaluate documented alternatives in the spike; fail closed until selected | R0-05 / Q-21 |
