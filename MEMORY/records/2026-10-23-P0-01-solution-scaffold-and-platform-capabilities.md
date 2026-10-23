# P0-01 — Solution Scaffold, Layering Test and Platform Capabilities

**Date:** 2026-10-23
**Task:** P0-01 (Scaffold repo and Avalonia solution) — partial; see *Definition of Done*
**Author:** code agent
**Spec refs:** [`docs/ARCHITECTURE/01-TECH-STACK.md`](../../docs/ARCHITECTURE/01-TECH-STACK.md) · [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](../../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) · [`docs/ENGINEERING/00-CODING-STANDARDS.md`](../../docs/ENGINEERING/00-CODING-STANDARDS.md) · ADR-012, ADR-015, ADR-024

---

## Hook

`Drape.UI` depends on Avalonia by definition, so the first version of the architecture test — "no
platform-neutral project may reference a UI framework" — failed on the project whose entire job is
to be a UI. The rule had to be split into two: *no shell references* (Core, UI, Infrastructure) and
*no UI/Windows SDK packages* (Core, Infrastructure only). A single blanket rule would have been
either useless or wrong.

---

## What Changed

- `tests/Drape.Arch.Tests/` — new `LayeringTests` reading the real `.csproj` files; project moved to
  `net10.0-windows10.0.19041.0` because it loads the Windows shell project; added to `Drape.slnx`
  (it was missing) and to central package management.
- `src/Drape.Shell.Windows/Display/MonitorEnumerator.cs` — `EnumDisplayMonitors` + `GetMonitorInfo`
  + `GetDpiForMonitor` returning `MonitorInfo` in physical pixels.
- `src/Drape.Shell.Windows/Dialogs/ShellFileDialog.cs` — folder and multi-file pickers on
  `IFileOpenDialog`, each run on a short-lived STA thread.
- `src/Drape.Shell.Windows/WindowsPlatformCapabilities.cs` — the `IPlatformCapabilities`
  implementation that wires the two above together.
- `tests/Drape.TestSupport/FakePlatformCapabilities.cs` — the fake required by the card.
- `tests/Drape.Core.Tests/PlatformCapabilitiesContractTests.cs` — contract tests against the fake.
- `MEMORY/DECISIONS.md` — ADR-024 (app → shell referencing decision).
- `.editorconfig` — added a `const` → PascalCase naming rule ahead of the private-field rule. The
  existing rule treated `private const` as a private field and demanded `_camelCase` on constants.
- `Directory.Packages.props` — `Microsoft.Windows.CsWin32` 0.3.106 → 0.3.367 (checked with
  `dotnet list package --outdated`).

---

## Why

The card requires an architecture test that fails on a forbidden reference, and an
`IPlatformCapabilities` skeleton with a fake. The shell implementation was written now rather than
in Phase 2 because the interface was being defined anyway and the CsWin32 bindings for monitors and
`IFileOpenDialog` already existed in `NativeMethods.txt`; leaving them unused would have meant
discovering the interop problems later, in a spike that is supposed to be about something else.

The app → shell decision (ADR-024) is option (c), one thin head per OS, because it keeps the
"which OS am I on" question inside the composition root, which is the only place CS-21 permits it.

---

## Alternatives Considered

- **Architecture test by reflection over loaded assemblies.** Rejected: it asserts that the
  assemblies load, which is the abuse case named in the card. Reading the project files catches a
  reference the moment it is added, in the same build that compiles it.
- **One blanket package rule for Core, UI and Infrastructure.** Rejected: fails on `Drape.UI`, see
  *Hook*.
- **Calling `IFileOpenDialog` directly on the Avalonia UI thread.** Rejected: the common item
  dialog needs an STA apartment and a message pump; Avalonia's UI thread is not guaranteed to be
  STA on every platform path, and a blocking modal call there would freeze the app.
- **Returning monitor rectangles in logical pixels.** Rejected: `docs/PLATFORM/00-WINDOWS-INTEGRATION.md`
  §8 says Shell works in physical pixels and converts at the `IDisplayMonitor` boundary.

---

## What Surprised Me

- `PInvoke.EnumDisplayMonitors` has two overloads — `RECT?` and `RECT*` — and passing `null` for the
  clip rectangle is ambiguous between them. The pointer overload has to be selected explicitly.
- CsWin32 generates `MONITORINFOEXW` with an embedded `MONITORINFO` plus `szDevice`, and
  `GetMonitorInfo` dispatches on `cbSize`. Passing the embedded struct with the `MONITORINFOEXW`
  size is what fills in the device name; using `MONITORINFO` silently loses it and every monitor
  gets the same fallback id.
- `COMDLG_FILTERSPEC` holds `PCWSTR`, not `string`. The strings must be pinned with `fixed` for the
  duration of `SetFileTypes`; letting the marshaller handle it compiles but is not what the
  signature offers.
- `TreatWarningsAsErrors` plus `CS1591` means every public test method needs XML doc. That is
  correct for the library but noisy for tests; it was not waived here, only written out.
- `.editorconfig` applied the private-field naming rule to `private const`, demanding `_camelCase`
  on constants. Naming rules are evaluated in declaration order, so a `const`-specific rule has to
  come *before* the private-field rule to win.
- CsWin32 0.3.367 changed the shape of two helpers: `CoCreateInstance<T>` and
  `SHCreateItemFromParsingName<T>` are now generic and no longer take a `riid` argument. The code
  written against 0.3.106 stopped compiling on the upgrade — a reminder that the generated surface
  is a dependency surface.
- `.editorconfig` requires `charset = utf-8-bom`. Files written by tooling without the BOM fail
  `dotnet format --verify-no-changes` with a `CHARSET` error that does not mention the BOM.

---

## Tests

- Unit: `PlatformCapabilitiesContractTests` — 7 tests, all passing.
  - `GetPrimaryMonitor_ReturnsMonitorFlaggedPrimary`
  - `GetMonitors_ReturnsEveryConfiguredMonitor`
  - `GetPrimaryMonitor_WithNoPrimaryFlagged_FallsBackToFirstMonitor`
  - `Constructor_WithEmptyMonitorList_Throws`
  - `PickFolderAsync_CancelledToken_Throws`
  - `PickFilesAsync_RecordsInitialPathAndReturnsResult`
  - `Defaults_DescribeAPlainPlatform`
- Architecture: `LayeringTests` — 7 tests, all passing.
  - `Project_ReferencesNoPlatformShellProject` (Core, UI, Infrastructure)
  - `Project_DeclaresNoUiOrWindowsPackageReference` (Core, Infrastructure)
  - `Core_ReferencesNoUiFrameworkOrWindowsApi`
  - `AppDesktop_IsTheOnlyProjectReferencingTheWindowsShell`
- Integration: none. The monitor enumeration and the dialogs were **not** exercised against a live
  desktop; they compile and are typed against the generated bindings only.
- **Desktop-safety test:** not applicable — nothing in this change hides icons, hosts a window or
  installs a hook.
- **Mutation check:** added `<PackageReference Include="Avalonia" />` to `Drape.Core.csproj` and
  watched `Project_DeclaresNoUiOrWindowsPackageReference(projectName: "Drape.Core")` fail with the
  expected message; then removed it and watched the suite go green again.

---

## Platform Matrix

| Windows build | DPI scale | Monitors | Result |
|---|---|---|---|
| Build machine (Windows 11), compile only | — | — | Builds, 0 warnings; `dotnet format --verify-no-changes` clean |
| — | — | — | Not run against a live desktop |

---

## Definition of Done

- [x] `dotnet build Drape.slnx -c Release` passes with no warnings
- [x] `dotnet format Drape.slnx --verify-no-changes` is clean
- [x] An architecture test fails if Core, UI or Infrastructure references a `Drape.Shell.*` project
- [x] An architecture test fails if Core or Infrastructure references Avalonia or the Windows SDK
- [x] Mutation check performed and the failure observed
- [x] The App-to-Shell referencing decision is recorded (ADR-024)
- [x] `IPlatformCapabilities` skeleton exists in Core with a fake in `tests/Drape.TestSupport`
- [x] `dotnet test Drape.slnx` green — 14 tests pass; the three projects without tests report
      "no test is available", which is expected until Phase 1 adds them
- [x] Package versions re-checked on NuGet with `dotnet list Drape.slnx package --outdated
      --include-prerelease`; the only non-prerelease update was CsWin32, applied
- [ ] Card DoD: "tested on a clean machine" — not done, this ran on the development machine
- [ ] Card DoD: root `README.md` accuracy re-check — not done in this session

**Waived:** none. The unticked items are not waived. P0-01 stays `WIP` for the two items above,
which belong to a reviewer or a second session rather than to more code.

---

## Desktop Safety

- **Icon hide path:** none. `WindowsPlatformCapabilities.CanHideDesktopIcons` returns `true` but no
  code behind it exists yet; it is a statement that the technique is available, not that it is
  implemented.
- **Restore paths exercised:** none — no hide exists to restore.
- **Handles:** `HMONITOR` values are used transiently inside the enumeration callback and never
  stored. COM objects from the dialogs are released with `Marshal.ReleaseComObject` in `finally`.
  No `SafeHandle` wrapper exists yet because no handle outlives a single call.
- **Hook callback:** none installed.
- **Capability gating:** `SupportsDesktopLevelWindow` deliberately returns `false` because ADR-007 is
  still `Proposed`; reporting `true` would let the UI offer a panel that has nowhere to render.
- **Config:** no schema change.
- **Privacy:** no network call, no data collected. The dialogs return paths to the caller; nothing
  logs them (CS-37).

---

## What to Watch

- `SupportsDesktopLevelWindow` must be flipped only when spike P0-03 accepts ADR-007, and the flip
  must come with the host implementation in the same change.
- `CanHideDesktopIcons` currently claims `true` before any icon code exists. If P0-04 changes the
  technique or finds a build where it fails, this must be revisited.
- The `LayeringTests` project-file reader locates the repository root by walking up for
  `Drape.slnx`. If the solution file is renamed, every architecture test fails with a
  `FileNotFoundException` rather than a useful message.
- `ShellFileDialog` opens a fresh STA thread per dialog. Under rapid repeated invocation that is a
  thread churn risk; it has not been measured.

---

## What Was Not Determined

- Monitor enumeration and DPI values were **not** compared against what Windows reports on a real
  multi-monitor mixed-DPI setup. The code is typed correctly but unproven.
- Neither picker was opened. The `HRESULT_FROM_WIN32(ERROR_CANCELLED)` constant
  (`0x800704C7`) is written from the documented value and has not been observed in a debugger.
- Whether Avalonia's UI thread is STA in practice on Windows was not checked; the code does not
  depend on it, but the assumption behind the extra thread is therefore untested.
- The build was not run on a clean machine or in CI; it ran on the development machine, so a
  missing prerequisite there would not have shown up.
- `README.md` was not re-read against the current state of the repository.
- `Drape.Arch.Tests` is in `Drape.slnx` but was not in the original solution file, so anything
  else that enumerates the solution may need the same addition.
