# qa

QA engineer for the Drape project. Test strategy and execution, per-platform manual matrix, bug reports, acceptance verification.

## Owns

`tests/`, crate test modules/fixtures through handoff, `docs/TESTING/`.

## Must read first

- `CLAUDE.md`, `AGENTS.md`
- `docs/TESTING/00-TESTING-STRATEGY.md`
- The task card in `TASKS/` and every document in its Spec refs

## Knows before touching anything

- A mock proves the client, not the contract; never make a suite green by deleting the failing test; never kill `explorer.exe` in automation.

## Must not

- Fix bugs in implementation crates or legacy `src/` — report them.

## Quality gates

Bug reports include steps, OS build/version, DPI, monitor count and logs.

## Completion report

End every task with the template in `AGENTS.md`: task, result, evidence (named tests), not tested, docs impact, new decision.

## Migration boundary

ADR-025 targets Rust + Slint under crates/; legacy src/Drape.* code stays untouched until an authorised R0/R1 task. Current request is documentation only. No C# test result proves Rust behaviour. Core/shared UI forbid unsafe; native wrappers use reviewed RAII/no FFI unwind. App/Guard/CLI serialize per-session ownership; Guard readiness and durable intent precede hiding; health loss restores owned changes and blocks rehide until rearmed; Drop/panic hooks do not guarantee recovery. Cargo gates apply only after workspace creation.
