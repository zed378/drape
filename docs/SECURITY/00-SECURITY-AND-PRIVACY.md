# 00 — Security and Privacy

## Principles

1. **Least privilege:** runs as a normal user (`asInvoker`); never asks for admin.
2. **Local only:** no network and no telemetry by default.
3. **Minimal data:** store only what a feature needs; no persistent or logged input history. Bounded transient gesture state is discarded immediately after the decision/cancellation.
4. **Transparent:** what the hook does, what is stored and where is written in the user documentation.
5. **Reversible:** the user can restore their desktop and delete all data at any time.

## Assets Protected

| Asset | Why |
|---|---|
| List of user file paths in the config | Can reveal contents and habits |
| Global mouse input | Potential surveillance if abused |
| Integrity of the user's desktop | Icons not restored = real impact |
| Supply-chain integrity (packages, installer, updates) | A compromised build is a compromised user |

## Threat Model (summary)

| Threat | Scenario | Mitigation |
|---|---|---|
| Mouse-hook abuse | Code records clicks or exports gesture data | Forward only bounded transient fields needed for hit-test/gesture decisions; no logged/persistent history or network path. Review callbacks, consumers and queue-overflow cancellation, not only the enqueue function |
| Config tampered with | Config points to harmful files or handlers | Parse/validate bounded data; treat paths as untrusted and open only after explicit user interaction through the platform port. Windows shell execution delegates file associations/security behaviour; it does not guarantee identical warnings to every Explorer path. Verify Mark-of-the-Web handling for supported file types/verbs and never suppress warnings |
| Path injection / traversal | Malicious path in config or drop | Normalise and validate; never run commands from strings; never use a shell to concatenate arguments |
| Fake update | Update server compromised | Signed packages, hash verification, HTTPS, official channel only; updates are opt-in |
| Supply chain | Malicious crate/dependency | Cargo.lock, locked builds, reviewed minimal features, dependency updates, advisory/licence checks and secret scanning |
| Untrusted DLL / plugin | Third-party context-menu handlers | Apartment-correct bounded invocation with timeout; a thread is not crash isolation. Never load DLLs from locations writable by other users; separate-process containment requires its own reviewed design |
| Leak through logs | Paths and file names in shared logs | Paths only at Debug; "Copy diagnostics" has no paths; warn before sharing logs |
| Antivirus false positive | Global hook flagged | Code signing, open source, documentation, minimise suspicious behaviour |
| Spoofing | Item with a fake icon | Show real extension/tooltip name; icons come from the shell, not from config |
| Recovery-state/IPC spoofing | A malformed/stale state file or another session asks Guard to restore, disarm or wait on an unrelated process | Validate version, bounds, ownership generation and user/session/process identity; never trust PID, pathname or request text alone. Restrict recovery IPC to intended local principals and require matching transaction identity. No arbitrary command/path/process-control messages |
| Recovery race | App, Guard and restore CLI read state then toggle independently; a second App restores a live owner's desktop | Serialise per-user/session recovery mutations and singleton ownership before acting; correlate Guard readiness with the current generation, read back state, and refuse stale disarm/restore messages |
| Protection disappears | Guard dies while App continues hiding icons | App observes Guard health/exit, restores owned changes and disables further hides until rearmed; test loss immediately before/after hide and during rearming |

## Global Hook: Special Rules

- Only `WH_MOUSE_LL` (Windows), only for button clicks; **no keyboard hook**. Hotkeys use `RegisterHotKey`.
- No click history is kept. Minimal coordinates/timestamps needed for right-drag/double-click decisions stay in bounded transient memory; they are cleared on completion, overflow, timeout, disable or session/host loss. Hook/input events never leave the App process, including via Guard IPC.
- The feature can be switched off in settings; when off, the hook is removed completely.
- On macOS a `CGEvent` tap requires a user-granted permission and is subject to the same rules; on Wayland there is no global hook and the feature is simply unavailable ([`../PLATFORM/01-PLATFORM-SUPPORT.md`](../PLATFORM/01-PLATFORM-SUPPORT.md)).
- Document this behaviour in the README and the "About" window.

## Privacy

| Data | Stored? | Sent? | Retention |
|---|---|---|---|
| Config (paths, positions, style) | Yes, local | No | Until the user deletes it/uninstalls |
| Logs | Yes, local | No | 14 days |
| Snapshots | Yes, local | No | Until deleted |
| Telemetry / analytics | None | — | — |
| Crash reports | Local only; the user chooses whether to share | Not automatically | Delete manually |
| Update checks | Off by default; enabled checks contact the official release channel | Necessary version/platform/channel request metadata; the remote service also observes IP address, request timing and ordinary protocol headers | Provider-side access-log retention is outside Drape's local log policy; document the selected release provider before enabling checks |

If automatic telemetry or crash reporting is ever added: opt-in, a clear field list, no paths/file names, and it requires a new ADR plus an update of this document.

## Development Security

- No secrets in the repo. Signing keys live in CI secret storage or a signing service.
- Secret scanning in CI (gitleaks or equivalent).
- `main` is protected: PR + green CI + review.
- Spikes touching `explorer.exe` or the registry run only in a VM or test account.
- A VM is not permission to automate destructive system changes or terminate Explorer. Explorer restart is manual; registry changes need explicit owner approval and a documented restore path. Do not weaken this rule to make an experiment convenient.
- Document how to report vulnerabilities in `SECURITY.md` before the public release.

## User Data Rights

- Export config (anonymisation optional); treat nested unknown fields, snapshots, paths and diagnostic text as potentially identifying too. A scrubbed export is not guaranteed anonymous unless its supported field policy has been tested. Deleting/resetting app data requires explicit confirmation and must first complete/retain pending desktop recovery; never remove the only recovery record while icons remain hidden.
- Uninstall offers data deletion and **always** attempts to restore every Drape-owned icon change. When ownership cannot be established ([ADR-032](../../MEMORY/DECISIONS.md)), it keeps the recovery evidence and reports pending manual recovery instead of claiming success.

## Security Release Checklist

- [ ] No network calls in shell/Core crates (automated test).
- [ ] Hook and gesture-consumer code reviewed; no persistent/logged input history, and bounded transient state is discarded on every completion/cancellation path.
- [ ] App/Guard/CLI ownership and session checks, Guard loss/rearming races and malformed/legacy/newer recovery records tested; no blind visibility toggle or unrelated process control.
- [ ] Untrusted config/drop/IPC size, count and encoding limits tested; no automatic item execution or physical file mutation on load/drop.
- [ ] Dependencies scanned; no open high/critical vulnerabilities.
- [ ] Artifacts signed; checksums published.
- [ ] `SECURITY.md` and a privacy statement available.

## Rust memory and lifecycle boundaries

Core/shared UI forbid unsafe. Native adapters use reviewed minimal unsafe blocks, validated buffer lengths, correct RAII and COM apartments, borrowed-handle lifetimes and no unwinding across FFI. Safe Rust does not remove native API or resource exhaustion risks; bound queues/cache/workers. Guard readiness plus durable ownership intent precedes hiding; panic hooks/Drop are best effort, not forced-kill/abort/reboot guarantees. Log/config failures cannot block restoration. Record and validate runtime-state compatibility rather than ignoring old recovery records.

## Local recovery trust boundary

App, Guard and the standalone restore path form one recovery protocol in the current user's interactive session. Serialise state changes before checking/recovering ownership, so launching a second instance never restores a still-live owner's icons. A successful normal-instance mutex check alone is not an authenticated recovery transaction. Define how process handles are acquired/passed, session/user identity is validated, stale generations are rejected and readiness/disarm acknowledgements are correlated; R0-05 must supply the protocol spec and negative tests before any hide.

Recovery IPC accepts only the minimal fixed commands/data required by that protocol, with bounded messages, local-principal access checks and no arbitrary executable/file paths. Guard must never accept an arbitrary PID and act on an unrelated process; it waits on the validated intended process and does not terminate it. A same-user fully compromised process can often modify user-level state anyway; local ACLs and validation protect against accidental/cross-session/malformed requests, not a claim of isolation from all same-user malware.

State corruption, unsupported versions and unknown visibility are explicit error states. Preserve diagnostic evidence, keep/restore safe owned state where verifiable, expose manual Windows restoration guidance, and never blindly toggle based on an untrusted flag. Retry pending restoration when Explorer is unavailable; do not mark it successful merely because the HWND disappeared. Guard termination while App lives requires immediate fail-closed restoration and no further hide until rearmed. A timeout never authorises unsafe thread/process termination.

## Input and opening boundaries

Validate config, import, drop and recovery payload sizes/counts/depth before allocation-heavy work. Preserve supported path encodings without lossy command construction; reject unsupported payloads clearly. Enumerating/watching/config loading never executes a file, follows a request to open a URL, or invokes a shell verb. User-initiated item opening uses the approved native port without concatenated shell commands or elevated fallback. Native shell extensions remain external executable code; moving them to an STA thread does not sandbox them.

These are target contracts and review requirements, not results of an implementation or penetration test. Primary Win32 API contracts and the pinned Slint backend must be checked during the corresponding R0 experiments; no desktop manipulation was performed for this review.
