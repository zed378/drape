# R1-01: ConfigStore contract, save gate and fake

Date: 2026-11-26. Author: Claude Code session as core-engineer; the owner assigned R1-01 to it. Status: partial REVIEW. No commit, merge or DONE is implied.

## Scope

Added the platform-free persistence boundary named in the system architecture and the R1-02 spec. No parser, native adapter, dependency, schema or Cargo manifest change.

| File | Content |
|---|---|
| `crates/drape-core/src/config_store.rs` | `ConfigStore`, `ConfigLoad`, `ConfigSave`, `ConfigStoreError`, `SaveGate` |
| `crates/drape-test-support/src/config_store.rs` | `FakeConfigStore`, scripted and gated |
| `crates/*/tests/config_store.rs` | 10 Core and 6 fake tests |
| both `lib.rs` | one module line each |

## Contract

- A save needs a writable baseline: a `Loaded` current-schema config or `Empty`. `ReadOnly`, a failed load and no load all refuse.
- The caller issues a new `OperationId` per load. A save names the load it derives from; a superseded one returns `StaleBaseline`.
- `DocumentReadOnly` reports a save that found the stored document unsafe to rewrite and latches read-only. A plain `ReadOnly` refusal does not change the gate.
- `Conflict`, `NotFound` and `AlreadyExists` on save drop the baseline, so the caller reloads.
- Any indeterminate result latches `Stopped` for both load and save. Recovery needs reconciliation and a new store.
- `ReadOnly` loads carry `origin` and `quarantined`, so a recovery notice survives a missing projection.

## Different-role review

`codex exec` in a read-only sandbox, acting as architect, reviewed the four new files. First pass: ACCEPT WITH CHANGES with three findings (save not bound to its load baseline; unsafe-to-rewrite save indistinguishable from an argument refusal; read-only recovery could not report backup or quarantine). Tests were updated first, the code was changed, and a second pass returned ACCEPT with no remaining defect. The reviewer ran no builds or tests.

## Evidence

Run on copies of the repository outside the tree, offline, Rust 1.90.0.

- Tests-first: the first Core run failed with E0432 (module absent); the post-review tests failed with E0061, E0559 and E0599 before the fix.
- Fresh copy after applying: format check, denied-warning Clippy, 126 workspace tests and 7 boundary tests pass.
- Release build and Core coverage on the same sources before applying: build passes; Core lines 89.25%, `config_store.rs` 88.89% (uncovered lines were not itemised).
- Six mutations each failed a named test, not compilation: unknown gate allows save; stopped latch removed; conflict keeps baseline; newer schema writable; stale baseline accepted; unsafe document stays writable.

## Remaining and not determined

- R1-01 still lacks host, icon, recovery, hook, context-menu, display, autostart and blur contracts. They wait for R0 spike evidence.
- The R1-02 card depends on R1-01. Whether the config-related subset is enough to unblock R1-02 is an architect decision and has not been taken here.
- No real store, wire format, filesystem replacement or Windows behaviour is tested.

## Handoff core-engineer to architect and orchestrator, 2026-11-26

- Context: the owner assigned R1-01 while Codex continues R0-05, R0-06 and R0-08.
- Done: contract, fake, tests, architect review.
- Not done: board, card, index and changelog entries, which the orchestrator owns during this split.
- Contract/interface changed: new module `drape_core::config_store`; `ports.rs` unchanged by this work.
- How to verify: `scripts/cargo-windows.cmd test --workspace --locked`.
- Risks / surprises: the gate trusts callers to issue a unique `OperationId` per load; reuse defeats the stale-save check.
