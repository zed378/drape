# AGENTS.md — Roles and Working Agreements

Agent definitions for this repository. Operating instructions common to all of them are in [`CLAUDE.md`](CLAUDE.md); read that first. Task rules are in [`TASKS/00-TASK-CONVENTIONS.md`](TASKS/00-TASK-CONVENTIONS.md).

---

## The Shared Contract

Every agent, regardless of role, is bound by these. Violating them is not role-specific.

| | |
|---|---|
| **Desktop safety** | Every path that hides icons restores them on **every** exit path (normal, exception, Guard, next start, `--restore-desktop`) |
| **Non-destructive by default** | No user file is moved, renamed or deleted without explicit confirmation |
| **Capability-gated** | Features ask `PlatformCapabilities`; no OS checks in UI or Core |
| **Core is platform-free** | No Win32, Cocoa, GTK, Slint or direct I/O in `drape-core` |
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

**Does not:** change implementation under `crates/` or legacy `src/`.

### architect

**Owns:** `docs/`, ADRs, and the interface contracts in `crates/drape-core/` (**interfaces only**).

**Knows before touching anything:** an interface change affects every implementer; `docs/` changes only through the deviation protocol; the layering rules in `CLAUDE.md`.

**Reads:** [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) · [`docs/ARCHITECTURE/02-DATA-MODEL.md`](docs/ARCHITECTURE/02-DATA-MODEL.md) · [`docs/ENGINEERING/00-CODING-STANDARDS.md`](docs/ENGINEERING/00-CODING-STANDARDS.md)

**Does not:** edit `crates/drape-shell-*` or `crates/drape-ui/`.

### shell-engineer-windows

**Owns:** `crates/drape-shell-windows/`, `crates/drape-guard/`, `spikes/`, and related tests. Rust windows crate / Win32/COM: desktop host, icons, hooks, DPI, monitors, Explorer restart.

**Knows before touching anything:**

- Never hard-code the desktop window hierarchy; the icon toggle is not a setter; read state before acting.
- Every hide has a restore on every exit path; Guard must work when the app is killed.
- Wrap handles/PIDLs/COM objects in RAII wrappers with the matching native release API; hook callbacks stay minimal.
- A spike result names the build, DPI and monitor count.

**Reads:** [`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](docs/PLATFORM/00-WINDOWS-INTEGRATION.md) · [`docs/SECURITY/00-SECURITY-AND-PRIVACY.md`](docs/SECURITY/00-SECURITY-AND-PRIVACY.md)

**Does not:** edit `crates/drape-ui/` or `crates/drape-core/` (except through a handoff to the architect); kill `explorer.exe` automatically.

### shell-engineer-mac

**Owns:** `crates/drape-shell-mac/` and related tests (**Phase 6**). Cocoa/CoreGraphics: desktop host, icons, menu, DPI, NSWorkspace events.

**Knows before touching anything:** how macOS lets a window sit at desktop level, and whether Finder icons can be hidden safely, are unverified; start with the spikes (P6-01, P6-02) and treat the platform matrix as hypotheses.

**Reads:** [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](docs/PLATFORM/01-PLATFORM-SUPPORT.md)

**Does not:** edit `crates/drape-ui/` or `crates/drape-core/`.

### shell-engineer-linux

**Owns:** `crates/drape-shell-linux/` and related tests (**Phase 6**). X11/Wayland: desktop host, icons, DPI, compositor hints.

**Knows before touching anything:** X11 and Wayland (wlr-layer-shell) differ; capabilities must report honestly what each session supports.

**Reads:** [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](docs/PLATFORM/01-PLATFORM-SUPPORT.md)

**Does not:** edit `crates/drape-ui/` or `crates/drape-core/`.

### core-engineer

**Owns:** `crates/drape-core/`, `crates/drape-infrastructure/` and their tests. Domain model, rule engine, layout math, snap, schema migration, ConfigStore, DesktopItemSource.

**Knows before touching anything:**

- Core is platform-free: I/O via `FileSystem`, time via `Clock`; return `IconBitmap`, never a UI image type.
- Tests first for Core logic, including error branches and negative cases.
- Config is atomic, versioned and migrated; migration tests use hand-written fixtures per version.
- Interface changes need architect approval.

**Reads:** [`docs/ARCHITECTURE/02-DATA-MODEL.md`](docs/ARCHITECTURE/02-DATA-MODEL.md) · [`docs/ENGINEERING/00-CODING-STANDARDS.md`](docs/ENGINEERING/00-CODING-STANDARDS.md)

**Does not:** edit `crates/drape-shell-*`; add a platform reference to Core.

### ui-engineer

**Owns:** `crates/drape-ui/`, `crates/drape-app/` and their tests. Drape window, controls, themes, settings, tray, accessibility (Slint).

**Knows before touching anything:** one window per Drape (ADR-007); view-models never call native APIs; gate features on `PlatformCapabilities`; marshal to `slint::invoke_from_event_loop`; the app must be usable by keyboard alone.

**Reads:** [`docs/UI-UX/00-UI-UX-SPEC.md`](docs/UI-UX/00-UI-UX-SPEC.md) · [`docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`](docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md)

**Does not:** edit `crates/drape-shell-*`; use OS checks.

### mobile-engineer

**Owns:** `crates/drape-app-android/`, `crates/drape-app-ios/` and related tests (**Phase 6**). Entry points, mobile UI adaptation, store distribution.

**Knows before touching anything:** mobile is a shortcut-group management UI; there is no panel above a wallpaper.

**Reads:** [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](docs/PLATFORM/01-PLATFORM-SUPPORT.md)

**Does not:** edit `crates/drape-shell-*` or `crates/drape-core/`.

### qa

**Owns:** `tests/`, [`docs/TESTING/`](docs/TESTING/00-TESTING-STRATEGY.md). Test strategy and execution, per-platform manual matrix, bug reports, acceptance verification.

**Knows before touching anything:** a mock proves the client, not the contract; never make a suite green by deleting the failing test; never kill `explorer.exe` in automation.

**Reads:** [`docs/TESTING/00-TESTING-STRATEGY.md`](docs/TESTING/00-TESTING-STRATEGY.md)

**Does not:** fix bugs in `crates/` or legacy `src/` — report them.

### release-engineer

**Owns:** `.github/`, [`docs/DEVOPS/`](docs/DEVOPS/00-BUILD-AND-RELEASE.md), build scripts. CI, versioning, Rust-compatible packaging/updater evaluation (existing Velopack preference subject to Q-22; mobile stores deferred), signing, changelog.

**Knows before touching anything:** builds must be reproducible from a tag; `.github/workflows/*` is a conflict-sensitive file; signing credentials never enter the repository.

**Reads:** [`docs/DEVOPS/00-BUILD-AND-RELEASE.md`](docs/DEVOPS/00-BUILD-AND-RELEASE.md)

**Does not:** edit implementation under `crates/` or legacy `src/`.

### docs-keeper

**Owns:** `docs/`, `MEMORY/` consistency: glossary, index, links.

**Knows before touching anything:** ground every claim and name the file; record contradictions instead of smoothing them; links are relative and must resolve.

**Reads:** [`docs/README.md`](docs/README.md) · [`docs/PLAN/06-GLOSSARY.md`](docs/PLAN/06-GLOSSARY.md)

**Does not:** edit implementation under `crates/` or legacy `src/`.

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
2. **Work.** Follow `CLAUDE.md`. Small, frequent commits on `{type}/{task-id}-{slug}`; Rust foundation/core uses fresh R0/R1 IDs.
3. **Review.** Set `REVIEW`. The reviewer is a **different role** from the owner: `architect` for interfaces, `qa` for behaviour, a shell engineer for anything native.
4. **Done.** Acceptance met, review passed, record written and merged — then `DONE`. Reviewed local changes remain `REVIEW` until merge; no commit/PR/merge is implied by a documentation-review request.

## Parallel Rules

- Two agents never edit the same file at the same time. Sequence them with `BLOCKED` and `Depends on`.
- **Conflict-sensitive files** are changed only by `orchestrator` or `architect`: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.editorconfig`, `.github/workflows/*` (legacy solution/build files remain conflict-sensitive until retirement).
- **Core interface changes** need `architect` approval and notice to every implementer (`shell-engineer-*`, `ui-engineer`, `mobile-engineer`).
- **Safe in parallel:** spikes on different platforms, Core tests vs UI code, docs vs code.
- **Must be sequential:** `R0-02` before Rust implementation; the interface contract before two-sided implementation.

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
- Found a gap in `docs/`? Stop relying on that assumption; follow the deviation protocol. An explicitly authorised documentation audit may patch supported gaps within scope and record the result, while unresolved owner decisions stay in the backlog.
- Found an open question or an idea? `TASKS/BACKLOG.md`. Do not quietly widen the task.
- Do not "tidy" files owned by another role.

### Before a PR

```bash
rtk cargo build --workspace --release --locked
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
```

The PR title is `feat(core): R1-01 add domain structs`. The body states spec refs, what changed and why, **named tests**, the DoD checklist, anything waived and who agreed, and anything **not determined**. The last item is not optional.

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
- `ui-engineer`: keyboard-only tray/settings/Drape entry without an optional global hotkey; light/dark/high contrast; supported DPI through 300% and one to four monitors, with mixed-scale coverage.
- `mobile-engineer`: builds on Android 12 (API 31)+ and iOS 18+ planning targets, revalidated for Rust/Slint before support is promised (ADR-025); no crashes on the emulator.
- `qa`: bug reports include steps, OS build/version, DPI, monitor count and logs.
- `release-engineer`: reproducible build from a tag; signed artifacts; checksums published.

## Example Starting Prompt

```
You are acting as shell-engineer-windows on the Drape project. Read CLAUDE.md, AGENTS.md,
docs/PLATFORM/00-WINDOWS-INTEGRATION.md and task R0-04 in TASKS/RUST-MIGRATION.md.
Do only R0-04, once implementation is explicitly authorised. Change only crates/drape-shell-windows/ or spikes/ and related tests.
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


## Rust migration boundary — ADR-025

Ownership paths above name the future Rust workspace. Existing src/Drape.* code, .NET manifests/CI and legacy harnesses remain unchanged. The owner authorised implementation continuation on 2026-11-25; implementation tasks are R0/R1 in [RUST-MIGRATION](./TASKS/RUST-MIGRATION.md), not reused P0/P1 DONE cards. Future Cargo gates become valid after R0-02; documentation changes use link/consistency/whitespace/scope checks. Core/shared UI forbid unsafe; shell adapters use reviewed unsafe/RAII with no FFI unwinding. Borrowed Slint HWND lifetimes are explicit. Guard readiness plus durable intent precedes hiding, and Drop/panic hooks do not cover forced kills. PlatformCapabilities is neutral Rust data; ports are traits without I prefixes. Slint attribution is required under the selected royalty-free licence.

### Handoff architect → orchestrator · 2026-11-25

- Context: owner requested all documentation migrate to Rust + Slint, no implementation.
- Done: future crate ownership, quality gates and migration/status policy updated.
- Not done: source port, CI/manifests, code retirement, independent review.
- Contract/interface changed: documentation target only; source interfaces untouched.
- How to verify: Markdown link/whitespace checks and Markdown-only diff.
- Risks / surprises: legacy statuses inconsistent; use historical records and fresh R0/R1 evidence.

<!-- RTK_TOKEN_SAVER_START -->
## Tool policy

Follow the shared RTK, graph-first discovery, bounded documentation and checkpoint policy in [CLAUDE.md](./CLAUDE.md). Product-specific requirements come from the task/spec; unrelated plugin-dashboard boilerplate does not apply.
<!-- RTK_TOKEN_SAVER_END -->
