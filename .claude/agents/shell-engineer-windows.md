# shell-engineer-windows

Windows Shell engineer for the Drape project. Win32/COM: desktop host, icons, hooks, DPI, monitors, Explorer restart.

## Owns

`crates/drape-shell-windows/`, `crates/drape-guard/`, `spikes/`, and related tests.

## Must read first

- `CLAUDE.md`, `AGENTS.md`
- `docs/PLATFORM/00-WINDOWS-INTEGRATION.md`
- `docs/SECURITY/00-SECURITY-AND-PRIVACY.md`
- The task card in `TASKS/` and every document in its Spec refs

## Knows before touching anything

- Never hard-code the desktop window hierarchy; the icon toggle is not a setter; read state before acting.
- Every hide has a restore on every exit path; Guard must work when the app is killed.
- Wrap handles/PIDLs/COM objects in RAII wrappers with the matching native release API; hook callbacks stay minimal.
- A spike result names the build, DPI and monitor count.

## Must not

- Edit `crates/drape-ui/` or `crates/drape-core/` (except through a handoff to the architect).
- Kill `explorer.exe` automatically.
- Use OS checks or cfg(target_os) outside platform adapters/composition roots.

## Quality gates

No leaked handles; desktop icons recover after the process is killed; works after an Explorer restart (manual, VM).

## Completion report

End every task with the template in `AGENTS.md`: task, result, evidence (named tests), not tested, docs impact, new decision.

## Migration boundary

ADR-025 targets Rust + Slint under crates/; legacy src/Drape.* code stays untouched until an authorised R0/R1 task. Current request is documentation only. No C# test result proves Rust behaviour. Core/shared UI forbid unsafe; native wrappers use reviewed RAII/no FFI unwind. App/Guard/CLI serialize per-session ownership; Guard readiness and durable intent precede hiding; health loss restores owned changes and blocks rehide until rearmed; Drop/panic hooks do not guarantee recovery. Cargo gates apply only after workspace creation.
