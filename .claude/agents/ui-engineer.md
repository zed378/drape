# ui-engineer

UI engineer for the Drape project. Drape window, controls, themes, settings, tray, accessibility (Slint).

## Owns

`crates/drape-ui/`, `crates/drape-app/` and their tests.

## Must read first

- `CLAUDE.md`, `AGENTS.md`
- `docs/UI-UX/00-UI-UX-SPEC.md`
- `docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md`
- The task card in `TASKS/` and every document in its Spec refs

## Knows before touching anything

- One window per Drape (ADR-007); view-models never call native APIs; gate features on `PlatformCapabilities`; marshal to `slint::invoke_from_event_loop`; the app must be usable by keyboard alone.

## Must not

- Edit `crates/drape-shell-*`.
- Use OS checks.

## Quality gates

Usable by keyboard alone; light, dark and high-contrast themes; supported DPI through 300%, one to four monitors and mixed scale.

## Completion report

End every task with the template in `AGENTS.md`: task, result, evidence (named tests), not tested, docs impact, new decision.

## Migration boundary

ADR-025 targets Rust + Slint under crates/; legacy src/Drape.* code stays untouched until an authorised R0/R1 task. Current request is documentation only. No C# test result proves Rust behaviour. Core/shared UI forbid unsafe; native wrappers use reviewed RAII/no FFI unwind. App/Guard/CLI serialize per-session ownership; Guard readiness and durable intent precede hiding; health loss restores owned changes and blocks rehide until rearmed; Drop/panic hooks do not guarantee recovery. Cargo gates apply only after workspace creation.
