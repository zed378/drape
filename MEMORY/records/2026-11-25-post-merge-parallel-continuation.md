# Post-merge parallel continuation

Date: 2026-11-25. Status: REVIEW (R1-01 partial implementation; R1-02 spec only). Authority: owner request to continue implementation in parallel; ADR-029's one sequential validation owner remains in force.

## Ownership and scope

- `core_r101` owns the R1-01 capability fake and focused contract tests in `crates/drape-test-support/`.
- The orchestrator/architect owns the R1-02 config persistence feature spec in `MEMORY/specs/` and task/index updates.
- `qa_queue` is the sole owner of command execution and validates the implementation in a sequential queue.
- Core and spec files are disjoint. No Cargo manifest/lock, Windows native host, icon visibility, recovery-state, or legacy source changes are in scope.

## R1-01 handoff

`FakePlatformCapabilities` defaults to all unsupported and accepts explicit capability values. Named tests: `fake_capabilities_default_to_unsupported` and `fake_capabilities_preserve_explicit_partial_support`.

QA independently reviewed the final capability fake and reports both focused tests passed. Final sequential commands through `rtk proxy scripts/cargo-windows.cmd`: `build --workspace --release --locked`, `fmt --all -- --check`, `clippy --workspace --all-targets --locked -- -D warnings`, and `test --workspace --locked` passed (40 tests). The architecture suite passed all six cases: `test_actual_workspace_and_guard_dependency_boundaries`, `test_core_and_ui_forbid_unsafe_and_platform_access`, `test_forbidden_core_ui_dependency_mutation_is_rejected`, `test_guard_ui_dependency_mutation_is_rejected`, `test_infrastructure_shell_build_dependency_mutation_is_rejected`, and `test_shared_ui_shell_dev_dependency_mutation_is_rejected`.

Initial format failed on import ordering; Core formatted its two owned paths before the final green queue. Initial architecture execution discovered zero tests because subprocess Cargo was missing from PATH; QA configured the repository-local toolchain and reran successfully. No coverage or dependency delta exists (test-support changes only), so the unchanged Core coverage and dependency audits were not repeated. Architect source review found no Core interface change, native/I/O access or capability enablement in production. The fake represents configured test data only; consumer gating remains future integration evidence.

## R1-02 specification

The [feature spec](../specs/R1-02-durable-config.md) was written before persistence implementation and indexed. It keeps schema 1, separates lossless wire data from `DomainConfig`, defines injected filesystem and infrastructure ownership, and excludes `state.json`/Guard recovery. R1-02 remains spec-only until R1-01 acceptance; no persistence code or dependency change is authorized by this checkpoint.

QA static review found no material blocker in the spec; method signatures, parser selection and numeric input limits remain prerequisites before implementation. Final documentation validation passed for six changed/new Markdown files and 78 local links, with trailing whitespace checked including untracked files (Markdown two-space hard breaks accepted). `rtk git diff --check` exited zero.

## Not determined

Exact config resource limits require comparison with a sanitized legacy-format corpus. Process-failure tests cannot prove universal storage power-loss guarantees. No hosted CI, native Windows matrix, runtime GUI, config migration or recovery-state behavior was tested by this continuation checkpoint. No commit/merge/DONE status is implied.
