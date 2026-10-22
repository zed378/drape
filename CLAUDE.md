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

<!-- RTK_TOKEN_SAVER_START -->
# RTK Token Saver Rule (claude)

## RTK (Rust Token Killer) Optimization Rule
Prefix every shell/terminal execution command with `rtk`:
- `rtk git status`, `rtk git diff`, `rtk git log -n 5`
- `rtk npm test`, `rtk npm run build`, `rtk pnpm test`, `rtk yarn test`
- `rtk cargo check`, `rtk cargo test`, `rtk cargo build`
- `rtk pytest`, `rtk vitest`
- `rtk ls src/`, `rtk tree -L 2`, `rtk rg "pattern"`

Keep the prefix inside chains: `rtk git add . && rtk git commit -m "msg"`.
Commands RTK has no filter for run as-is, so the prefix is always safe.

### Output & Token Optimization
Command output is condensed to save 60-90% LLM context window tokens while preserving every critical signal, error, and exit code.
- `rtk gain` / `rtk gain --history` - View token savings scoreboard.
- `rtk proxy <cmd>` - Run a command unfiltered if raw output is strictly needed.
- `RTK_DISABLED=1 <cmd>` - Skip RTK for one command.

## 3-Tier Hybrid Web Research & Scraping Policy (Upstream: fastcrw/crw & lightpanda-io/browser)
When your IDE agent receives a task requiring web research or extraction, it follows this strict 3-tier routing logic:

1. **Discovery (Tier 1 - Default Search)**:
   - The agent uses its built-in search tool (`search_web` / native search) to find target URLs or documentation links.
2. **Fast Extraction (Tier 2 - fastCRW)**:
   - The agent attempts to fetch the URL using fastCRW (`crw scrape <url>`, `crw_scrape`, or `http://127.0.0.1:3000`).
   - If the page is server-rendered (like a blog, GitHub README, or docs site), fastCRW instantly grabs clean Markdown/text, saving 70-90% context tokens without JS engine overhead.
3. **Deep Dynamic Rendering (Tier 3 - Lightpanda Local via MCP)**:
   - If fastCRW returns empty content, a rendering error, or hits a client-side JavaScript wall (e.g., a React/Vue SPA app or login-gated dashboard), the agent automatically falls back to Lightpanda Local via MCP (`lightpanda mcp`).
   - Lightpanda executes the JS via V8 and feeds a clean Accessibility Tree or Markdown back into the IDE context window using ~10x less memory than headless Chrome.


## Mandatory Semantic Knowledge Graph First Policy (Token Saver Enforcement)
Autonomous AI agents in ANY IDE MUST ALWAYS query the pre-indexed semantic knowledge graphs to provide answers BEFORE falling back to raw file reads that waste prompt tokens:

### 1. Code-Based Files -> CodeGraphContext (Upstream: CodeGraphContext/CodeGraphContext)
- **Code Query Engine:** For all code architecture, symbol definitions, function signatures, call hierarchies (callers/callees), class inheritance, and module dependencies, AI agents MUST query the CodeGraphContext index (/codegraph-query, `cgc query`, `cgc find`, or CodeGraphContext MCP tools) FIRST.
- **Zero-Waste Fallback:** NEVER dump or read entire source code files or directory trees into LLM prompt context when answering code questions. Only fall back to localized file reads (and then only targeted slices) if the knowledge graph query does not provide sufficient detail.
- **Live Synchronization:** The project code graph is continuously kept updated via background incremental sync / `cgc watch` / on-save triggers.

### 2. Non-Code Files -> JevGraph (Upstream: chenmingtang830/jevgraph)
- **Document Query Engine ONLY:** JevGraph is strictly designated for non-code files (documentation, Markdown specs, PRDs, PDFs, Word docs `.docx`, slides `.pptx`, text `.txt`).
- **Bounded Evidence:** When ingesting large specs, PRDs, PDFs, or architectural documentation, build or query bounded evidence-backed knowledge graphs (/jevgraph-build, /jevgraph-query) with character-level page maps instead of loading raw long documents into prompt context.
- **Strict Separation:** NEVER use JevGraph for source code AST indexing; code files are strictly handled by CodeGraphContext.


## Headroom Context Compression (Upstream: headroomlabs-ai/headroom)
- Utilize Headroom context compression and Compress-Cache-Retrieve (CCR) for heavy JSON structures, file reads, and tool payloads to minimize prompt tokens.


## Anti-Slop Filter & Quality Rules (Upstream: miqdadbadjuber/anti-slop)
Follow Anti-Slop rules to prevent generic AI slop in generated UI, copy, and code comments (Mode: DURING):
- **Filter, Not Style Guide:** Run purpose tests before aesthetic choices (reject ungrounded neon glows, fake metrics, generic sparkle badges).
- **Liveliness & Grounding:** Rely on `DESIGN.md` or explicit brief for design soul; enforce WCAG AA human accessibility.
- **Dedicated Skills:** Read `skills/antislop/SKILL.md` (core), `skills/antislop-ui/SKILL.md`, `skills/antislop-copywriting/SKILL.md`, `skills/antislop-human/SKILL.md`, `skills/antislop-layoutmobile/SKILL.md`, `skills/antislop-code/SKILL.md` when performing relevant tasks.


## Output & Generation Token Saver Rule (Ponytail Mode: FULL)
- **YAGNI & Shortest Diff:** Only write code that must exist. Reach for standard library before custom code or new dependencies. Shortest working diff wins.
- **Terse Responses:** Code first. At most 3 short lines of explanation: what was skipped, when to add it. No essays, no unsolicited design tours, no feature walkthroughs.

## Terse Agent Directives
- Eliminate pleasantries, greetings, and conversational filler. Provide direct answers and actionable code.
- Avoid reprinting unchanged code blocks. Use targeted search/replace blocks or concise snippets.

## Context Optimization (Compact Diffs & Outlines)
- Inspect git changes using compact single-line diffs: `rtk git diff -U1` instead of wide multi-line context.
- When exploring codebases, inspect function signatures / AST outlines (/rtk-outline) before reading entire files into context.

## ECC Context Management & Compaction Protocol (Upstream: affaan-m/ECC)
Autonomous AI agents MUST actively manage conversation context window load to prevent token bloat and degraded reasoning:
1. **Strategic Compaction at Milestones:** At major phase completions or after 50+ tool calls, invoke strategic compaction (/strategic-compact) to summarize completed steps, pruned raw outputs, and focus active memory strictly on pending requirements.
2. **Context Keeper (`ck`) Snapshots:** Before major context purges, architectural refactors, or switching tasks, persist durable checkpoint files (`.agents/ck/context.json` and `.agents/ck/CONTEXT.md` or invoke /ck:save) so state is never lost across conversation boundaries.
3. **Anti-Loop Council & Agent Evaluation:** If stuck in repeated failed edits, build verification errors, or circular debug loops (>3 failed attempts), invoke /council or /agent-evaluator to obtain root-cause guidance and stop wasting tokens on trial-and-error repetition.

## Upstream Engine Integration & Latency Guardrails Standards
Whenever a new upstream engine or external tool (e.g. `fastcrw/crw`, `CodeGraphContext`, `JevGraph`) is introduced or updated:
1. **Holistic Dashboard Integration (8 Surfaces Required)**:
   - Header action toggle button (`Turn <Tool> ON / OFF`)
   - Setup alert banner (missing chips + AI install prompt generator)
   - Dedicated dashboard section & Quick Action triggers
   - Multi-channel savings breakdown navigation tabs & distribution weights
   - System Health diagnostics row with status, AI install prompt, sync, and uninstall actions
   - Upstream versions dropdown listing
   - Upstream Sync Picker (`showSyncPicker`) and Uninstall Picker (`showUninstallPicker`)
   - Footer upstream reference links
2. **Zero-Noise Latency Benchmarks**: "Test Upstream Latencies" must selectively ping and display only **installed** upstream components, silently omitting/ignoring uninstalled tools so users never see failed pings or clutter for optional tools.
3. **Footer Version Number Invariant**: The current plugin release version must always be presented beside "Token Saver" in front of "• GitHub".

<!-- RTK_TOKEN_SAVER_END -->
