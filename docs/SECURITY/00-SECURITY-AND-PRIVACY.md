# 00 — Security and Privacy

## Principles

1. **Least privilege:** runs as a normal user (`asInvoker`); never asks for admin.
2. **Local only:** no network and no telemetry by default.
3. **Minimal data:** store only what a feature needs; never record input.
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
| Mouse-hook abuse | Code in the app records clicks | The callback forwards only coarse event types; no coordinates stored; no disk writes; hook code reviewed in every PR; static test that Shell assemblies have no network access |
| Config tampered with | Malware edits `config.json` so items point to harmful files | Config sits in the user profile (user-level access); path validation; opening items always goes through the platform open call (Windows `ShellExecute`), which applies the same checks as Explorer (Mark-of-the-Web warnings remain) |
| Path injection / traversal | Malicious path in config or drop | Normalise and validate; never run commands from strings; never use a shell to concatenate arguments |
| Fake update | Update server compromised | Signed packages, hash verification, HTTPS, official channel only; updates are opt-in |
| Supply chain | Malicious NuGet package | Central Package Management, `--locked-mode`, Dependabot, review new packages, vulnerability scan in CI |
| Untrusted DLL / plugin | Third-party context-menu handlers | Isolation with a timeout; never load DLLs from locations writable by other users |
| Leak through logs | Paths and file names in shared logs | Paths only at Debug; "Copy diagnostics" has no paths; warn before sharing logs |
| Antivirus false positive | Global hook flagged | Code signing, open source, documentation, minimise suspicious behaviour |
| Spoofing | Item with a fake icon | Show real extension/tooltip name; icons come from the shell, not from config |

## Global Hook: Special Rules

- Only `WH_MOUSE_LL` (Windows), only for button clicks; **no keyboard hook**. Hotkeys use `RegisterHotKey`.
- No click history is kept. No events leave the process.
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
| Update checks | Off by default; when on, contact only the official release channel | Version number only | — |

If automatic telemetry or crash reporting is ever added: opt-in, a clear field list, no paths/file names, and it requires a new ADR plus an update of this document.

## Development Security

- No secrets in the repo. Signing keys live in CI secret storage or a signing service.
- Secret scanning in CI (gitleaks or equivalent).
- `main` is protected: PR + green CI + review.
- Spikes touching `explorer.exe` or the registry run only in a VM or test account.
- Document how to report vulnerabilities in `SECURITY.md` before the public release.

## User Data Rights

- Export config (anonymised optional) and delete all data from settings ("Reset and delete data").
- Uninstall offers data deletion and **always** restores desktop icons.

## Security Release Checklist

- [ ] No network calls in Shell/Core assemblies (automated test).
- [ ] Hook code reviewed; no input is stored.
- [ ] Dependencies scanned; no open high/critical vulnerabilities.
- [ ] Artifacts signed; checksums published.
- [ ] `SECURITY.md` and a privacy statement available.
