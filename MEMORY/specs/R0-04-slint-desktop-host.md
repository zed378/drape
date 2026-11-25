# Feature Spec — R0-04 Slint desktop-host and borrowed-handle spike

Written: 2026-11-25, before implementation. Author: shell-engineer-windows.
Status: partial diagnostic implementation; host strategies and environment matrix pending.

Spec refs: [Windows integration](../../docs/PLATFORM/00-WINDOWS-INTEGRATION.md), sections 1–2; [system architecture](../../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md); [Rust migration](../../docs/ARCHITECTURE/03-RUST-MIGRATION.md); [ADR-007](../DECISIONS.md); [security](../../docs/SECURITY/00-SECURITY-AND-PRIVACY.md).

## Problem and existing decisions

The workspace has a Slint window but no evidence that its public native-handle API works with the pinned backend. A real Slint window is required; a plain Win32 window cannot establish this boundary. One window per Drape remains required, and native ownership belongs to Slint. Strategy A (verified desktop parent), B (bottom-most top-level) and C (accepted A with labelled degraded B) remain hypotheses. Failed discovery never falls back blindly to Progman.

## This implementation boundary

The architect approved an isolated spike manifest with its own workspace/lock, exact Slint 1.13.1 baseline features plus `raw-window-handle-06`, and exact raw-window-handle 0.6.2. Root workspace/lock and shared crate interfaces remain unchanged. Slint-build remains 1.13.1. This is a diagnostic composition root, not a product capability or desktop-host implementation.

The harness selects winit/software explicitly, creates an actual compiled Slint window, and observes the public handle before show, after show, from an explicit UI callback and after hide. Every observation acquires a fresh provider and borrowed handle on the UI thread, formats a temporary diagnostic and drops both immediately. No raw handle is retained in state, passed to workers, wrapped as owned, destroyed, registered or used for native mutation. Generated Slint glue alone has the existing ADR-028 unsafe exception; hand-written adapters forbid unsafe.

The window displays a transparency sample, keyboard reachable sample/close buttons and AboutSlint attribution. An optional `--smoke` mode shows, samples and hides the diagnostic window without starting a persistent event loop. `--help` and unsupported-argument rejection happen before backend startup. Smoke is a real GUI operation for a dedicated interactive VM/test account, not a headless test or proof of desktop attachment.

## Safety, failure and privacy

There is no native icon hide/show path, recovery state, Guard, hook, registry operation, Explorer manipulation, filesystem operation or config/schema change. Errors obtaining handles are reported as unavailable; they never authorize host discovery or mutation. Backend/show/hide errors return failure. A Win32 handle is printed only as an ephemeral diagnostic identifier; it is not process identity or a stable registration key. No personal paths, input history or file names are logged. No network call occurs at runtime.

The harness intentionally implements none of A/B/C: reparenting/style/z-order changes require a separate reviewed shell adapter, rollback and verified discovery before use. There is no silent fallback and no production DesktopHost capability is enabled. No requirement is removed by this partial checkpoint.

## Acceptance and verification

QA alone generates the isolated lock, compiles Release, checks formatting/Clippy and tests CLI rejection. Manual interactive verification records exact Windows edition/build, backend/renderer, Rust/Slint/lock, DPI, monitor topology, transparency and each lifecycle observation. Before-show and after-hide availability are observations, never asserted fixed backend behavior. A successful handle lookup proves acquisition only, not safe reparenting or native resource lifetime after destruction.

Full R0-04 acceptance still requires A/B/C native adapters and Windows 10 22H2/Windows 11 23H2 and 24H2+ matrix: Win+D, peek, no focus stealing, keyboard focus, Alt+Tab/taskbar absence, manual Explorer restart in checkpointed VM, transparency, 100/150/200% DPI and mixed-DPI two-monitor/negative origins. No automation restarts Explorer. Reacquisition, detach-before-destruction and style/parent rollback remain native-adapter acceptance. See the [runbook](../../spikes/r0-04-slint-desktop-host/RUNBOOK.md) for result slots; all begin NOT RUN.
