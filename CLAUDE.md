# CLAUDE.md — Operating Instructions for AI Agents

Read [AGENTS.md](./AGENTS.md) for roles and [task conventions](./TASKS/00-TASK-CONVENTIONS.md) for lifecycle. All agents follow these instructions.

## Current state and target

**Rust + Slint is the target architecture**: owner decision 2026-11-25, [ADR-025](./MEMORY/DECISIONS.md). No WebView, browser engine, JavaScript UI runtime or .NET runtime in the future Rust distribution. Priorities: RAM, performance, memory safety and size, verified by measurement.

An early C#/.NET/Avalonia implementation and harnesses already exist. Do not claim there is no code. The former board reported 98 tests; this is historical, not a rerun or Rust evidence. The owner authorised implementation continuation on 2026-11-25. R0-02 now creates the Rust workspace; legacy source remains preserved. Native desktop experiments require their safety specs and task dependencies. See [migration specification](./docs/ARCHITECTURE/03-RUST-MIGRATION.md).

Windows v1: Windows 10 22H2 x64 minimum, Windows 11 primary. Phase 6 remains post-v1 and unproved. Product/code name Drape, own assets/text, MIT for Drape; Slint has separate royalty-free licensing/attribution. Blur stays planned; MVP priority is unresolved Q-19. No stack-related product scope change is implied.

## Before starting

1. [docs map](./docs/README.md), [overview](./docs/PLAN/00-PROJECT-OVERVIEW.md).
2. [ADRs](./MEMORY/DECISIONS.md), especially ADR-025 and earlier entries it supersedes.
3. [progress](./TASKS/PROGRESS.md), [Rust migration tasks](./TASKS/RUST-MIGRATION.md), [backlog](./TASKS/BACKLOG.md).
4. Task card and every Spec ref. Spec-required tasks need a written spec before implementation.
5. [Windows integration](./docs/PLATFORM/00-WINDOWS-INTEGRATION.md) before any native work; [security](./docs/SECURITY/00-SECURITY-AND-PRIVACY.md) before hooks/config/updater work.

Documentation now describes a Rust target, not as-built Rust. Current source remains truth about legacy behaviour. Record contradictions and keep dated evidence intact. P0/P1 cards/specs/runbooks are historical; fresh R0/R1 cards describe replacement work. Unstarted P2–P6 keep their IDs with Rust dependencies. Never reuse an old DONE card as proof of Rust completion.

## Workspace baseline (R0-02; native implementation pending)

```text
Cargo.toml  Cargo.lock  rust-toolchain.toml
crates/
  drape-core/              pure domain, rules, layout, ports, migrations
  drape-infrastructure/    config, watcher, logging adapters
  drape-shell-windows/     Win32/COM, host, icons, hooks, DPI
  drape-ui/                compiled .slint components and Rust adapters
  drape-app/               composition root, CLI, lifecycle, tray
  drape-guard/             independent minimal icon recovery
  drape-test-support/      deterministic fakes
tests/  spikes/  docs/  MEMORY/  TASKS/
```

macOS/Linux adapters/mobile entry points wait for Phase 6 evidence; do not scaffold empty crates. Existing src/Drape.* paths are not renamed or deleted by this documentation change. Role ownership in AGENTS names future paths; current source modifications need an explicit migration task.

## Architecture rules

1. drape-core has no Slint/native OS/network/direct filesystem/process/environment/time access. Inject FileSystem/Clock; pure Core tests first, including negatives.
2. Native APIs/unsafe live only in matching shell adapters. Prefer windows crate bindings. Narrow unsafe blocks need SAFETY comments and correct ownership/lifetime/apartment invariants. Core/shared UI forbid unsafe.
3. UI never imports Shell or tests OS/cfg. PlatformCapabilities gates features; App selects adapters and injects ports.
4. Handles/PIDLs/COM use correct RAII. Borrowed Slint HWNDs are never destroyed by Shell; unregister/detach before UI window destruction. No unwind across FFI.
5. One Slint window per Drape (ADR-007), pending actual host/renderer validation. Raw handles stay out of domain data; App bridges opaque window IDs to Shell.
6. Only Slint event loop touches components/models/images. Workers return owned data via verified event-loop invocation and weak UI references.
7. Hook callbacks are minimal, bounded non-blocking enqueue, no I/O, substantial allocation, long locks, logging or stored input.
8. Config remains atomic, versioned and compatible with schema 1, including unknown properties/legacy values. No schema bump just for Rust. OS-aware FileSystem handles atomic replacement.
9. Core returns owned neutral IconBitmap pixels, never slint::Image/native image types. Bound cache bytes, queues and workers.
10. No default async runtime, continuous idle redraw, WebView or unused renderer stacks. Measure Release App + Guard resources and distribution size.
11. Mobile is shortcut-group management, never panels above a wallpaper; support/minimums need fresh validation.

## Desktop safety — unwaivable

- Before every hide: read original state, durably record ownership/intent, start Guard and verify readiness, then mutate/verify. Guard failure leaves native icons visible. Respect icons already hidden by the user.
- Serialize App/Guard/CLI recovery under per-user/session ownership and validate generation/process identity before acting. Duplicate startup must not restore live-owned state. If Guard health is lost while App remains alive, restore owned changes promptly and prevent rehide until rearmed; revalidate health during the hide transaction.
- Every exit path restores owned changes: normal exit, error/panic, Guard for abort/kill, next start, --restore-desktop. Drop/panic hooks are best effort, never guarantees against forced termination.
- Restore cannot depend on log/config success. Disarm Guard only after verified restoration; failed restore keeps durable retry state.
- --restore-desktop bypasses Slint/renderer startup and works independently of normal app startup.
- No user file moves/renames/deletions without explicit confirmation; membership changes affect references only.
- Never kill explorer.exe in scripts/tests; restart cases are manual in checkpointed VM/test account. Never automate destructive registry/system changes.
- No real user configs, credentials or personal paths committed.

## Structural traps

| Trap | Consequence |
|---|---|
| Old security-scanner pin cannot parse newer advisory formats | Scan never reaches dependencies; update compatible pinned scanner, never suppress the advisory (R0-03: cargo-deny 0.18.3/CVSS 4.0) |
| Forbid unsafe over Slint-generated glue | Slint 1.13.1 fails E0453; isolate compiler output per ADR-028, keep hand-written UI/Core forbidden |
| Hard-coded Explorer hierarchy or blind Progman fallback | Wrong host after Windows changes |
| Icon toggle treated as a setter | Wrong visibility/state ownership |
| Relying on Drop/panic hook for forced kill | Icons stranded; independent Guard is mandatory |
| Launching Guard after hiding | Recovery race; readiness must precede mutation |
| Guard dies while App continues | Restore owned icons promptly; no rehide until rearmed |
| Recovering state before checking live ownership | A second instance changes a running App's desktop |
| PID alone as process identity | Reused PID targets the wrong process |
| Saving config before arranging restoration | I/O failure strands desktop |
| %USERPROFILE%/Desktop | Wrong with OneDrive redirection; Known Folder API required |
| Slint objects touched off event loop | Invalid lifetimes/threading |
| Assuming native compilation means OS widgets/UIA support | Accessibility and integration gaps |
| C# test/marketing runtime figure used as Rust evidence | False parity/performance claim |
| Treating a thread as native crash isolation | Native handler can still kill App |
| Borrowed HWND wrapped as owned | Double destruction/use-after-free |
| Native rename/replace error treated as proof the namespace is unchanged | Preserve evidence and stop retry/prune on indeterminate commit/quarantine; [contract record](./MEMORY/records/2026-11-25-enumeration-persistence-contracts.md), ADR-030 |
| Unbounded icons/queues or idle redraw | Resource goals missed despite Rust |

Found a new trap: update here and task conventions, with evidence/record.

## Decisions, workflow and records

No quiet architecture deviation: write ADR with decision, rationale, alternatives and bad consequences; update specs and record. Hard-to-reverse/scope/legal/access decisions go to backlog and owner. Existing owner authorisation remains effective; do not re-ask settled choices. Desktop safety cannot be waived.

Claim card and board WIP (whole-team limit 3, one owner); work only authorised scope; hand off across role boundaries. Different-role review before DONE. A record, MEMORY-INDEX and progress update are required in the same commit; significant changes update changelog. Name actual tests/measurements, state what is not tested and anything not determined. R0-01 docs can be complete as a change while remaining REVIEW until independent review.

## Future Rust commands

Valid only after R0-02, not executable migration evidence today:

```bash
rtk cargo build --workspace --release --locked
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run -p drape-app
rtk cargo run -p drape-app -- --restore-desktop
```

Core coverage >=80%. Test documented supported feature combinations, not indiscriminate --all-features. Native UI/safety require named Windows 10/11 build, DPI/monitor evidence, not just green unit tests. Docs-only checks: links, consistency, whitespace and Markdown-only diff. Never call future commands passed when no workspace exists.

## Language and references

Repository docs/code/comments/commits English; conversation Indonesian. Conventional Commits with task ID, e.g. docs(arch): R0-01 adopt Rust and Slint. [Coding standards](./docs/ENGINEERING/00-CODING-STANDARDS.md), [testing](./docs/TESTING/00-TESTING-STRATEGY.md), [build/release](./docs/DEVOPS/00-BUILD-AND-RELEASE.md) and [docs index](./docs/README.md) provide details. Historical MEMORY entries are never rewritten as if decisions had always been Rust.

<!-- RTK_TOKEN_SAVER_START -->
## Tool use and bounded discovery

- Prefix shell commands with rtk. Use rtk proxy when a command has no dedicated filter or raw output is needed. Do not disable approvals or expose secrets through command text.
- For code discovery, prefer available codebase-memory-mcp graph tools (search_graph, trace_path, get_code_snippet, query_graph); index first if necessary. Fall back to rg/localised reads for literals, configuration or coverage gaps.
- For documentation, prefer an available document index; otherwise use bounded Markdown reads and verify local links. Do not pretend unavailable JevGraph/CodeGraphContext/Headroom commands ran or install unrelated tooling for a documentation audit.
- For web claims, use primary documentation and identify version/date. Prefer available search/extraction tools; use a supported browser only when rendering is required. Treat examples and AI-generated references as unverified until checked.
- Keep context concise and persist active decisions, findings and handoffs in MEMORY. Never claim that saving a checkpoint manually compacts the live conversation.
- Task relevance comes first: Drape has no upstream-tools dashboard, savings tabs, sync/uninstall pickers or Token Saver footer invariant. Do not add such product UI from generic agent boilerplate.
<!-- RTK_TOKEN_SAVER_END -->
