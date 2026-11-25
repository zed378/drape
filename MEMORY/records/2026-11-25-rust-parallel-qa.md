# Rust parallel implementation — sequential QA

Date: 2026-11-25. Role: qa. Status: WIP.

The owner requested parallel implementation with exactly one testing agent and sequential test execution. The QA agent owns the validation queue; implementation agents author tests but do not execute validation. No validation processes overlap.

## Queue

1. R1-01 tests-first red phase when definitions are stable.
2. R0-03 CI-specific checks when implementation is stable.
3. R1-01 targeted final checks and behaviour review.
4. Integrated locked Release build, format, denied-warning Clippy, workspace tests, architecture mutations and Core coverage.
5. Final documentation links and whitespace checks.

## Review baseline

R0-02 source inspection confirms App parses CLI before Slint startup, restore returns an explicit unavailable error, and Guard exits without readiness. Native desktop mutation is absent. This is a safe foundation limitation, not implemented recovery. Existing R0-02 evidence is historical until rerun here.

## Evidence

- scripts/ci/test-core-coverage.py: five tests passed: test_accepts_exactly_eighty_percent, test_rejects_below_eighty_percent, test_rejects_impossible_coverage_counters, test_rejects_missing_core_files, test_rejects_zero_executable_lines.
- scripts/ci/check-quality-gates.py: two real isolated tests passed: test_clippy_accepts_clean_source_and_denies_warning_mutation, test_rustfmt_accepts_clean_source_and_rejects_format_mutation. Both first require clean input to pass, then require mutated input to fail.
- scripts/cargo-windows.cmd test -p drape-core --test domain --locked: expected red before domain implementation, E0432 unresolved drape_core::domain.
- scripts/cargo-windows.cmd test -p drape-test-support --test ports --locked: expected red before port/fake implementation, E0583 missing ports module.
- cargo deny --locked check advisories licenses sources with cargo-deny 0.18.3: failed to parse current RustSec CVSS 4.0 advisory RUSTSEC-2026-0552. CI owner replaced helper pin with Rust-1.90-compatible 0.21.0; installation/retest pending. No advisory was ignored or removed.
- Gitleaks 8.24.3 Git-history scan with redaction: 23 commits, 1.69 MB, no leaks found. This scans committed history, not current untracked changes.


## Not determined

Scanner upgrade timing: the approved `cargo install cargo-deny --version 0.21.0 --locked` began at 15:37:34 Asia/Jakarta and completed successfully in 13m02s. The installation remained the only active build/validation process; Core green checks stayed queued. The original QA agent then handed the queue to a replacement QA agent; they did not execute validation concurrently.

## Integrated validation continuation

- `rtk proxy scripts/cargo-windows.cmd fmt --all -- --check` first reported formatting differences only in the new Core/test-support source/tests. Under the explicit formatting handoff, QA ran `rtk proxy scripts/cargo-windows.cmd fmt -p drape-core -p drape-test-support`; the workspace format check then passed.
- `rtk proxy scripts/cargo-windows.cmd build --workspace --release --locked` passed without warnings in 1m24s, before the Clippy-only source simplification below.
- `rtk proxy scripts/cargo-windows.cmd clippy --workspace --all-targets --locked -- -D warnings` found `clippy::collapsible_if` in the duplicate-item identity guard. The Core owner replaced the nested condition with an if-let chain without suppressing the lint. The same command then passed in 12.05s.
- `rtk proxy scripts/cargo-windows.cmd test --workspace --locked` passed all 24 tests. This overlapped the small owner edit at compilation time, so exact final-source validation is queued separately rather than assumed. Besides the domain/port names in the R1-01 record, named baseline tests are `unknown_and_combined_arguments_fail_closed`, `help_and_default_route_are_explicit`, `restore_is_selected_before_ui_startup`, `restore_reports_unavailable_without_starting_ui`, `help_exits_successfully_and_unknown_argument_fails`, `unverified_platform_has_no_enabled_capabilities`, `baseline_guard_cannot_acknowledge_readiness`, and `clock_returns_supplied_pre_epoch_time_without_advancing`.
- With repository-local CARGO_HOME/RUSTUP_HOME and tool PATH, `rtk proxy python -X utf8 scripts/check-rust-boundaries.py` passed six tests in 150.657s: `test_actual_workspace_and_guard_dependency_boundaries`, `test_core_and_ui_forbid_unsafe_and_platform_access`, `test_forbidden_core_ui_dependency_mutation_is_rejected`, `test_guard_ui_dependency_mutation_is_rejected`, `test_shared_ui_shell_dev_dependency_mutation_is_rejected`, and `test_infrastructure_shell_build_dependency_mutation_is_rejected`. Mutations use real Cargo metadata from isolated manifest copies.
- On final Core source, `rtk proxy scripts/cargo-windows.cmd llvm-cov --package drape-core --locked --json --output-path coverage/core.json --fail-under-lines 80` passed its 14 Core tests. `rtk proxy python -X utf8 scripts/ci/check-core-coverage.py coverage/core.json` independently confirmed every Core source file present and **496/609 lines (81.44%)**. `rtk proxy scripts/cargo-windows.cmd llvm-cov report --package drape-core --lcov --output-path coverage/core.lcov` exported the LCOV report successfully.
- The 0.21.0 full audit first failed fetching RustSec in the sandbox with Windows TLS `SEC_E_NO_CREDENTIALS`. Approved elevated execution then encountered Git ownership checking on the exact sandbox-created advisory cache. A second approved rerun supplied a process-only `safe.directory` exception for that single cache via GIT_CONFIG environment entries; global Git configuration and advisory policies were not modified.

No GUI, real desktop, Windows build/DPI/monitor, recovery, UIA or performance matrix is exercised by this queue. Hosted workflow execution and network-dependent security scans require separate actual evidence.
