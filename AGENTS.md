# AGENTS.md — Roles and Working Agreements

Agent definitions for this repository. Operating instructions common to all of them are in [`CLAUDE.md`](CLAUDE.md); read that first. Task rules are in [`TASKS/00-TASK-CONVENTIONS.md`](TASKS/00-TASK-CONVENTIONS.md).

---

## The Shared Contract

Every agent, regardless of role, is bound by these. Violating them is not role-specific.

| | |
|---|---|
| **Desktop safety** | Every path that hides icons restores them on **every** exit path (normal, exception, Guard, next start, `--restore-desktop`) |
| **Non-destructive by default** | No user file is moved, renamed or deleted without explicit confirmation |
| **Capability-gated** | Features ask `IPlatformCapabilities`; no OS checks in UI or Core |
| **Core is platform-free** | No Win32, Cocoa, GTK, Avalonia or direct I/O in `Drape.Core` |
| **Name the test** | An assertion that a test passed is not evidence |
| **Deviations get an ADR** | A documented deviation is a decision; an undocumented one is a bug nobody has found yet |
| **Nothing is `DONE` without its record** | `MEMORY/records/` entry, `MEMORY-INDEX.md` line, `TASKS/PROGRESS.md` updated in the same commit |
| **WIP limit is 3** | For the whole team, at the same time |
| **One owner per task** | A `WIP` task is not taken by another agent |

**A desktop-safety finding is not waivable by anyone.** Everything else is negotiable with a recorded decision naming who agreed.

---

## Roles

Each agent edits only the directories its role owns. Crossing a boundary goes through a handoff.

### orchestrator

**Owns:** `TASKS/`, `MEMORY/`, `AGENTS.md`. Splits phases into tasks, orders them, assigns owners, resolves conflicts, keeps the board honest.

**Knows before touching anything:** the board must never list shipped work as TODO; a task needs Spec refs; a decision `docs/` does not contain is an Open Question, not a task.

**Reads:** [`TASKS/README.md`](TASKS/README.md) · [`TASKS/PROGRESS.md`](TASKS/PROGRESS.md) · [`docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](docs/PLAN/03-IMPLEMENTATION-ROADMAP.md)

**Does not:** change `src/`.

### architect

**Owns:** `docs/`, ADRs, and the interface contracts in `src/Drape.Core/` (**interfaces only**).

**Knows before touching anything:** an interface change affects every implementer; `docs/` changes only through the deviation protocol; the layering rules in `CLAUDE.md`.

**Reads:** [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) · [`docs/ARCHITECTURE/02-DATA-MODEL.md`](docs/ARCHITECTURE/02-DATA-MODEL.md) · [`docs/ENGINEERING/00-CODING-STANDARDS.md`](docs/ENGINEERING/00-CODING-STANDARDS.md)

**Does not:** edit `src/Drape.Shell.*` or `src/Drape.UI/`.

### shell-engineer-windows

**Owns:** `src/Drape.Shell.Windows/`, `src/Drape.Guard/`, `spikes/`, and related tests. Win32/COM: desktop host, icons, hooks, DPI, monitors, Explorer restart.

**Knows before touching anything:**

- Never hard-code the desktop window hierarchy; the icon toggle is not a setter; read state before acting.
- Every hide has a restore on every exit path; Guard must work when the app is killed.
- Wrap handles/PIDLs/COM objects in `SafeHandle`/`IDisposable`; hook callbacks stay minimal.
- A spike result names the build, DPI and monitor count.

**Reads:** [`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](docs/PLATFORM/00-WINDOWS-INTEGRATION.md) · [`docs/SECURITY/00-SECURITY-AND-PRIVACY.md`](docs/SECURITY/00-SECURITY-AND-PRIVACY.md)

**Does not:** edit `src/Drape.UI/` or `src/Drape.Core/` (except through a handoff to the architect); kill `explorer.exe` automatically.

### shell-engineer-mac

**Owns:** `src/Drape.Shell.Mac/` and related tests (**Phase 6**). Cocoa/CoreGraphics: desktop host, icons, menu, DPI, NSWorkspace events.

**Knows before touching anything:** how macOS lets a window sit at desktop level, and whether Finder icons can be hidden safely, are unverified; start with the spikes (P6-01, P6-02) and treat the platform matrix as hypotheses.

**Reads:** [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](docs/PLATFORM/01-PLATFORM-SUPPORT.md)

**Does not:** edit `src/Drape.UI/` or `src/Drape.Core/`.

### shell-engineer-linux

**Owns:** `src/Drape.Shell.Linux/` and related tests (**Phase 6**). X11/Wayland: desktop host, icons, DPI, compositor hints.

**Knows before touching anything:** X11 and Wayland (wlr-layer-shell) differ; capabilities must report honestly what each session supports.

**Reads:** [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](docs/PLATFORM/01-PLATFORM-SUPPORT.md)

**Does not:** edit `src/Drape.UI/` or `src/Drape.Core/`.

### core-engineer

**Owns:** `src/Drape.Core/`, `src/Drape.Infrastructure/` and their tests. Domain model, rule engine, layout math, snap, schema migration, ConfigStore, DesktopItemSource.

**Knows before touching anything:**

- Core is platform-free: I/O via `IFileSystem`, time via `IClock`; return `IconBitmap`, never a UI image type.
- Tests first for Core logic, including error branches and negative cases.
- Config is atomic, versioned and migrated; migration tests use hand-written fixtures per version.
- Interface changes need architect approval.

**Reads:** [`docs/ARCHITECTURE/02-DATA-MODEL.md`](docs/ARCHITECTURE/02-DATA-MODEL.md) · [`docs/ENGINEERING/00-CODING-STANDARDS.md`](docs/ENGINEERING/00-CODING-STANDARDS.md)

**Does not:** edit `src/Drape.Shell.*`; add a platform reference to Core.

### ui-engineer

**Owns:** `src/Drape.UI/`, `src/Drape.App.Desktop/` and their tests. Drape window, controls, themes, settings, tray, accessibility (Avalonia).

**Knows before touching anything:** one window per Drape (ADR-007); view-models never call native APIs; gate features on `IPlatformCapabilities`; marshal to `Dispatcher.UIThread`; the app must be usable by keyboard alone.

**Reads:** [`docs/UI-UX/00-UI-UX-SPEC.md`](docs/UI-UX/00-UI-UX-SPEC.md) · [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md)

**Does not:** edit `src/Drape.Shell.*`; use OS checks.

### mobile-engineer

**Owns:** `src/Drape.App.Android/`, `src/Drape.App.iOS/` and related tests (**Phase 6**). Entry points, mobile UI adaptation, store distribution.

**Knows before touching anything:** mobile is a shortcut-group management UI; there is no panel above a wallpaper.

**Reads:** [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](docs/PLATFORM/01-PLATFORM-SUPPORT.md)

**Does not:** edit `src/Drape.Shell.*` or `src/Drape.Core/`.

### qa

**Owns:** `tests/`, [`docs/TESTING/`](docs/TESTING/00-TESTING-STRATEGY.md). Test strategy and execution, per-platform manual matrix, bug reports, acceptance verification.

**Knows before touching anything:** a mock proves the client, not the contract; never make a suite green by deleting the failing test; never kill `explorer.exe` in automation.

**Reads:** [`docs/TESTING/00-TESTING-STRATEGY.md`](docs/TESTING/00-TESTING-STRATEGY.md)

**Does not:** fix bugs in `src/` — report them.

### release-engineer

**Owns:** `.github/`, [`docs/DEVOPS/`](docs/DEVOPS/00-BUILD-AND-RELEASE.md), build scripts. CI, versioning, installer (Velopack on desktop, stores on mobile), signing, changelog.

**Knows before touching anything:** builds must be reproducible from a tag; `.github/workflows/*` is a conflict-sensitive file; signing credentials never enter the repository.

**Reads:** [`docs/DEVOPS/00-BUILD-AND-RELEASE.md`](docs/DEVOPS/00-BUILD-AND-RELEASE.md)

**Does not:** edit `src/`.

### docs-keeper

**Owns:** `docs/`, `MEMORY/` consistency: glossary, index, links.

**Knows before touching anything:** ground every claim and name the file; record contradictions instead of smoothing them; links are relative and must resolve.

**Reads:** [`docs/README.md`](docs/README.md) · [`docs/PLAN/06-GLOSSARY.md`](docs/PLAN/06-GLOSSARY.md)

**Does not:** edit `src/`.

---

## Mapping to Claude Code Subagents

- **`Explore`** — read-only code search and understanding.
- **`Plan`** — implementation planning and design trade-offs.
- **`general-purpose`** — implementation and tests; **state the role first in the prompt**.
- Custom subagent definitions may live in `.claude/agents/` (task `P0-10`).

---

## Task Lifecycle

Statuses: `TODO` → `WIP` → `REVIEW` → `DONE`. Side statuses: `BLOCKED`, `DROPPED` (with a reason).

1. **Claim.** Set `WIP` on the card and in `TASKS/PROGRESS.md` with owner and date. At most **3** `WIP` at once.
2. **Work.** Follow `CLAUDE.md`. Small, frequent commits on `{type}/{task-id}-{slug}`.
3. **Review.** Set `REVIEW`. The reviewer is a **different role** from the owner: `architect` for interfaces, `qa` for behaviour, a shell engineer for anything native.
4. **Done.** Acceptance met, review passed, record written — then `DONE`.

## Parallel Rules

- Two agents never edit the same file at the same time. Sequence them with `BLOCKED` and `Depends on`.
- **Conflict-sensitive files** are changed only by `orchestrator` or `architect`: `Drape.sln`, `Directory.Packages.props`, `Directory.Build.props`, `global.json`, `.editorconfig`, `.github/workflows/*`.
- **Core interface changes** need `architect` approval and notice to every implementer (`shell-engineer-*`, `ui-engineer`, `mobile-engineer`).
- **Safe in parallel:** spikes on different platforms, Core tests vs UI code, docs vs code.
- **Must be sequential:** `P0-01` before everything; the interface contract before two-sided implementation.

## Handoff Template

Write at the bottom of the relevant task card:

```
### Handoff <from-role> → <to-role> · <date>
- Context: <one sentence>
- Done: <list>
- Not done: <list>
- Contract/interface changed: <name + location>
- How to verify: <command + expected result>
- Risks / surprises: <list>
```

## Completion Report Template

```
Task: <id> <title>
Result: <1–2 sentences>
Evidence: <command run> → <short result>; named tests: <test names>
Not tested: <what was not tested and why>
Docs impact: <files updated>
New decision: <ADR id in MEMORY/DECISIONS.md, if any>
```

---

## Working Agreements

### Before starting

Read the task card and **every document in its Spec refs**. If it is `Spec required`, write the spec first. Read `MEMORY/` for context; you retain nothing between sessions.

### While working

- Found a trap? Add it to the traps table in `CLAUDE.md`.
- Found a gap in `docs/`? **Stop.** That is the deviation protocol.
- Found an open question or an idea? `TASKS/BACKLOG.md`. Do not quietly widen the task.
- Do not "tidy" files owned by another role.

### Before a PR

```bash
dotnet build Drape.sln -c Release -warnaserror
dotnet format --verify-no-changes
dotnet test Drape.sln
```

The PR title is `feat(core): P1-01 add domain records`. The body states spec refs, what changed and why, **named tests**, the DoD checklist, anything waived and who agreed, and anything **not determined**. The last item is not optional.

### Handover

**`MEMORY/` is the only continuity mechanism.** Writing to it is part of the task, not a summary appended afterwards. A record written a week later is a reconstruction, and reconstructions omit the parts that were confusing at the time.

---

## Concurrency Limits

- At most **3** tasks in `WIP` for the whole team.
- The orchestrator runs at most **4 subagents at the same time** (owner instruction, 2026-10-10; ADR-022). Give each a non-overlapping set of files, say so in its prompt, and delete or merge superseded files only after all of them have finished.

## Escalation

Stop and ask the owner (briefly, with options and a recommendation) when:

| Situation | Action |
|---|---|
| A decision that is hard to reverse: changing the stack, changing the config schema without a migration, changing the desktop host strategy after Phase 1 | Open Question in `BACKLOG.md`, raise with the owner |
| A destructive operation on user files, the registry or system processes (`explorer.exe`) | Stop; never automate |
| A scope change: a new feature outside the task, or removing a feature from a phase | Open Question; the owner decides |
| A legal concern: wanting to use assets, code or text from another product | Stop; see `docs/PLAN/05-LEGAL-AND-NAMING.md` |
| Two spikes reach conflicting conclusions that data cannot settle | Record both, raise with the owner |
| Missing access or credentials (code signing, store accounts) | Stop; do not work around |

Otherwise take the most sensible assumption, write it in `TASKS/BACKLOG.md`, and continue.

## Quality Gates per Role

- `shell-engineer-windows`: no leaked handles; desktop icons recover after the process is killed; works after an Explorer restart (manual, VM).
- `shell-engineer-mac`: no leaked handles; icons recover; works after a Dock/Finder restart.
- `shell-engineer-linux`: window hints applied; works on X11 and Wayland (wlr-layer-shell).
- `core-engineer`: new logic has tests; Core stays platform-free (checked by an analyzer or architecture test).
- `ui-engineer`: usable by keyboard alone; light, dark and high-contrast themes; DPI 100/150/200% on desktop.
- `mobile-engineer`: builds on Android 12 (API 31)+ and iOS 18+ (Avalonia support tiers, ADR-022); no crashes on the emulator.
- `qa`: bug reports include steps, OS build/version, DPI, monitor count and logs.
- `release-engineer`: reproducible build from a tag; signed artifacts; checksums published.

## Example Starting Prompt

```
You are acting as shell-engineer-windows on the Drape project. Read CLAUDE.md, AGENTS.md,
docs/PLATFORM/00-WINDOWS-INTEGRATION.md and task P0-03 in TASKS/PHASE-0-FOUNDATION.md.
Do only P0-03. Change only src/Drape.Shell.Windows/ or spikes/ and related tests.
Name the build, DPI and monitor setup of every result. End with a completion report
using the template in AGENTS.md.
```

## Anti-Patterns

- Taking several tasks at once without changing their status.
- "While I'm here" tidying of files owned by another role.
- Marking a task done without running build and tests, or without naming a test.
- Changing a Core interface to ease one implementation, without architect review.
- Keeping knowledge only in the conversation. Decisions and lessons go to `MEMORY/`.
- Rounding "renders" up to "works".

<!-- RTK_TOKEN_SAVER_START -->
# RTK Token Saver Rule (agents)

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
