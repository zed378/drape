# Progress

Single status board. Updated 2026-11-25 for [ADR-025](../MEMORY/DECISIONS.md). Target: Rust + Slint. The owner authorised Rust implementation continuation on 2026-11-25. R0-02 establishes the workspace; legacy source/CI remain preserved.

## Current state

The owner authorized commit, merge to main and push of the reviewed accumulated baseline on 2026-11-25. See the [integration record](../MEMORY/records/2026-11-25-main-integration.md). REVIEW rows below retain unfinished acceptance; earlier local/unmerged wording describes the validation checkpoint rather than a prohibition on this authorized integration.

Early C#/.NET/Avalonia code and spike harnesses remain. Previous board reported 98 tests (71 Core, 14 Infra, 7 architecture, 6 UI); not rerun here and not Rust evidence. Legacy P0/P1 status tables sometimes disagree with that board; they are frozen as historical snapshots, not active assignments. The Rust baseline starts with fresh R0/R1 IDs in [RUST-MIGRATION](./RUST-MIGRATION.md).

Legacy P1-04 partial UI and P0-03–P0-07 VM-pending harnesses are superseded as active work by Rust replacement tasks. Their source, specs, runbooks and dated records are preserved. No legacy task ID is reused or relabelled as Rust DONE. Old VM ZIPs require .NET and cannot validate Slint.

## In progress (WIP limit 3)

| ID | Work | Owner | Started |
|---|---|---|---|
| R0-05 | Recovery protocol spec drafting | orchestrator | 2026-11-26 |

## Review

| ID | Work | Owner | Status |
|---|---|---|---|
| R0-06 | Bounded STA icon worker and raw-DIB handoff (partial) | shell-enumeration | 23 enumeration/icon/cache/path tests; 50 shell tests, 110 workspace tests, locked build/format/Clippy and 7 boundary tests pass; pixel semantics, provider/UI integration and VM matrix pending |
| R0-08 | Deferred callback-data read and panic-safe forwarding (partial) | hook-consumer | 27 hook tests; 50 shell tests, 110 workspace tests, locked build/format/Clippy and 7 boundary tests pass; native WH_MOUSE_LL integration and VM matrix pending |
| R1-01 | Domain models, neutral ports, snapshot, filesystem fake, ConfigStore contract and fake (partial) | core-engineer | [ConfigStore record](../MEMORY/records/2026-11-26-R1-01-config-store-contract.md): architect review ACCEPT, 126 workspace tests, 7 boundary tests; native port contracts and R1-02 dependency decision pending |
| R1-02 | Lossless schema-1 persistence service (partial) | core-engineer | [Record](../MEMORY/records/2026-11-26-R1-02-persistence-service.md): 181 workspace tests, 8 boundary tests, two review rounds; native adapter, wiring and Windows evidence pending |
| R0-04 | Slint native-handle diagnostic (partial spike) | shell-engineer-windows | Isolated locked Release/format/Clippy and 2 CLI tests pass; architect/QA static review passed; full native hosting and Windows matrix pending |
| R0-03 | Rust CI and quality gates | release-engineer | QA/static review and local gates passed; integrated on main; hosted Actions pending |
| R1-03 | Pure layout, snap and monitor recovery | core-engineer | 14 new tests, no-drift mutation and integrated gates pass; persistence/native integration and full prerequisites pending |
| R0-01 | Rust + Slint documentation baseline | architect | Specialist documentation review complete; merge pending |
| R0-02 | Rust workspace, contracts and dependency/licence baseline | architect | Local release/format/Clippy, 8 Rust and 6 architecture tests pass; independent implementation review/merge pending |

[Original change record](../MEMORY/records/2026-11-25-R0-01-rust-slint-documentation.md). The owner explicitly requested specialist agents on 2026-11-25; architecture, Windows/security and QA/release/UI reviewers have disjoint document ownership. They review one task, not three new implementation tasks.

Testing policy for this owner-authorised continuation: one active dedicated QA agent runs a sequential validation queue. Implementation agents do not execute verification commands. The reviewed baseline is integrated on main; R0-02 retains REVIEW until its task-specific independent acceptance is recorded. No commit or DONE status is implied for this continuation.

Current continuation: R0-06 adds a bounded STA icon worker that intentionally returns raw DIB data pending verified pixel semantics; R0-08 defers callback-data reads for negative hook codes and contains adapter panics while forwarding. Both passed sequential QA and remain partial pending native integration and VM evidence. R1-01 remains partial: the ConfigStore contract is [added and reviewed](../MEMORY/records/2026-11-26-R1-01-config-store-contract.md). R1-02 started under [ADR-031](../MEMORY/DECISIONS.md) and its [service](../MEMORY/records/2026-11-26-R1-02-persistence-service.md) is in REVIEW against the fake. A [cross-review](../MEMORY/records/2026-11-26-claude-cross-review.md) lists open findings. R0-05's recovery protocol spec is written and has [open review findings](../MEMORY/records/2026-11-26-R0-05-spec-review.md); native API and VM evidence remain open. See the [current parallel continuation record](../MEMORY/records/2026-11-26-parallel-r0-06-r0-08-worker-callback.md), the [earlier bounded-adapter record](../MEMORY/records/2026-11-26-parallel-r0-06-r0-08-follow-up.md) and [prior continuation evidence](../MEMORY/records/2026-11-25-autonomous-continuation.md).

## Next work

R0-02 remains REVIEW; see its [record](../MEMORY/records/2026-11-25-R0-02-rust-workspace.md). R0-03 is integrated on main with hosted Actions pending. R1-01's domain/port/fake and bounded R0-06/R0-08 adapters remain partial REVIEW and do not satisfy full migration/native acceptance. R1-02 still needs the native filesystem adapter, platform path keys and a legacy-file compatibility run. R0-06 PathKey/source and worker/UI/native integration, R0-08 native hook integration, R1-03 persistence/native integration and R0-04 full host acceptance remain pending. R0-10 and R1-06 remain BLOCKED on new evidence. No old DONE status satisfies these dependencies.

## Phase state

| Work | Status |
|---|---|
| R0 documentation | REVIEW: R0-01 reviewed baseline, merge pending |
| R0 Rust foundation/spikes | R0-02/R0-03 REVIEW; R0-04/R0-06/R0-08 partial REVIEW; R0-05 WIP; R0-07/R0-09 TODO; R0-10 BLOCKED |
| R1 Rust core/UI/parity | R1-01 partial REVIEW; R1-02 partial REVIEW (fake filesystem only); R1-03 pure subset REVIEW; R1-04/R1-05 TODO; R1-06 BLOCKED |
| P2–P5 Windows product phases | TODO, wait for Rust foundation/core |
| P6 other platforms | Post-v1, unverified Rust/Slint support |

## Open decisions

Q-19 blur MVP priority; Q-20 measured RAM/size budgets; Q-21 backend/renderer/native integration; Q-22 Rust packaging/updater compatibility. See [BACKLOG](./BACKLOG.md). Windows minimum, MIT/Drape name, no default network and opt-in updates remain unchanged.

## Historical completion ledger (C# / earlier documentation only)

The following original entries retain their dates/results and do not count toward Rust completion.

| ID | Title | Date | Record |
|---|---|---|---|
| P1-01 | Core domain model (records, validation, one item per Drape) | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P1-02 | Interface contracts in Core, with a fake for each | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P1-06 | Layout maths (`LayoutService`) in Core | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P1-11 | Debounced persistence with a shutdown flush | 2026-10-23 | [record](../MEMORY/records/2026-10-23-P1-core-domain-config-layout.md) |
| P0-01 | Scaffold repo and Avalonia solution | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P0-02 | CI on GitHub Actions + Dependabot | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P0-08 | App skeleton: composition root, DI, logging, single-instance, tray, `--restore-desktop` | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P1-03 | ConfigStore: atomic save, backup, quarantine, migration | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P1-07 | Create, rename and delete a Drape (Core logic) | 2026-10-25 | [record](../MEMORY/records/2026-10-25-phase0-p1-batch.md) |
| P0-10 | Custom subagent definitions | 2026-10-25 | [record](../MEMORY/records/2026-10-25-p0-10-p1-12-p1-04.md) |
| P1-12 | Phase 1 testing: coverage (Core 85.8%), baseline | 2026-10-25 | [record](../MEMORY/records/2026-10-25-p0-10-p1-12-p1-04.md) |
| SETUP-01 | Project structure: `docs/`, `MEMORY/`, `TASKS/`, `CLAUDE.md`, `AGENTS.md` | 2026-10-09 | [project setup](../MEMORY/records/2026-10-09-project-setup.md) |
| DEC-03 | Windows 10 22H2 minimum (owner); other platforms' minimums and channels (ADR-023) | 2026-10-10 | [record](../MEMORY/records/2026-10-10-windows-10-minimum-and-platform-minimums.md) |
| DEC-02 | Remaining open questions answered with research (ADR-022); root README and LICENSE | 2026-10-10 | [record](../MEMORY/records/2026-10-10-remaining-questions-readme-licence.md) |
| DEC-01 | Owner decisions: MIT, final name Drape, Windows first, mobile scope, `Fence`→`Drape` (ADR-019..021) | 2026-10-10 | [owner decisions](../MEMORY/records/2026-10-10-owner-decisions-licence-name-rename.md) |
| DOCS-01 | Restructure and translate all documentation to English; reference layout adopted | 2026-10-10 | [docs restructure](../MEMORY/records/2026-10-10-docs-restructure-and-translation.md) |

## Updating the board

Claim active card and board together; one owner, WIP <=3. Review by a different role before DONE. Add the record/index/progress update in the same commit. Do not overwrite historical completion evidence to manufacture Rust parity.
