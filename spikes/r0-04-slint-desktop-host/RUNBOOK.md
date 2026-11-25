# R0-04 — Slint borrowed-handle diagnostic

Status: partial spike implementation, 2026-11-25. This harness implements native-handle acquisition only. It is a normal top-level Slint window; it does not attach to Explorer, maintain bottom z-order or implement A/B/C. No desktop-host result is claimed.

Read the [spec](../../MEMORY/specs/R0-04-slint-desktop-host.md) and [Windows integration requirements](../../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) before use. Runtime is intended for a non-elevated dedicated interactive VM/test account. Normal desktop icons remain visible throughout.

## Build and sequential QA

Run from the repository root using the pinned Rust 1.90.0 MSVC toolchain and existing helper. The manifest is deliberately its own workspace, so the root Cargo.lock does not cover it. QA generates and retains its own Cargo.lock before locked validation. Root manifests and lock must not change. Existing product attribution/licence files apply; the probe also renders AboutSlint.

```powershell
rtk proxy scripts/cargo-windows.cmd generate-lockfile --offline --manifest-path spikes/r0-04-slint-desktop-host/Cargo.toml
rtk proxy scripts/cargo-windows.cmd fmt --manifest-path spikes/r0-04-slint-desktop-host/Cargo.toml -- --check
rtk proxy scripts/cargo-windows.cmd build --release --locked --offline --manifest-path spikes/r0-04-slint-desktop-host/Cargo.toml
rtk proxy scripts/cargo-windows.cmd clippy --all-targets --locked --offline --manifest-path spikes/r0-04-slint-desktop-host/Cargo.toml -- -D warnings
rtk proxy scripts/cargo-windows.cmd test --locked --offline --manifest-path spikes/r0-04-slint-desktop-host/Cargo.toml
```

QA may apply rustfmt to this spike only if needed. Reusing the already generated root lock as an initial seed is permitted only if Cargo updates it for this isolated package and checks its own dependency graph. Do not remove locked/offline flags to conceal a resolver mismatch. Missing cached dependencies require normal approved dependency access.

Named CLI tests: `help_and_smoke_are_explicit_before_backend_startup`, `native_mutation_flags_and_combined_arguments_are_rejected`. These do not create a GUI. Test the compiled binary `--help` returns zero and unknown arguments return 2 without renderer startup.

## Interactive acquisition check

The executable is normally `spikes/r0-04-slint-desktop-host/target/release/drape-r0-04-handle-probe.exe`. If QA chooses a shared target directory, record the actual output location. Launch it without arguments in the test account. Confirm a real Slint window, transparency sample and AboutSlint. Tab to Sample and activate it using Space/Enter, then tab to Close and activate it. Window-manager close is also supported. Record stdout/stderr outside the repository or in a sanitized result artifact.

The optional `--smoke` performs show/sample/hide and exits immediately; it requires a GUI session and does not exercise sustained rendering, keyboard input, desktop parenting or an event-loop callback. Build/test automation must not assume an interactive session.

Expected observation labels: `before-show`, `after-show`, `ui-sample` (interactive button only), `after-hide`, followed by a component-drop message. A Windows observation should identify Win32 or explicitly report unavailability. Record availability at every phase without assuming before-show/after-hide handle existence. Acquiring a handle is not proof it remains valid after recreation. The program never retains a raw handle, calls DestroyWindow, or probes a handle after component destruction.

## Evidence sheet

Fill one sheet per exact environment, with no inferred passes.

| Field | Result |
|---|---|
| Date/operator/VM checkpoint | NOT RUN |
| Windows edition/version/build | NOT RUN |
| Rust/Slint/dependency lock revision | 1.90.0 / 1.13.1 / record actual lock |
| Backend/renderer | requested winit/software; runtime NOT RUN |
| DPI and text scale per monitor | NOT RUN |
| Monitor count, origins and work areas | NOT RUN |
| before-show / after-show / UI sample / after-hide | NOT RUN |
| Keyboard sample/close | NOT RUN |
| Transparency over contrasting background | NOT RUN |
| Normal close and component teardown | NOT RUN |
| Errors and retained resources | NOT RUN |

## Remaining full host matrix

Run A/B/C only after the next reviewed shell adapter implements validated discovery and style/parent rollback. This probe has no strategy flags and rejects attempted mutation flags. Never manually reparent this window using external tools and describe that as harness acceptance. No strategy is currently accepted and no fallback to Progman exists.

Required environments: Windows 10 22H2 minimum, Windows 11 23H2 and 24H2+ with exact builds; 100/150/200% DPI and two mixed-DPI monitors including negative origin. Add wider DPI/monitor range in R0-07. For each A/B/C environment record:

| Criterion | A verified parent | B bottom-most top-level | C accepted A/degraded B |
|---|---|---|---|
| Host discovery ownership/class/relationships unambiguous | NOT IMPLEMENTED | n/a | NOT IMPLEMENTED |
| Above wallpaper and below applications | NOT RUN | NOT RUN | NOT RUN |
| Win+D and Win+, peek | NOT RUN | NOT RUN | NOT RUN |
| No focus stealing on show; explicit keyboard focus works | NOT RUN | NOT RUN | NOT RUN |
| Absent from Alt+Tab and taskbar | NOT RUN | NOT RUN | NOT RUN |
| Transparency, mixed DPI, negative origins | NOT RUN | NOT RUN | NOT RUN |
| Manual Explorer restart rediscovery/reattach | NOT RUN | NOT RUN | NOT RUN |
| Partial attach failure rolls back original styles/parent | NOT RUN | NOT RUN | NOT RUN |
| Detach/unregister precedes Slint window destruction | NOT RUN | NOT RUN | NOT RUN |
| Host loss/ambiguous discovery leaves native icons visible | NOT RUN | NOT RUN | NOT RUN |

Explorer restart is manual only in a checkpointed VM/test account. This harness provides no restart command or icon manipulation. A bottom-most window that fails Win+D remains degraded; do not label it a functioning desktop host. R0-10 chooses a strategy only from actual evidence.

## Primary API evidence

Pinned dependency source, inspected 2026-11-25: Slint/i-slint-core 1.13.1 `api.rs`, `Window::window_handle`, `WindowHandle` and its `HasWindowHandle` implementation; Slint 1.13.1 feature `raw-window-handle-06`; backend selector 1.13.1 `backend_name`/`renderer_name`; raw-window-handle 0.6.2 borrowed-handle contract. Corresponding public references: [Slint Window](https://docs.rs/slint/1.13.1/slint/struct.Window.html), [HasWindowHandle](https://docs.rs/raw-window-handle/0.6.2/raw_window_handle/trait.HasWindowHandle.html). Static source inspection is not a successful runtime test.
