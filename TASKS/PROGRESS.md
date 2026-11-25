# Progress

Single status board. Updated 2026-11-25 for [ADR-025](../MEMORY/DECISIONS.md). Target: Rust + Slint. The owner authorised Rust implementation continuation on 2026-11-25. R0-02 establishes the workspace; legacy source/CI remain preserved.

## Current state

The owner authorized commit, merge to main and push of the reviewed accumulated baseline on 2026-11-25. See the [integration record](../MEMORY/records/2026-11-25-main-integration.md). REVIEW rows below retain unfinished acceptance; earlier local/unmerged wording describes the validation checkpoint rather than a prohibition on this authorized integration.

Early C#/.NET/Avalonia code and spike harnesses remain. Previous board reported 98 tests (71 Core, 14 Infra, 7 architecture, 6 UI); not rerun here and not Rust evidence. Legacy P0/P1 status tables sometimes disagree with that board; they are frozen as historical snapshots, not active assignments. The Rust baseline starts with fresh R0/R1 IDs in [RUST-MIGRATION](./RUST-MIGRATION.md).

Legacy P1-04 partial UI and P0-03–P0-07 VM-pending harnesses are superseded as active work by Rust replacement tasks. Their source, specs, runbooks and dated records are preserved. No legacy task ID is reused or relabelled as Rust DONE. Old VM ZIPs require .NET and cannot validate Slint.

## In progress (WIP limit 3)

| ID | Work | Owner | Started |
|---|---|---|---|
| — | No active implementation; final QA/document checks | qa | 2026-11-25 |

## Review

| ID | Work | Owner | Status |
|---|---|---|---|
| R0-04 | Slint native-handle diagnostic (partial spike) | shell-engineer-windows | Isolated locked Release/format/Clippy and 2 CLI tests pass; architect/QA static review passed; full native hosting and Windows matrix pending |
| R0-03 | Rust CI and quality gates | release-engineer | QA/static review passed locally; final workspace gates, 86.75% Core coverage and current advisory/licence/source audit pass; hosted Actions and merge pending |
| R1-03 | Pure layout, snap and monitor recovery | core-engineer | 14 new tests, no-drift mutation and integrated gates pass; persistence/native integration and full prerequisites pending |
| R1-01 | Domain models and neutral read-only ports (partial) | core-engineer | Architect-reviewed subset; 18 targeted Core/test-support tests pass; remaining contracts and integrated gates pending |
| R0-01 | Rust + Slint documentation baseline | architect | Specialist documentation review complete; merge pending |
| R0-02 | Rust workspace, contracts and dependency/licence baseline | architect | Local release/format/Clippy, 8 Rust and 6 architecture tests pass; independent implementation review/merge pending |

[Original change record](../MEMORY/records/2026-11-25-R0-01-rust-slint-documentation.md). The owner explicitly requested specialist agents on 2026-11-25; architecture, Windows/security and QA/release/UI reviewers have disjoint document ownership. They review one task, not three new implementation tasks.

Testing policy for this owner-authorised continuation: one active dedicated QA agent runs a sequential validation queue for both implementation owners. Implementation agents do not execute verification commands. R0-02 remains REVIEW; dependent local work is explicitly authorised by the owner continuation request, with independent baseline review queued before acceptance. Existing branch feat/R0-03-rust-ci and uncommitted baseline are preserved; no merge or DONE status is implied.

Current continuation: `layout` owns the new pure layout module/tests; `host_spike` owns an isolated read-only Slint native-handle diagnostic/spec; `qa` is the sole sequential verification owner. Root owns shared task/continuity files. See the [ownership record](../MEMORY/records/2026-11-25-parallel-layout-host-continuation.md). R1-03 uses only the reviewed R1-01 data subset; complete dependent acceptance remains pending. No desktop-host strategy or native-icon mutation is authorized by the diagnostic harness.

## Next work

R0-02 remains REVIEW; see its [record](../MEMORY/records/2026-11-25-R0-02-rust-workspace.md). Current shared work preserves feat/R0-03-rust-ci. R0-03 is locally reviewed; R1-03 pure layout and R0-04 diagnostic are reviewed subsets with integration/native evidence pending. R0-05–R0-09 experiments retain explicit dependencies. R0-10 and R1-06 remain BLOCKED on new evidence. No old DONE status satisfies these dependencies.

## Phase state

| Work | Status |
|---|---|
| R0 documentation | REVIEW: R0-01 reviewed baseline, merge pending |
| R0 Rust foundation/spikes | R0-02/R0-03 REVIEW; R0-04 diagnostic subset REVIEW; R0-05–R0-09 TODO; R0-10 BLOCKED |
| R1 Rust core/UI/parity | R1-01 partial REVIEW; R1-03 pure subset REVIEW; R1-02/R1-04/R1-05 TODO; R1-06 BLOCKED |
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
