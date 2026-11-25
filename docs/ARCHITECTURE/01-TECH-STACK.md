# 01 — Tech Stack

Accepted target: **Rust + Slint**, owner decision [ADR-025](../../MEMORY/DECISIONS.md), superseding ADR-012's C#/.NET/Avalonia stack. No Rust workspace, pinned dependencies or implementation are created by this documentation update.

## Choices

| Area | Target | Status / rationale |
|---|---|---|
| Language | Rust stable, exact toolchain selected/pinned at R0-02 | Ownership/borrowing and native compilation; RAM/performance/size benefits require measurement |
| UI | Slint with compiled .slint components and Rust adapters | Owner selected; no WebView, browser engine or JavaScript UI runtime |
| Backend/renderer | Eligible native non-Qt deployment | R0-04/R0-07 compare supported renderers, handle API and host compatibility |
| State/composition | Core state, Slint models/properties/callbacks; explicit constructor injection | No reflection DI or .NET Generic Host |
| Windows integration | Microsoft's windows crate, minimal features | Win32/COM isolated in drape-shell-windows; verify API/OS minimum |
| Serialisation | serde + serde_json in persistence adapters | Preserve existing schema/names/values and unknown fields; platform-free document transformations may use neutral value data without performing I/O/parsing in Core |
| Errors | Typed Result; consider thiserror | No panic for expected environment failures |
| Logging | tracing, tracing-subscriber, rolling writer such as tracing-appender | Local/redacted, bounded retention and explicit flush lifetime |
| Watcher | notify candidate behind DesktopItemSource | Debounce, overflow/rescan and shutdown tested |
| Tray | tray-icon candidate or Windows adapter | Prove native/Slint event-loop integration |
| Concurrency | Standard threads and bounded channels first | No mandatory Tokio/general async runtime |
| Tests | Rust unit/integration/doc tests, hand-written fakes | tempfile/proptest only if useful; Core coverage >=80% |
| UI/accessibility tests | Slint testing support and real Windows UIA/manual checks | Verify exact version; no invented Avalonia.Headless equivalents |
| Release | Windows x64 App + independent Guard, portable zip, per-user installer, winget | Evaluate existing Velopack preference for Rust: Q-22 / P5-03 |
| CI | GitHub Actions, exact toolchain, locked dependencies | Current workflows remain legacy until R0-03 |

These crates are candidates, not installed or pinned packages. R0-02 checks releases, MSRV, licence, maintenance, transitive features and minimum Windows support. Commit Cargo.lock. Select necessary Slint features rather than unused backends/renderers. Native compilation does not imply OS-native widgets.

Feature review covers both target and host/build dependency graphs: build-time Slint generation and testing features must not silently enable another production backend. Record exactly which executable owns each dependency; Cargo's unified feature graph is not an excuse to ship Slint or broad application features with Guard. Verify Guard's own binary/runtime imports and footprint using its supported build command and release artefact, not only the workspace manifest.

## Workspace and future gates

[Migration specification](./03-RUST-MIGRATION.md) defines paths. Future build controls are Cargo.toml, Cargo.lock and rust-toolchain.toml; legacy .NET files remain until separately authorised.

Valid only after R0-02, not build evidence today:

```bash
rtk cargo build --workspace --release --locked
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run -p drape-app -- --restore-desktop
```

Do not indiscriminately enable --all-features: backend/platform features can conflict. Test the documented supported matrix.

## Safety and performance

- Core/shared UI forbid unsafe and OS-specific code. Native adapters use reviewed narrow unsafe blocks, safety comments and RAII.
- Guard has no Slint, renderer, broad watcher or application runtime; only minimal recovery dependencies.
- Measure LTO, codegen units, stripping, CRT linkage and panic strategy. Abort invalidates in-process panic recovery; Guard remains mandatory.
- Windows manifests preserve PerMonitorV2/long-path/asInvoker as applicable. Restore mode bypasses renderer startup.
- Windows 10 22H2 x64 is a real test gate. Document deployment prerequisites instead of promising a universally dependency-free binary.

## Slint licence

Drape's own code remains MIT. Slint is not MIT. The [royalty-free desktop/mobile/web licence 2.0](https://github.com/slint-ui/slint/blob/master/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md) reviewed 2026-11-25 requires attribution. Plan AboutSlint accessible through top-level Settings/About; verify the exact pinned release and bundled notices in R0-02 and before distribution. Do not silently substitute another licence option.

## References and deferred platforms

[Owner-supplied Gemini discussion](https://share.gemini.google/UwyvsCzSs0aM) informs the design, not API correctness or performance promises. [Slint documentation](https://docs.slint.dev) and [Microsoft Windows bindings](https://github.com/microsoft/windows-rs) are primary implementation references.

macOS/Linux remain post-v1; maintained native Rust bindings/protocols are selected after spikes, no version chosen today. Mobile is a companion UI, not a desktop layer. ADR-023 non-Windows minimums remain planning targets requiring Rust/Slint revalidation; old Avalonia support tiers are no longer evidence.
