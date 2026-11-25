# R0-02 — Rust workspace baseline

Date: 2026-11-25. Owner: architect. Branch: feat/R0-02-rust-workspace.
Status: REVIEW; local checks passed, different-role review and merge pending.

## Scope and implementation

Continued the existing seven-crate Rust scaffold under the owner's implementation authorisation (ADR-027). Rust 1.90.0, Slint 1.13.1, Cargo.lock and the original Slint royalty-free licence/notices are retained. Core exposes neutral capability data and a Clock port; FixedClock is deterministic. UI is a compiled foundation window with AboutSlint, not a production desktop panel. App routes CLI before renderer initialisation. Restore and Guard fail explicitly with code 2 until R0-05; neither mutates desktop state. Legacy source/CI remain preserved.

The Windows build helper originally missed the installed incomplete Visual Studio instance. It now queries all instances requiring the x64 C++ component and still runs VsDevCmd/linker checks. No installer/system modification was made.

Initial release build and Clippy exposed E0453: Slint-generated ItemTreeVTable_static glue cannot compile beneath forbid(unsafe_code). ADR-028 confines the exception to a private generated module; hand-written presentation code and Core forbid unsafe, while the UI root denies it. No hand-written unsafe or native UI calls were added. The UI handoff is restricted to this generated-code integration and renderer-independent CLI regression checks.

## Evidence so far

- `scripts/cargo-windows.cmd fmt --all -- --check`: initial pass; rerun after adapter extraction pending.
- `python scripts/check-rust-boundaries.py`: four original tests passed in 199.434 s, including real Cargo metadata mutations.
- Added `test_shared_ui_shell_dev_dependency_mutation_is_rejected` and `test_infrastructure_shell_build_dependency_mutation_is_rejected`: both passed in 145.329 s. These inspect fresh Cargo output after forbidden dev/build edges are injected into isolated manifests.
- Other named architecture tests: `test_actual_workspace_and_guard_dependency_boundaries`, `test_core_and_ui_forbid_unsafe_and_platform_access`, `test_forbidden_core_ui_dependency_mutation_is_rejected`, `test_guard_ui_dependency_mutation_is_rejected`.
- CLI integration tests now supply an invalid SLINT_BACKEND so help/restore cannot accidentally depend on renderer availability. Final execution pending.

## Not determined

No GUI runtime, Windows 10/11 build/DPI/monitor, desktop-host, recovery, UIA, performance, coverage or advisory audit is claimed. Metadata is a dependency/licence inventory including host/build and optional package information, not a list of shipped runtime binaries or complete release notices. Native mutation is disabled, so native safety matrix experiments belong to R0-04 onward. No commit, PR, merge or independent implementation review has occurred.

## Final verification (supersedes pending outcomes above)

- `scripts/cargo-windows.cmd build --workspace --release --locked`: pass, optimized profile, 2m 28s after the generated-code correction.
- `scripts/cargo-windows.cmd fmt --all -- --check`: pass after extraction.
- `scripts/cargo-windows.cmd clippy --workspace --all-targets --locked -- -D warnings`: pass, no warnings, 37.61s.
- `scripts/cargo-windows.cmd test --workspace --locked`: pass, 8 tests, 0 failed. Named tests: `help_and_default_route_are_explicit`, `restore_is_selected_before_ui_startup`, `unknown_and_combined_arguments_fail_closed`, `restore_reports_unavailable_without_starting_ui`, `help_exits_successfully_and_unknown_argument_fails`, `unverified_platform_has_no_enabled_capabilities`, `baseline_guard_cannot_acknowledge_readiness`, `clock_returns_supplied_pre_epoch_time_without_advancing`.
- Six architecture tests passed across the original four-test run and the two additional dev/build mutation tests. Following ADR-028 extraction, `test_core_and_ui_forbid_unsafe_and_platform_access` passed again in 22.973s; other manifest edges were unchanged.
- `rtk git diff --check`: pass after documentation closure. Local Markdown link validation across nine touched task/record/spec/architecture/standards files: 67 links checked, 0 missing. Required record/spec/source/licence paths exist.

No GUI runtime/native safety/performance evidence was added by these checks. No test waiver was requested. ADR-027 and ADR-028 are the new decisions; the pinned backend remains provisional pending Q-21 experiments.
