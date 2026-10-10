# 00 — Build, CI and Release

Status: plan. Distribution is direct — GitHub Releases, a Velopack per-user installer, a portable zip and a winget manifest ([ADR-022](../../MEMORY/DECISIONS.md)); the Store/MSIX path is deferred (B-09). v1.0 ships for Windows only ([ADR-014](../../MEMORY/DECISIONS.md)).

## Development Environment

1. Windows 11, .NET SDK per `global.json` (verified in P0-01).
2. IDE: Visual Studio or Rider (".NET desktop development" workload), or VS Code with C# Dev Kit.
3. `dotnet restore && dotnet build Drape.sln && dotnet test Drape.sln`.
4. Hyper-V VMs for Windows 11 and Windows 10 for risky tests (Explorer restart, display changes). Take a checkpoint before each test.

## Build Configuration

- `global.json`: pins the SDK (`rollForward: latestFeature`).
- `Directory.Build.props`: `Nullable`, `TreatWarningsAsErrors`, `LangVersion`, analyzers, `Deterministic`, `ContinuousIntegrationBuild` in CI.
- `Directory.Packages.props`: Central Package Management.
- `.editorconfig`: code style and analyzer severity.
- Version derived from Git tags (MinVer or equivalent): `v0.1.0`, `v0.2.0-beta.1`.

## CI (GitHub Actions) — plan

Workflow `ci.yml` on `push` and `pull_request`, runner `windows-latest`:

1. Checkout, set up .NET per `global.json`.
2. Cache NuGet.
3. `dotnet restore --locked-mode`.
4. `dotnet build -c Release --no-restore -warnaserror`.
5. `dotnet format --verify-no-changes`.
6. `dotnet test -c Release --no-build --collect:"XPlat Code Coverage"` (without E2E).
7. Upload coverage and test results.
8. Secret scan (e.g. gitleaks) and package vulnerability check (`dotnet list package --vulnerable`).

Portable projects (`Core`, `Infrastructure`, `UI`) and their tests are also built and run on `ubuntu-latest` and `macos-latest` once Phase 6 starts, to keep portability honest. Until then the architecture tests are the portability guard.

Workflow `release.yml` on tag `v*`: build Release, publish, build installer, sign, compute SHA-256, create a GitHub Release with change notes.

Dependabot (or Renovate) for NuGet and GitHub Actions, weekly.

## Publish

- Self-contained, `win-x64`, ReadyToRun. Native `win-arm64` follows v1.0 (B-02); until then Windows 11 on ARM runs the x64 build by emulation (unverified).
- Trimming / NativeAOT: evaluate in Phase 5 (Avalonia supports them with compiled bindings; measure size, startup and breakage before enabling).
- Single-file only if startup time and extraction behaviour are acceptable; measure in Phase 5.
- Artifacts: `Drape-<version>-win-x64-setup.exe` (installer), `Drape-<version>-win-x64.zip` (portable), `SHA256SUMS.txt`.

## Installer and Updates (proposal)

- Velopack: per-user install without admin, delta updates, small launcher. Alternatives rejected for v1.0: MSIX/Store (packaged apps redirect HKCU writes, so the `Run` key autostart would not work; Store policy on global hooks is unverified) and WiX/MSI. Update source: GitHub Releases; auto-apply on startup disabled (ADR-022).
- The winget manifest declares `MinimumOSVersion: 10.0.19045.0` (Windows 10 22H2, [ADR-023](../../MEMORY/DECISIONS.md)). Hyper-V VMs for **both** Windows 10 22H2 and Windows 11 are required for release testing.
- Autostart is set from inside the app (HKCU Run key), not by the installer.
- Uninstall: remove the app and, at the user's choice, the data in `%LOCALAPPDATA%\Drape`. **Uninstall must restore desktop icons** (run `--restore-desktop` before removal).
- Updates: **off by default** ([ADR-017](../../MEMORY/DECISIONS.md)). The user may enable manual or periodic checks; no telemetry; download only from the official release channel; signed packages only.

## Code Signing

- Code-signing certificate (OV/EV) or a managed signing service; keys never in the repository.
- Sign `Drape.exe`, `Drape.Guard.exe` and the installer. Important for reducing SmartScreen warnings and antivirus false positives, since the app uses a global hook.
- Before a certificate exists: release betas with a clear warning and checksums.

## Release Channels

| Channel | Contents | Audience |
|---|---|---|
| `dev` | Build of every merge to `main` (CI artifact, unsigned) | Developers |
| `beta` | `-beta.N` tags | Volunteer testers |
| `stable` | Release tags | Everyone |

## Versioning and Changelog

- SemVer. Before 1.0, config schema changes must still migrate.
- User-visible and operationally significant changes are recorded in [`../../MEMORY/CHANGELOG.md`](../../MEMORY/CHANGELOG.md) (Keep a Changelog style: Added, Changed, Fixed, Removed, Security), updated in every PR that affects users. A release `CHANGELOG.md` for end users is generated from it at Phase 5.

## Release Checklist

- [ ] All phase tasks `DONE` with records; no open blocker/major bug.
- [ ] Manual checklist ([`../TESTING/00-TESTING-STRATEGY.md`](../TESTING/00-TESTING-STRATEGY.md)) complete, recorded per Windows build.
- [ ] Performance tests pass.
- [ ] Migration from the previous version tested with a real (anonymised) config.
- [ ] Uninstall restores desktop icons.
- [ ] Changelog and user docs updated.
- [ ] Tag created, artifacts signed, checksums published.
- [ ] Release notes state known limitations.

## Post-Release Incident Recovery

If a release leaves desktop icons unrestored or Drapes not showing: pull the release (mark pre-release), publish the `--restore-desktop` instructions and the manual steps (right-click desktop → View → Show desktop icons), ship a fix, and write a short post-mortem record in `MEMORY/records/`.
