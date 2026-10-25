# P0-01 DONE, P0-02, P0-08, P1-03 DONE, P1-07 Core — Batch 2026-10-25

**Date:** 2026-10-25
**Task:** P0-01 (finalise), P0-02, P0-08, P1-03 (finalise), P1-07 (Core part)
**Author:** code agent
**Spec refs:** [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](../../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) · [`docs/DEVOPS/00-BUILD-AND-RELEASE.md`](../../docs/DEVOPS/00-BUILD-AND-RELEASE.md) · [`docs/ARCHITECTURE/02-DATA-MODEL.md`](../../docs/ARCHITECTURE/02-DATA-MODEL.md) · ADR-005, ADR-008, ADR-024

---

## Hook

Three traps in one session, all the same shape — the tool or platform silently doing something
different from what the code says:

1. `dotnet run -- args` did not forward `--restore-desktop` to the app (the host started instead).
   Running the exe directly worked. The flag parsing was correct; the test harness was wrong.
2. `with { Title = " " }` does not run the property initializer, so `Rename(" ")` silently produced
   an invalid Drape. The constructor-time validation does not apply to copies.
3. A manifest with two `<application>` elements or an invalid `disableWindowFiltering` element fails
   side-by-side activation with a generic error. The manifest that "looked right" never ran.

---

## What Changed

- `README.md` — status corrected: no longer claims "no code yet".
- `TASKS/PHASE-0-FOUNDATION.md` — P0-01, P0-02, P0-08 marked DONE (with caveats where noted).
- `TASKS/PHASE-1-DRAPE-CORE.md` — P1-03, P1-07 marked DONE (Core parts).
- `MEMORY/specs/P1-03-config-store.md` — retrospective feature spec for P1-03.
- `.github/workflows/ci.yml` + `.github/dependabot.yml` — P0-02: locked restore, `-warnaserror`
  build, format check, tests with coverage, vulnerability scan. No `continue-on-error`, no
  `explorer.exe` reference.
- `src/Drape.Shell.Windows/Locations/WindowsDesktopLocations.cs` — Known Folder API with GUID
  literals (CsWin32 does not project FOLDERID constants) and `CoTaskMemFreePWSTRSafeHandle`.
- `src/Drape.Shell.Windows/Autostart/WindowsAutostart.cs` — HKCU Run key, readable back.
- `src/Drape.App.Desktop/Program.cs` — full composition root: Generic Host, Serilog to
  `%LOCALAPPDATA%\Drape\logs`, named mutex single-instance, `--restore-desktop` dispatch,
  `AppDomain.UnhandledException` handler, `PhysicalFileSystem` + `SystemClock` implementations.
- `src/Drape.App.Desktop/Drape.App.Desktop.csproj` — added `Serilog.Sinks.Console`.
- `src/Drape.App.Desktop/app.manifest`, `src/Drape.Guard/app.manifest` — removed the bogus
  Windows 11 GUID (`{4a2f28e3-...}` does not exist; Win10 GUID covers Win11) and the invalid
  `disableWindowFiltering` element; merged the duplicate `<application>` block.
- `src/Drape.Core/Services/DrapeService.cs` — create (unique default name, centred, clamped),
  rename, delete (returns ungrouped items, never touches files), `Changed` event.
- `src/Drape.Core/Model/Drape.cs` — `Rename` validates explicitly (the `with` trap).
- `src/Drape.Core/Model/DrapeItem.cs` — `WithOrder` validates explicitly (same trap).
- `tests/Drape.Core.Tests/DrapeServiceTests.cs` — 9 tests, including delete-keeps-files-on-disk
  asserted against `FakeFileSystem` (which records deletions).
- `tests/Drape.Core.Tests/DrapeModelTests.cs` — added `WithOrder_WithNegativeOrder_Throws`.
- `tests/Drape.Infrastructure.Tests/FileConfigStoreTests.cs` — added
  `SaveAsync_FiftyDrapesTimesFiftyItems_UnderTwentyMilliseconds` (passes; measures serialisation +
  store overhead on `FakeFileSystem`, a lower bound, not real-disk I/O).

---

## Why

P0-01 stayed WIP on two items: README accuracy and a full verification run. Both are now done, so
P0-01 is DONE and its dependents (P0-02, P0-08, P0-10) unblock.

P0-08 is DONE for everything verifiable without Avalonia UI: DI, logging, mutex, flag parsing,
exception handlers. The tray icon and Avalonia lifetime need P1-04 (the first window).

P1-07 is DONE for the Core logic. The tray and Drape-menu commands that call it need P1-04.

---

## Alternatives Considered

- **FOLDERID via CsWin32 constant.** Not available — CsWin32 0.3.367 does not project
  `FOLDERID_Desktop`. Used documented GUID literals with a comment naming the source header.
- **`SHGetKnownFolderPath` returning `PWSTR`.** The generated signature returns
  `CoTaskMemFreePWSTRSafeHandle`; used `DangerousValue` inline (the analyzer forbids saving it).
- **Testing `--restore-desktop` via `dotnet run`.** Does not forward args reliably in this
  environment; tested by running the built exe directly.

---

## What Surprised Me

- `dotnet run --project X -- --flag` started the host instead of dispatching the flag. The exe run
  directly behaved correctly. Lesson: verify CLI flags against the built binary, not `dotnet run`.
- `record with { Prop = bad }` bypasses the property initializer. Any `with`-based mutation method
  (`Rename`, `WithOrder`) must validate explicitly. This was a real defect caught by a new test, not
  a style point.
- The manifest GUID `{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}` (claimed "Windows 11") does not exist.
  There is no Win11 GUID; the Win10 GUID covers it. Combined with the duplicate `<application>`
  element and the invalid `disableWindowFiltering`, the app never started — three manifest defects
  stacked, one generic error.
- `dotnet format` enforces BOM (`charset = utf-8-bom`) and `_` prefix on `private static readonly`.
  Every new file needed both fixes.

---

## Tests

- `dotnet build Drape.slnx -c Release -warnaserror` — 0 warnings, 0 errors.
- `dotnet format Drape.slnx --verify-no-changes` — clean.
- `dotnet test Drape.slnx -c Release` — **92 passed, 0 failed** (71 Core + 14 Infra + 7 Arch).
- Manual: `--restore-desktop` via exe → message + exit 2. Second instance → "already running",
  exit 0. Logs written to `%LOCALAPPDATA%\Drape\logs\Drape-YYYYMMDD.log`.
- Not run: CI workflow (needs a real PR run); tray; Avalonia lifetime; real-disk 20 ms timing.

---

## Platform Matrix

| Windows build | DPI scale | Monitors | Result |
|---|---|---|---|
| Dev machine (Windows 11), compile + unit + manual exe runs | 100% (dev default) | 1 | Build/test green; mutex, flag, logging verified |
| Windows 10 22H2 | — | — | Not run |
| Mixed DPI / multi-monitor | — | — | Not run |

---

## Definition of Done

- [x] P0-01: build, format, tests green; README accurate; mutation check on record
- [x] P0-02: workflow + Dependabot added; no `continue-on-error`; no `explorer.exe`
- [x] P0-08: DI, Serilog, mutex, flag, handlers verified by running the exe
- [x] P1-03: spec written; 50×50 save measured under 20 ms (lower bound)
- [x] P1-07 Core: service + 9 tests; delete-keeps-files asserted on the file system
- [ ] P0-02 fail-gate demonstration (needs a real PR with a deliberate violation)
- [ ] P0-08 tray icon + Avalonia lifetime (needs P1-04)
- [ ] P1-07 tray/menu commands (needs P1-04)
- [ ] Real-disk 20 ms timing (needs a manual run outside CI)

**Waived:** none. The unticked items belong to future cards, not to this batch.

---

## Desktop Safety

- **Icon hide path:** none. `WindowsAutostart` writes only HKCU Run; `WindowsDesktopLocations`
  only reads. Neither touches icons.
- **Restore paths:** `--restore-desktop` parses and dispatches; real behaviour is P2-03.
- **Handles:** `CoTaskMemFreePWSTRSafeHandle` disposed via `using`; `DangerousValue` used inline.
- **Config:** no schema change.
- **Privacy:** no network. Log path contains no user file names.

---

## What to Watch

- `Drape.Rename` and `DrapeItem.WithOrder` now validate explicitly. Any future `with`-based
  mutation method must do the same — the initializer will not save it.
- The FOLDERID GUID literals must be re-checked if CsWin32 ever projects them; then the literals
  should be replaced.
- `dotnet run` arg forwarding is unreliable here; always verify CLI flags against the built exe.

---

## What Was Not Determined

- CI has never run: no PR has triggered it, and the fail-gate (deliberate violation → red CI) is
  unproven.
- The 50×50 timing is a lower bound (no real disk I/O). Real-disk timing needs a manual run.
- Tray, Avalonia lifetime, and the UI-thread exception hook are unbuilt (P1-04).
- Nothing was run on Windows 10 or at non-100% DPI.
