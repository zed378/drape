# R0-02 — Rust Workspace Baseline

Date: 2026-11-25. Role: architect. Owner request: continue implementation after the reviewed R0-01 documentation baseline. Spec refs: docs/ARCHITECTURE/00–03, docs/ENGINEERING/00, docs/PLAN/05; ADR-025/026.

## Scope and contracts

Create the seven Windows-v1 workspace members from ARCHITECTURE/03. Preserve all legacy source/build files. Use exact stable Rust 1.90.0 (released 2025-09-18) and an application lockfile. Slint 1.13.1 is the initial reproducible candidate, with default features disabled, winit/software rendering and accessibility; it is not the final backend/host decision Q-21. Validate the release's MSRV, feature graph and licence before accepting the pin. No async runtime, Qt or WebView is selected.

Core initially contains only neutral PlatformCapabilities (all unsupported by default), an injected Clock port and an owned timestamp value. R1-01 defines domain models and further ports before two-sided implementations. Shared Core/UI forbid unsafe; no environment, filesystem, process, OS checks or UI dependencies in Core. Infrastructure and UI depend on Core; Shell depends on Core; App composes all three; Guard depends only on minimal Shell/Core. Avoid empty future-platform crates.

UI compiles an original .slint baseline window with a visible non-production status and accessible Slint attribution. App parses help, invalid arguments and --restore-desktop before creating a component/event loop. Restore delegates to a minimal Shell contract and exits nonzero with an explicit unavailable result until R0-05 implements verified recovery. Guard similarly refuses to claim readiness/recovery. Neither binary changes icons, reads config or manipulates user files. Default startup opens a normal baseline window; it makes no desktop-host claim.

## Verification

- Locked release build, rustfmt, Clippy with warnings denied, workspace unit/doc tests.
- Architecture tests query actual cargo metadata, including target/build/dev dependency edges and resolved Guard closure. Reject Core external dependencies, shared Shell imports, Qt/WebView renderer edges and Slint in Guard. Mutation-check by injecting an actual forbidden dependency into an isolated manifest copy and obtaining fresh metadata; the negative case must fail the boundary validator.
- Named CLI tests cover help/unknown/restore parsing; process smoke checks require nonzero unsupported restore/Guard status and no UI startup. Capability defaults remain false.
- Record exact dependency/MSRV/licence/features and Win10 minimum evidence separately from actual Win10 runtime acceptance. Never infer Windows support from successful compilation on this machine.

## Deferred / risks

Implementation clarification ADR-028: Core and hand-written UI forbid unsafe; the UI root denies it and allows only the private Slint compiler-output module to use generated unsafe glue. Slint 1.13.1 fails E0453 under crate-wide forbid. The presentation adapter remains unconditionally forbidden from using unsafe.

Native host/recovery, real Guard protocol, persistence/schema ports, tray and production accessibility/localisation await their task dependencies. Baseline software rendering is provisional. No native mutations or legacy code retirement. Review remains pending a different role; local work stays REVIEW until merged. Windows 10/11 manual build/DPI/monitor, distribution performance and final backend selection are not determined by this task.
