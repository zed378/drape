# 00 — Coding Standards

Applies to every project under `src/` and `tests/`. Violations are caught by analyzers, `dotnet format`, architecture tests and review. Rules are numbered so a review can cite them (`CS-n`).

## Language

- **CS-1** Code, identifiers, code comments, branch names and commit messages: English.
- **CS-2** Documentation in `docs/`, `MEMORY/`, `TASKS/`: English.
- **CS-3** User-visible text lives in resources (id-ID, en-US), never hard-coded.
- **CS-4** The conversation with the project owner follows the owner's language (currently Indonesian). It does not affect repository content.

## Structure and Naming

- **CS-5** One public type per file; file name = type name.
- **CS-6** Namespace follows folder: `Drape.Core.Layout`, `Drape.Shell.Windows.Desktop`, … File-scoped namespaces.
- **CS-7** `PascalCase` for types and public members, `camelCase` for parameters and locals, `_camelCase` for private fields.
- **CS-8** Interfaces start with `I`. Platform implementations have descriptive names (`WorkerWDesktopHost`), not `DesktopHostImpl`.
- **CS-9** Async: `Async` suffix; I/O operations take a `CancellationToken`.
- **CS-10** Domain term: the panel is a `Drape` in code, UI and docs ([ADR-020](../../MEMORY/DECISIONS.md)). The entity `Drape` shares its name with the root namespace `Drape`, so import `Drape.Core.Model` explicitly and use an alias where the simple name is ambiguous (`using DrapeEntity = global::Drape.Core.Model.Drape;`). Suppress CA1724 for that one type, with a comment pointing here.

## Language Features and Analyzers

- **CS-11** `Nullable=enable`, `TreatWarningsAsErrors=true`, `LangVersion=latest`, `ImplicitUsings=enable`.
- **CS-12** `record` for immutable data models; `init` for set-once properties.
- **CS-13** No mutable `static` state. No hand-rolled singletons; use DI.
- **CS-14** No `catch (Exception)` that swallows errors without logging. Catch specific types; a general catch must log and take an explicit decision.
- **CS-15** No `async void` except UI event handlers.
- **CS-16** No `.Result` / `.Wait()` on the UI thread.

## Architecture Boundaries

- **CS-17** `Drape.Core` references no UI framework, no Win32/Cocoa/GTK and no direct `System.IO` file-system operations. Use `IFileSystem` and `IClock`.
- **CS-18** P/Invoke and COM only in `Drape.Shell.<Platform>` (and `Drape.Guard`'s icon-restore part). On Windows use CsWin32 (`NativeMethods.txt`). Manual `DllImport` is forbidden unless CsWin32 does not provide the API, with a comment giving the reason and a follow-up task.
- **CS-19** View-models reference no platform or Shell type; platform behaviour arrives through interfaces from Core, injected by the App.
- **CS-20** `Drape.UI` and `Drape.Core` must not reference `System.Windows`, `Microsoft.Win32`, `Windows.*` or any `Drape.Shell.*`. An architecture test (`Drape.Core.Tests/ArchitectureTests`) checks project and assembly references so the rule cannot drift silently.
- **CS-21** Features that depend on a platform are gated with `IPlatformCapabilities`, never with `#if`, `OperatingSystem.IsWindows()` or similar in UI or Core ([ADR-015](../../MEMORY/DECISIONS.md)).
- **CS-22** Core returns neutral data (`IconBitmap`), never UI-framework image types.

## Native Interop

- **CS-23** Every handle, PIDL and COM object is wrapped in `SafeHandle`/`IDisposable`; release with `Marshal.ReleaseComObject` or an equivalent wrapper.
- **CS-24** Turn `HRESULT` and `GetLastError` into typed exceptions at the Shell boundary. Do not leak raw codes upwards.
- **CS-25** Calls that can block for long (context menu, icons) run off the UI thread with a timeout.
- **CS-26** Native callbacks (hooks, WndProc) keep a reference so they are not garbage collected (`GCHandle` or an instance field).
- **CS-27** Every undocumented technique is tagged `// UNDOCUMENTED:` with a reference to the ADR/doc that explains it.
- **CS-28** Global-hook callbacks: no allocation to speak of, no I/O, no long locks; hand work to another thread.

## UI (Avalonia)

- **CS-29** MVVM with CommunityToolkit.Mvvm (`[ObservableProperty]`, `[RelayCommand]`).
- **CS-30** No business logic in code-behind; code-behind only for view-specific things (hit-testing, window plumbing).
- **CS-31** Use resource dictionaries/styles for colours, brushes and sizes; no magic numbers in XAML. Prefer compiled bindings.
- **CS-32** Every interactive control has an `AutomationProperties.Name`.
- **CS-33** Marshal to the UI thread with `Dispatcher.UIThread.InvokeAsync`; avoid synchronous `Invoke` on hot paths.
- **CS-34** One Avalonia window per Drape ([ADR-007](../../MEMORY/DECISIONS.md)).

## Error Handling and Logging

- **CS-35** Fail fast on contract violations (`ArgumentNullException.ThrowIfNull`); be tolerant of environmental failure (missing file, icon failure).
- **CS-36** Structured logging with message templates: `_logger.LogInformation("Drape {DrapeId} moved", id)`. No string interpolation in log messages.
- **CS-37** File paths appear only at Debug level. Never log user file names at Information or above.
- **CS-38** Unhandled exceptions → the global handler restores icons (see [`../ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](../ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md)).

## Performance

- **CS-39** Avoid allocations on hot paths (item rendering, hook callbacks).
- **CS-40** Virtualise item lists above 200 items per Drape.
- **CS-41** Decode icons off the UI thread and cache them; bound the cache.
- **CS-42** Measure before optimising; include the measurement in the PR.

## Tests

- **CS-43** Naming: `MethodUnderTest_Condition_ExpectedResult`.
- **CS-44** One behaviour per test; clear arrange/act/assert.
- **CS-45** No `Thread.Sleep` in tests; use a fake `IClock` and explicit synchronisation.
- **CS-46** Details in [`../TESTING/00-TESTING-STRATEGY.md`](../TESTING/00-TESTING-STRATEGY.md). Name your tests when you claim they passed.

## Git and Pull Requests

- **CS-47** Branch: `{type}/{task-id}-{kebab-description}` (e.g. `feat/P1-04-drape-window`). Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`. One task, one branch, one PR.
- **CS-48** Commit: Conventional Commits with the project scope **and the task ID**: `feat(core): P1-01 add domain records`. Scopes: `core`, `shell`, `ui`, `app`, `infra`, `guard`, `docs`, `ci`. The body explains *why*, not what.
- **CS-49** PRs are small (ideally < 400 diff lines) with: purpose, changes, how tested (named tests), risks, task link, anything waived and who agreed, anything not determined.
- **CS-50** Squash merge; the final message follows Conventional Commits.
- **CS-51** No secrets, personal paths or user config in the repository. CI runs secret scanning.

## Comments and Code Documentation

- **CS-52** Comments explain *why*, not *what*.
- **CS-53** Public Core APIs carry XML doc.
- **CS-54** Behaviour that depends on a Windows version carries a comment with the build number it was tested on.

## Dependencies

- **CS-55** Add packages through `Directory.Packages.props`; check licence and maintenance.
- **CS-56** Do not add a package for something a few lines of code can do.
