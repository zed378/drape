# R0-04 — Safe Slint borrowed-handle diagnostic

Date: 2026-11-25. Owner: shell-engineer-windows (host_spike). Status: REVIEW (partial); full native spike acceptance and merge pending.

Spec refs: [spec](../specs/R0-04-slint-desktop-host.md), Windows integration sections 1–2, system architecture, Rust migration, ADR-007. The spec was written before implementation. The architect approved an isolated manifest/lock with exact Slint/slint-build 1.13.1 and raw-window-handle 0.6.2; no product workspace feature/lock change.

## Result and review

`spikes/r0-04-slint-desktop-host/` contains a real compiled Slint diagnostic, explicit winit/software selection, AboutSlint attribution, CLI routing and a runbook. The probe observes before show, after show, an explicit UI callback and after hide. Provider and borrowed native handle remain local to each observation. Weak callbacks avoid component cycles. The generated module alone has the existing ADR-028 unsafe exception; hand-written probe code forbids unsafe.

Architect static review found no retained raw handle/provider, native registration or native mutation path. The diagnostic does not attach to Explorer, set styles/z-order, hide icons or enable DesktopHost capabilities. Normal top-level rendering cannot prove any A/B/C desktop-host strategy. `--smoke` is an interactive GUI operation and is not automatically executed by build/test validation.

## Verification and limitations

QA generated the isolated lock offline using the root baseline as seed; the root lock was untouched. Initial offline Release compilation passed in 3m58s; the final `--locked --offline` Release check passed in 3.28s. Spike format and denied-warning Clippy passed, as did both named tests: `help_and_smoke_are_explicit_before_backend_startup`, `native_mutation_flags_and_combined_arguments_are_rejected`. Compiled Release `--help` returned zero and `--hide-icons` returned 2 before renderer startup, verified through a subprocess. No GUI was launched. QA and architect static safety review found no blocker in this diagnostic scope. See the [QA evidence](2026-11-25-rust-parallel-qa-continuation.md).

Full R0-04 acceptance remains incomplete: validated native host discovery, A/B/C adapters, style/parent rollback, detach/reacquisition, Win+D/peek, focus, taskbar/Alt+Tab, transparency, manual Explorer restart and Windows 10/11 build/DPI/monitor matrix are NOT RUN. No GUI/native lifetime/resource/performance claim follows from compilation. No user data or native icons are modified; no commit/merge/DONE claim.
