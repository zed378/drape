# 00 — Build, CI and Release

Target Rust + Slint, [ADR-025](../../MEMORY/DECISIONS.md). Existing .NET build/CI files remain untouched. Rust commands become valid after R0-02; Rust CI is R0-03. v1: Windows x64, minimum Windows 10 22H2; Windows 11 primary.

## Environment and controls

- Exact stable Rust toolchain selected/pinned in rust-toolchain.toml at R0-02; Windows MSVC build/link tools and SDK as required.
- rust-analyzer + Slint language-server/editor integration; no .NET workload for future Rust app.
- Cargo.toml: workspace, dependencies/features/measured profiles. Cargo.lock committed. Compatible Slint compiler/runtime versions.
- Supported Windows target/backend/renderer; no Qt or WebView in baseline. Document actual CRT/deployment prerequisites.
- PerMonitorV2/longPathAware/supported OS/asInvoker metadata as applicable to App/Guard.
- Risky tests in checkpointed VMs; never automate killing Explorer.

## Future commands and gates

Valid only after R0-02, not Rust evidence today:

```bash
rtk cargo build --workspace --release --locked
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run -p drape-app
rtk cargo run -p drape-app -- --restore-desktop
```

No --all-features by default: backend/platform combinations can conflict. Documentation-only changes require links/consistency/whitespace/scope checks, not unrelated legacy builds.

## CI plan (R0-03)

1. Checkout, exact toolchain/approved Windows SDK/link tools.
2. Cargo cache keyed by toolchain/target/lockfile/features/backend, trusted-cache policies.
3. Locked Release build, rustfmt, denied-warning Clippy, unit/architecture/integration/doc/supported headless tests. Demonstrate deliberate warning/format failures.
4. Core coverage >=80% using validated tooling such as cargo-llvm-cov; publish reports, not empty-suite metrics.
5. Advisory/licence checks (candidate cargo-audit/cargo-deny), secret scan, reviewed updates. Slint allowlisting retains attribution obligations.
6. UIA/E2E/desktop-host tests on interactive local/self-hosted VMs; do not assume hosted runners have desktop access.

Pin versions of CI helper tools and packaging/signing tools as well as rustc/crates. Validate lockfiles and build metadata, archive test/coverage/resource reports and third-party notices with artefacts, and prevent untrusted pull requests from accessing signing credentials or poisoning trusted release caches. Tag releases build from a reviewed commit and locked dependencies; source/tag identity and hashes travel together. Do not use credentials or broaden network permissions to bypass a missing release prerequisite.

Phase 6 adds portable-crate builds and native adapters on their OS runners after validation. Existing workflow edits belong to implementation.

## Publish and measurement

Ship native drape-app.exe and drape-guard.exe together, including MVP. Artefacts: Drape-<version>-win-x64.zip, Drape-<version>-win-x64-setup.exe, SHA256SUMS.txt. Sign both binaries and installer.

Measure download/installed/individual binary sizes and prerequisites separately. Compare LTO/codegen units/stripping/renderer before selecting profiles. Embedded Slint figures are not Windows application RAM/size promises. Record panic strategy; abort/native crash/forced kill cannot depend on Drop/panic hooks.

SemVer tags define versions. Reproducibility records: commit/toolchain/target/Cargo.lock/features/renderer/linker/SDK/packaging versions; distinguish unsigned reproducibility from signing/timestamps. Native ARM64 after v1; x64-on-ARM unverified.

## Installer and updates

Direct GitHub Releases, per-user installer, portable zip and winget remain the channels. Velopack was selected for C#. P5-03/Q-22 evaluates Rust integration, runtime footprint, Windows minimum, signing, silent switches, rollback and recovery. Do not silently switch installer or promise delta updates; a replacement needs an ADR.

- winget MinimumOSVersion 10.0.19045.0; installer/portable testing on Windows 10 22H2 and Windows 11.
- Autostart chosen in app, never silently enabled by installer.
- Updates off by default, manual/periodic only with opt-in; official HTTPS, authenticated/signed packages, hashes/rollback; no telemetry/default auto-apply.
- Coordinate App/Guard before replacement/removal; restore icons first. Never remove active recovery binaries. Data deletion requires user choice; never delete Desktop files.
- Include Slint attribution/notices; planned route AboutSlint in top-level Settings/About.

### Required update/install failure cases

- Verify the manifest/package signature with a defined trust root before extraction or execution; a hash fetched from the same compromised source is not authentication. Reject invalid signature, wrong target architecture, unsupported OS and path traversal in archives.
- Download/stage without changing the running installation. On disk-full, permission failure, interrupted download, invalid package or failed installation, keep the prior working App + Guard pair and durable recovery state. Never start an old Guard with a new App unless their versioned recovery protocol is explicitly compatible.
- Before committing an update or uninstall, request normal shutdown and verify icon restoration; if restoration cannot be verified, retain the recovery binaries/state, explain the failure and leave manual recovery available. Do not report success while an obsolete Guard still watches a replaced process.
- Rollback cannot overwrite a newer config schema. Preserve a validated pre-update backup; an older executable opens newer schema read-only. Test settings/data-path continuity for installer versus portable mode and prohibit simultaneous conflicting instances across modes.
- Portable mode is a packaging format, not permission to write alongside an executable in a protected directory. Use the documented data location unless an explicitly specified portable-data option is designed and tested.
- Show update/download failures without silently enabling periodic checks. Version checks disclose normal network metadata (for example IP address and HTTP headers) as well as requested release information; no personal paths or usage telemetry are sent.

## Signing, release and incidents

Credentials live in CI secrets/signing service, never source. Missing credentials are not bypassed. Sign App/Guard/installer; unsigned internal development experiments explicitly labelled and checksummed; formal public beta/stable distributions require signed App, Guard and installer, with no missing-credentials bypass.

Channels: dev CI artefacts, beta prerelease tags, stable tags. Release gates: supported CI/Core coverage, named manual safety matrix, App + Guard performance, legacy-schema compatibility, UIA/keyboard/high contrast, installer/update/uninstall recovery, licence/notices, docs/changelog, signed checksums and known limitations.

If icons are stranded: withdraw release, publish standalone --restore-desktop and manual Windows instructions, fix and record incident. See [testing](../TESTING/00-TESTING-STRATEGY.md), [security](../SECURITY/00-SECURITY-AND-PRIVACY.md) and [migration](../ARCHITECTURE/03-RUST-MIGRATION.md).
