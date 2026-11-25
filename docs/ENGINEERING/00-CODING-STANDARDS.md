# 00 — Coding Standards

Target Rust + Slint per [ADR-025](../../MEMORY/DECISIONS.md). Applies to future Rust crates/tests; existing C# is not ported by this document. CS rule IDs remain stable for review references.

## Language and structure

- **CS-1** Code, identifiers, comments, branches and commits are English.
- **CS-2** Repository documentation is English.
- **CS-3** Localisable Slint translations/resources for id-ID and en-US; verify pinned workflow.
- **CS-4** Conversation follows the owner's language (Indonesian).
- **CS-5** Cohesive Rust modules; one type per file is not mandatory.
- **CS-6** Packages kebab-case (drape-core), imports/modules snake_case (drape_core::layout); follow target workspace.
- **CS-7** Types/traits UpperCamelCase; functions/fields/modules snake_case; constants SCREAMING_SNAKE_CASE. Follow rustfmt/Rust API Guidelines.
- **CS-8** Traits omit C# I prefix: FileSystem, Clock, DesktopHost. PlatformCapabilities is neutral data. Concrete adapters use descriptive names.
- **CS-9** No Async suffix requirement. Define cancellation/work bounds/shutdown; add async runtime only if needed.
- **CS-10** Domain panel is Drape in code/UI/docs; explicit Rust imports, no old C# namespace-alias rule.

## Rust policy

- **CS-11** Pin stable toolchain/application lockfile; Clippy denies warnings; supported feature matrix, not unconditional --all-features.
- **CS-12** Structs/enums, only needed derives; validate constructors/transitions. Prefer borrowing over needless cloning.
- **CS-13** No mutable globals/hand-rolled singletons; explicit ownership/injection/synchronisation; avoid reference cycles.
- **CS-14** Typed Result for failures; no swallowed errors; context and recovery decision required.
- **CS-15** No unwrap/expect/panic on external data/environment failures. Tests may use them for setup failures.
- **CS-16** Never block Slint loop on I/O/worker/lock; no held locks across callbacks or suspension.

## Architecture

- **CS-17** drape-core has no UI/native OS/network/filesystem/process/environment or wall-clock access. Inject FileSystem/Clock. Core tests first.
- **CS-18** Native calls/unsafe only in matching shell adapters; Guard uses minimal recovery API. Prefer windows bindings over manual extern declarations.
- **CS-19** UI never imports shell types; App connects ports/window lifetimes.
- **CS-20** Tests inspect actual Cargo edges/shared-source boundaries and include a forbidden-dependency mutation check. Core cannot depend on Slint/native crates; UI/Infrastructure cannot depend on shell crates or call native bindings directly. Approved Slint/neutral I/O dependencies may have transitive platform internals; document those edges rather than banning every transitive native dependency. App injects platform-specific FileSystem implementations from Shell into Infrastructure through Core ports.
- **CS-21** Shared Core/UI use PlatformCapabilities, never cfg(target_os)/OS checks. Selection only in adapters/composition roots.
- **CS-22** Neutral owned IconBitmap, never slint::Image, native handles or borrowed OS buffers in Core models.

## Native interop

- **CS-23** Correct release API per owned handle/PIDL/COM allocation via RAII. Distinguish owned/borrowed lifetimes; never close Slint-owned HWND; respect COM apartments/thread affinity.
- **CS-24** HRESULT/Win32 failures become contextual typed errors. Distinguish zero-success returns correctly.
- **CS-25** Slow native calls: bounded apartment-correct workers, explicit timeout/cancellation; never unsafe thread termination. Cancellation does not mean native execution has stopped: retain resources until completion and reject stale generation-tagged responses. Port contracts state thread/apartment/lifetime requirements; do not require Send + Sync of thread-affine services merely for executor convenience.
- **CS-26** Correct FFI ABI/lifetime/state registration; never unwind across FFI; explicit queue-full/error behaviour. Core/shared UI forbid unsafe; native unsafe blocks need SAFETY comments on pointer/lifetime/thread invariants.
- **CS-27** Undocumented techniques tagged UNDOCUMENTED with ADR/spec and verified Windows builds.
- **CS-28** Hook callbacks: no I/O/logging/significant allocation/blocking; bounded preallocated enqueue, no input history.

## UI (Slint)

- **CS-29** Compiled .slint components, Rust adapters, models/callbacks; domain state stays in Core/services.
- **CS-30** No business rules in markup/callbacks; native plumbing behind ports/App.
- **CS-31** Shared Slint theme components/tokens for colours/sizes/states; no XAML/Avalonia API assumptions.
- **CS-32** Accessible roles/labels/states/actions via supported properties; verify Windows UIA/Narrator.
- **CS-33** slint::invoke_from_event_loop or verified equivalent; weak UI references; handle closure/failure. Component/model/image interaction on UI thread.
- **CS-34** One Slint window per Drape; detach/unregister borrowed handles before destruction.

## Errors, logging and performance

- **CS-35** Validate config/drop/path boundaries and resource limits before allocation; missing files/icons degrade safely; invariant failures explicit. Preserve literal/lossless paths and roots; per-filesystem comparison/case rules live in platform adapters. Never expand environment-variable syntax inside an already stored/enumerated path or silently drop unknown config values.
- **CS-36** Structured tracing fields; no eagerly formatted hot-path logs.
- **CS-37** Paths only at Debug/opt-in sharing; no filenames in normal diagnostics.
- **CS-38** Drop/panic hooks cannot replace Guard/durable state. Guard readiness + durable intent precede hiding. Restore cannot depend on config save; abort/kill/reboot need external/next-start recovery. App/Guard/restore CLI serialize ownership with authenticated generation-scoped IPC and bounded parsing; validate live session ownership before startup restoration. Guard loss causes restoration and disables native-icon mutation until rearmed; readiness-to-hide races and corrupt/unsupported state fail closed against new hiding.
- **CS-39** Bound allocations/queues/caches/workers; avoid needless idle redraw.
- **CS-40** Virtualise grids above 200 items; verify support and measure.
- **CS-41** Decode/cache owned pixels off-thread, image conversion on UI thread, cache byte limit.
- **CS-42** Measure Release App + Guard RAM/CPU/GPU/startup/distribution size, backend/build/DPI/monitors/cache state recorded.

## Tests, git and documentation

- **CS-43** Descriptive snake_case tests, e.g. restore_preserves_user_hidden_icons.
- **CS-44** One behaviour, meaningful negative/error branches, hand-written migration fixtures.
- **CS-45** No sleep-based correctness tests; fake clocks and explicit synchronisation. Benchmarks separate.
- **CS-46** Follow testing strategy; name tests, never reuse C# results as Rust evidence.
- **CS-47** One task/branch/PR, {type}/{task-id}-{slug}; fresh Rust foundation/core R0/R1 IDs.
- **CS-48** Conventional Commits with scope/task ID, English, explain why.
- **CS-49** PR: spec refs, validation, risks, waiver author if any, anything not determined. Owner-requested docs migrations can exceed usual small-PR guideline.
- **CS-50** Squash merge; preserve historical records/IDs; no incidental source retirement.
- **CS-51** No credentials/personal paths/user configs; secret scanning.
- **CS-52** Comments explain why and safety invariants.
- **CS-53** Public Rust APIs use rustdoc, error/safety contracts and useful doc tests.
- **CS-54** Windows-version behaviour names verified build/DPI/monitors.
- **CS-55** Review workspace dependencies/features; commit Cargo.lock; audit licences/advisories. Slint royalty-free licence is an explicit documented exception to MIT/Apache/BSD-only dependencies, with attribution.
- **CS-56** Prefer standard library; new runtime/backend justifies resource and distribution cost.

## Generated Slint exception (ADR-028)

Core and hand-written shared UI forbid unsafe. Slint 1.13.1 generated glue requires unsafe internals: drape-ui denies unsafe at its root, allows it only inside its private compiler-generated module, and forbids it in presentation adapters. See [ADR-028](../../MEMORY/DECISIONS.md). This does not permit hand-written unsafe or native bindings in shared UI.
