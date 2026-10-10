# CLAUDE.md — Operating Instructions for AI Agents

How Claude and other AI agents work in this repository. Roles and working agreements are in [`AGENTS.md`](AGENTS.md).

---

## Read This First

**An instruction document that disagrees with the code produces confidently wrong work, and the confidence is the dangerous part.**

This project is in the **planning phase**: there is no code yet. Until code exists, everything below describes the *intended* system, grounded in `docs/` and the ADRs in [`MEMORY/DECISIONS.md`](MEMORY/DECISIONS.md). From the first milestone on, the code is the truth. If you find a claim here that the code contradicts, **the code wins** — and correcting this file is part of the fix, in the same change, with a record.

## What This Is

**Drape** — a desktop organizer that groups desktop icons, shortcuts and files into semi-transparent panels ("Drapes") drawn above the wallpaper.

| | |
|---|---|
| Status | **Planning phase. No code yet.** Solution scaffolding is task `P0-01` |
| Target OS (v1) | **Minimum Windows 10 22H2 (fully supported, owner decision ADR-023)**; Windows 11 is the primary development target. x64 only in v1.0; native ARM64 later (ADR-022). Other platforms are Phase 6 with minimums in ADR-023 |
| Cross-platform | The architecture is cross-platform from day one. macOS, Linux, Android and iOS are **Phase 6, post-v1** (ADR-014) |
| Stack | C# / .NET 10 LTS, **Avalonia UI**, CommunityToolkit.Mvvm, Microsoft.Extensions.Hosting (DI), Serilog, CsWin32 (`Shell.Windows`), System.Text.Json, xUnit + NSubstitute + Shouldly + Avalonia.Headless.XUnit, FlaUI (Windows E2E). Rationale: [`docs/ARCHITECTURE/01-TECH-STACK.md`](docs/ARCHITECTURE/01-TECH-STACK.md) |
| Package versions | Pinned in task `P0-01`. **Verify the latest version on NuGet; never rely on memory** |
| Naming | Product, UI and code all say **Drape** (`Drape`, `DrapeItem`, `DrapeManager`, `DrapeWindow`, `DrapeViewModel`) — ADR-020. The word "fence" appears in no identifier. **Never use Stardock names, icons, UI text or assets** — [`docs/PLAN/05-LEGAL-AND-NAMING.md`](docs/PLAN/05-LEGAL-AND-NAMING.md) |
| Open decisions | None open as of 2026-10-10. Several answers are working decisions the owner may overturn (ADR-021, ADR-022, ADR-023); new questions are `Q-n` entries in [`TASKS/BACKLOG.md`](TASKS/BACKLOG.md) |

## Before You Start

1. **[`docs/README.md`](docs/README.md)** — the map.
2. **[`docs/PLAN/00-PROJECT-OVERVIEW.md`](docs/PLAN/00-PROJECT-OVERVIEW.md)** — what this is and what it deliberately is not.
3. **[`MEMORY/DECISIONS.md`](MEMORY/DECISIONS.md) Part II** — ADRs recording what was decided differently from what was first planned. Read Part II before acting on Part I.
4. **[`TASKS/PROGRESS.md`](TASKS/PROGRESS.md)** — the single status board.
5. **[`TASKS/00-TASK-CONVENTIONS.md`](TASKS/00-TASK-CONVENTIONS.md)** — task cards, branches, commits, the Definition of Done.
6. **[`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](docs/PLATFORM/00-WINDOWS-INTEGRATION.md)** — mandatory before touching any Shell code (the highest-risk area).

## Target Repository Structure

```
Drape.sln
global.json  Directory.Build.props  Directory.Packages.props  .editorconfig
.github/
src/
  Drape.Core/              domain, rules, layout math, interfaces (platform-free)
  Drape.Infrastructure/    ConfigStore, DesktopItemSource, logging
  Drape.Shell.Windows/     Win32/COM: desktop host, icons, hooks, DPI, monitors
  Drape.Shell.Mac/         Cocoa/CoreGraphics (Phase 6)
  Drape.Shell.Linux/       X11/Wayland (Phase 6)
  Drape.UI/                views, view-models, controls, themes (Avalonia)
  Drape.App.Desktop/       desktop composition root, tray, lifecycle
  Drape.App.Android/       Android entry point (Phase 6)
  Drape.App.iOS/           iOS entry point (Phase 6)
  Drape.Guard/             small process that restores desktop icons if the app dies
tests/
  Drape.Core.Tests/  Drape.Infrastructure.Tests/  Drape.UI.Tests/  Drape.E2E/  Drape.TestSupport/
spikes/
docs/  MEMORY/  TASKS/  CLAUDE.md  AGENTS.md  GEMINI.md
```

## The Non-Negotiables

### Architecture rules

1. **`Drape.Core` references no platform** — no Win32, Cocoa, GTK, WPF, Avalonia, and no direct I/O. The outside world is reached through interfaces (`IFileSystem`, `IClock`, ...).
2. **P/Invoke and native APIs live only in the matching `Drape.Shell.*`** (`Shell.Windows` uses CsWin32). Everything is exposed through interfaces defined in `Drape.Core`.
3. **View-models never call native APIs.** Platform calls go through interfaces injected by the App project.
4. **Native handles are wrapped in `SafeHandle` or `IDisposable`.** No leaked handle, PIDL or COM object.
5. **Global hook callbacks stay minimal** (`WH_MOUSE_LL`, `CGEvent tap`): no large allocation, no I/O, no long lock, no stored input. The work is moved to another thread.
6. **Config is written atomically** (temp file, then replace), is schema-versioned, and migrates. See [`docs/ARCHITECTURE/02-DATA-MODEL.md`](docs/ARCHITECTURE/02-DATA-MODEL.md).
7. **One Avalonia window per Drape** (ADR-007).
8. **Desktop-layer features exist only where `IPlatformCapabilities` says so.** Never `#if`, `OperatingSystem.IsWindows()` or other OS checks in UI or Core (ADR-015).
9. **Core returns the neutral `IconBitmap`,** never a UI-framework image type.
10. **Mobile is a shortcut-group management UI.** There is no panel above a wallpaper on Android or iOS.

### Desktop safety — unwaivable

Drape hides the user's desktop icons. If the app dies and the icons stay hidden, the user is stranded. **No one can waive these — not for a deadline, a demo, or with a follow-up ticket.**

- Every code path that hides native icons has a matching restore on **every exit path**: normal exit, exception, Guard, next start, and `--restore-desktop`.
- `--restore-desktop` must always work. Shutdown and unhandled-exception handlers restore icons.
- The default mode is **non-destructive**: Drape stores references to files and never moves, renames or deletes user files without explicit confirmation (ADR-006).
- **Never kill `explorer.exe`** in a script or automated test. Explorer-restart tests are manual, in a VM or a test account.
- Never commit credentials, personal paths or dumps of a user's config.

## The Traps

Each is structural, not careless. Check them before you finish. More in [`docs/PLAN/04-RISK-REGISTER.md`](docs/PLAN/04-RISK-REGISTER.md).

| Trap | Consequence |
|---|---|
| Hard-coding the desktop window hierarchy | Breaks on the next Windows build |
| Treating the icon toggle as a setter | A wrong state read flips the user's desktop |
| An exit path that does not restore icons | The user is stranded |
| `%USERPROFILE%\Desktop` | Wrong on OneDrive-redirected machines |
| Returning a UI-framework image type from Core | Breaks portability; return `IconBitmap` |
| `OperatingSystem.IsWindows()` in UI/Core | Bypasses the capability model; use `IPlatformCapabilities` |
| Touching an Avalonia object off the UI thread | Intermittent crashes; marshal with `Dispatcher.UIThread` |
| A slow hook callback | The OS disables the hook; input is affected system-wide |
| Dropping from an elevated process | Blocked by UIPI; document, never escalate |
| Reading icons that trigger OneDrive hydration | Unwanted downloads |

Found a new trap? Add it here and to [`TASKS/00-TASK-CONVENTIONS.md`](TASKS/00-TASK-CONVENTIONS.md).

## When to Act, and When Not To

**Act on:** an assigned task, a bug found during work, a question about the repository, documentation that disagrees with the code.

**Do not:**

- start a task that is not assigned or next in the queue,
- change architecture without an ADR,
- **amend `docs/` quietly** — that is the deviation protocol, and it needs a record,
- mark something `DONE` without its `MEMORY/records/` entry,
- claim a test passed without **naming it**,
- add features outside the task — ideas go to [`TASKS/BACKLOG.md`](TASKS/BACKLOG.md).

## The Deviation Protocol

When implementation reveals `docs/` is wrong, incomplete or contradictory:

1. **Stop.** Do not quietly implement something different.
2. Write an **ADR** in `MEMORY/DECISIONS.md`: decision, rationale, **alternatives considered**, implications **including the bad ones**.
3. Amend the `docs/` document, referencing the ADR.
4. Note both in the change record.

**A documented deviation is a decision. An undocumented one is a bug nobody has found yet.**

If it needs a decision the owner has not made, it is an Open Question in `TASKS/BACKLOG.md`, not a judgement call.

## Evidence

> **An assertion that a test passed is not evidence. Name the test.**

"Layout tested, all good" with no test named is worse than saying nothing, because it stops anyone looking again.

- **A test generated from the code it tests verifies consistency, never correctness.** A migration test that round-trips the migrator's own output cannot catch a dropped field; use hand-written fixtures per version.
- **A mock proves the client, not the contract.** A fake `IDesktopHost` proves the consumer; only the real implementation on a real Windows build proves the host.
- **A spike result names the build.** "Works on Windows 11" means nothing; "works on 24H2 build 26100.xxxx, 150% DPI, two monitors" is evidence.

## Distinguish "Renders" From "Works"

A window that shows on the developer's machine *renders*. It *works* when it meets the host criteria on every build in the test matrix. State which one you mean. Do not round up.

## Workflow

```
1. read the task card and every document in its Spec refs
2. if "Spec required", write MEMORY/specs/<task-id>-<slug>.md FIRST
3. branch:   feat/P1-04-drape-window            ({type}/{task-id}-{slug})
4. implement (tests first for Core logic)
5. verify:   build · format · test (see Commands); manual checklist for Shell/UI
6. record:   MEMORY/records/ + MEMORY-INDEX.md + CHANGELOG.md + ADR if a decision
7. update:   TASKS/PROGRESS.md — in the SAME commit
8. PR:       feat(core): P1-01 add domain records
```

A green build is not a green release: E2E and the manual checklist ([`docs/TESTING/00-TESTING-STRATEGY.md`](docs/TESTING/00-TESTING-STRATEGY.md)) are separate. Shell and UI changes are tested on Windows 11 and Windows 10 22H2, with at least one non-100% DPI.

## Commands

Valid after `P0-01`.

```bash
dotnet restore
dotnet build Drape.sln -c Debug
dotnet test Drape.sln
dotnet format --verify-no-changes
dotnet run --project src/Drape.App.Desktop
dotnet run --project src/Drape.App.Desktop -- --restore-desktop   # restore desktop icons
```

Before a PR: `dotnet build Drape.sln -c Release -warnaserror`, `dotnet format --verify-no-changes`, `dotnet test Drape.sln`.

## Language and Style

- **All repository content is English**: documentation, code, comments, identifiers, commit messages (ADR-013).
- **Conversation with the owner is in the owner's language** (Indonesian).
- Conventional Commits with the task ID: `feat(core): P1-01 ...`. Scopes: `core`, `shell`, `ui`, `app`, `infra`, `guard`, `docs`, `ci`.
- Coding standards: [`docs/ENGINEERING/00-CODING-STANDARDS.md`](docs/ENGINEERING/00-CODING-STANDARDS.md).
- **Do not say something "works" without running it.** State what was tested and what was not.
- **WIP limit is 3** tasks at once for the whole team; one owner per task.
- Warnings are errors; `dotnet format` must be clean.

## Document Map

| Need | Open |
|---|---|
| Map of all documentation | [`docs/README.md`](docs/README.md) |
| Vision, scope, non-goals | [`docs/PLAN/00-PROJECT-OVERVIEW.md`](docs/PLAN/00-PROJECT-OVERVIEW.md) |
| Requirements (FR/NFR) | [`docs/PLAN/01-PRODUCT-REQUIREMENTS.md`](docs/PLAN/01-PRODUCT-REQUIREMENTS.md) |
| Feature behaviour | [`docs/PLAN/02-FEATURE-SPECIFICATION.md`](docs/PLAN/02-FEATURE-SPECIFICATION.md) |
| Roadmap and phases | [`docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](docs/PLAN/03-IMPLEMENTATION-ROADMAP.md) |
| Risks | [`docs/PLAN/04-RISK-REGISTER.md`](docs/PLAN/04-RISK-REGISTER.md) |
| Legal and naming | [`docs/PLAN/05-LEGAL-AND-NAMING.md`](docs/PLAN/05-LEGAL-AND-NAMING.md) |
| Glossary | [`docs/PLAN/06-GLOSSARY.md`](docs/PLAN/06-GLOSSARY.md) |
| Architecture and lifecycle | [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) |
| Technology choices | [`docs/ARCHITECTURE/01-TECH-STACK.md`](docs/ARCHITECTURE/01-TECH-STACK.md) |
| Data model and config schema | [`docs/ARCHITECTURE/02-DATA-MODEL.md`](docs/ARCHITECTURE/02-DATA-MODEL.md) |
| Win32 integration | [`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](docs/PLATFORM/00-WINDOWS-INTEGRATION.md) |
| Platform support matrix | [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](docs/PLATFORM/01-PLATFORM-SUPPORT.md) |
| UI/UX spec | [`docs/UI-UX/00-UI-UX-SPEC.md`](docs/UI-UX/00-UI-UX-SPEC.md) |
| Coding standards | [`docs/ENGINEERING/00-CODING-STANDARDS.md`](docs/ENGINEERING/00-CODING-STANDARDS.md) |
| Testing strategy | [`docs/TESTING/00-TESTING-STRATEGY.md`](docs/TESTING/00-TESTING-STRATEGY.md) |
| Build, CI, release | [`docs/DEVOPS/00-BUILD-AND-RELEASE.md`](docs/DEVOPS/00-BUILD-AND-RELEASE.md) |
| Security and privacy | [`docs/SECURITY/00-SECURITY-AND-PRIVACY.md`](docs/SECURITY/00-SECURITY-AND-PRIVACY.md) |
| ADRs, records, changelog | [`MEMORY/`](MEMORY/README.md) · [`MEMORY/DECISIONS.md`](MEMORY/DECISIONS.md) |
| Status board | [`TASKS/PROGRESS.md`](TASKS/PROGRESS.md) |
| Task rules and DoD | [`TASKS/00-TASK-CONVENTIONS.md`](TASKS/00-TASK-CONVENTIONS.md) |
| Open questions, backlog | [`TASKS/BACKLOG.md`](TASKS/BACKLOG.md) |
| Roles and handoffs | [`AGENTS.md`](AGENTS.md) |

## If You Are Unsure

Read the code once it exists; before that, read the document and the ADR it names.

If a decision is needed that `docs/` does not contain, ask the owner by writing an **Open Question** in [`TASKS/BACKLOG.md`](TASKS/BACKLOG.md). Meanwhile take the most sensible assumption, **write the assumption in `TASKS/BACKLOG.md`**, and continue. Desktop-safety questions are never resolved by assumption: stop and ask.

If the code and this file disagree, **the code wins, and this file gets fixed** — in the same change, with a record.
