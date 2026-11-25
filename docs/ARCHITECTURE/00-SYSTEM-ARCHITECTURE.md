# 00 — System Architecture

Target: Rust + Slint, owner decision [ADR-025](../../MEMORY/DECISIONS.md). This is a design, not implemented Rust behaviour. Existing C# remains untouched; see [migration boundaries](./03-RUST-MIGRATION.md).

## Layers and ownership

R0-02 now provides a scaffold for these layers, not their production behaviour. See the [implementation record](../../MEMORY/records/2026-11-25-R0-02-rust-workspace.md). ADR-028 permits only Slint compiler-generated glue inside a private UI module to use unsafe internals; hand-written UI and Core remain forbidden from using unsafe.

```text
drape-app (composition root, CLI, lifecycle, single instance, tray)
    -> drape-ui             -> drape-core
    -> drape-infrastructure -> drape-core
    -> drape-shell-windows  -> drape-core
    -> drape-core
drape-guard -> drape-shell-windows (minimal recovery features), drape-core
```

Core has no Slint, native OS, filesystem, network, process or wall-clock dependency. Infrastructure implements I/O ports. UI consumes domain data/ports and never imports a shell crate. App connects UI window lifetimes to native shell attachment through opaque IDs. Platform cfg selection belongs only in adapters/composition roots. Architecture tests inspect real Cargo dependencies and shared-source boundaries. Core is UI/OS/I/O-free; shared UI has no shell imports/direct native bindings. Approved Slint/neutral filesystem watcher internals may have transitive platform dependencies and must not be confused with direct UI/Core platform calls.

## Components

| Component | Owner | Responsibility |
|---|---|---|
| Drape, DrapeItem, Page, Rule, Snapshot | Core | Owned structs/enums, stable config semantics |
| DrapeManager, RuleEngine, LayoutService | Core | Pure state transitions, rules, stored-position math |
| PlatformCapabilities | Core data / Shell provider | Host, icon hiding, hook, blur, hotkeys; capability gating |
| ConfigStore, Clock | Core ports / Infrastructure adapters | Persistence coordination and time outside Core |
| FileSystem | Core port / platform-selected adapter | Neutral filesystem operations, platform path/identity rules and durable atomic replacement; native implementation belongs to the matching shell adapter |
| DesktopLocations | Core port / Shell | Known Folder Desktop/Public Desktop, including redirection |
| DesktopItemSource | Core port / Infrastructure | Enumeration, watcher, bounded reconciliation |
| DesktopHost | Core port / Shell | Attach/place/detach/reattach and degraded mode |
| DesktopIcons | Core port / Shell | Read/change/verify icon state and recovery ownership |
| IconProvider | Core port / Shell | Neutral owned IconBitmap pixels, bounded cache |
| InputHook, ShellContextMenu, DisplayMonitor | Core ports / Shell | Native events, apartment-aware COM, DPI/monitor data |
| Autostart, BlurEffect | Core ports / Shell | Capability-gated run-at-login and effects/fallback |
| DrapeWindow and presentation adapter | UI | One Slint window per Drape; models/properties/callbacks |
| SettingsWindow and themes | UI | Accessible light/dark/high-contrast presentation |
| tray and application services | App | Command routing and composition |
| independent Guard | Guard | Restore owned icon changes after App termination |

Rust traits omit the C# I prefix. IconBitmap has owned pixels, validated dimensions/stride/format/buffer length; Core never exposes slint::Image or HWND. Data-model fields remain compatible with existing JSON.

Infrastructure owns persistence coordination, wire-format parsing/serialisation, backups, watcher batching and conversion to domain data. It receives FileSystem and DesktopLocations from App; it never imports Shell to construct them. An adapter that needs Win32 replacement, file IDs or other direct platform APIs lives in drape-shell-windows and implements the neutral Core port. Standard-library/approved neutral dependencies may wrap native implementations internally; this does not authorise direct Win32 calls in Infrastructure. Domain migrations operate on platform-free document data; selecting a JSON library must not move parsing, I/O or platform policy into Core.

## Native window boundary

UI owns each Slint window. A private App integration adapter obtains its native handle through the selected backend's supported public API and registers an opaque window ID with Shell. Shell borrows the handle: never destroy a UI-owned window or retain the handle beyond registration. Detach/unregister before destruction and reacquire after recreation. R0-04/R0-07 establish actual API, threading and renderer requirements; Gemini sample code is not proof.

One window per Drape remains the ADR-007 constraint. Host parenting, transparency and explicit keyboard focus require Rust spikes. A bottom-most top-level fallback is degraded if Win+D or other criteria fail. Failed discovery never guesses Progman is safe. An unsafe/unavailable host leaves native icons visible. Required keyboard access enters through tray/settings commands and explicit focus activation; it cannot depend on a later optional global-hotkey feature.

## Startup

1. Parse CLI before renderer initialisation. --restore-desktop selects the standalone recovery route, which uses the ownership coordination in step 2 and then exits without loading Slint/UI; it does not bypass live-owner validation.
2. Acquire per-user/session ownership coordination before evaluating or changing recovery state. Establish single-instance ownership and validate any live App/Guard generation using OS identity/signalling; a second instance signals the owner and exits without restoring its live session's icons. Recover unfinished state only when no validated owner retains responsibility. Standalone restore and Guard use the same coordination protocol; R0-05 specifies the handshake and forced-recovery policy.
3. Load/migrate/validate config; newer schema is read-only. Read capabilities and disable unsupported actions.
4. Discover host dynamically; enumerate Desktop/Public Desktop and reconcile membership.
5. Create/attach/show windows and verify host readiness without stealing focus.
6. Before hiding icons: read original state, durably record intent/ownership, start Guard and authenticate its readiness/generation, then revalidate liveness and hide/verify under coordinated ownership. Already-hidden icons are not claimed as Drape's change. Guard failure/unsafe host keeps icons visible; failed readiness/liveness validation must never authorize mutation.
7. Start tray, display notifications, watcher and optional input hook. No network by default.

## Shutdown and exceptional paths

A failed config save must never prevent restoration. Stop hooks/watchers/work, restore and verify owned icon changes, persist recovery success, and disarm Guard only after success; flush config, detach windows and release resources. Failed restoration retains retryable state and Guard responsibility.

Expected failures return typed Result. Panic recovery is best effort: hooks must not block or recursively panic. Drop does not run for all termination paths. Never unwind across native callbacks. Guard handles abort/forced kill; durable next-start recovery and standalone CLI cover reboot/both processes dying. Guard identifies the intended process by OS identity/handle semantics, not PID alone.

App monitors Guard after readiness. If Guard dies or its channel becomes invalid while App remains alive, immediately restore owned icon changes, suspend every native-icon mutation (including quick-hide), and do not hide again until a new Guard is authenticated and armed. R0-05 fault-injects Guard loss before readiness, between readiness and hide, after hide and during restore. Readiness alone is not lifetime protection; overlapping failures still require durable next-start/CLI recovery.

App/Guard IPC is scoped to the same user/session and authenticated process generation, with bounded message/state parsing and explicit timeout/rejection behaviour. An untrusted stale/corrupt/future record or spoofed readiness/disarm message cannot authorize a visibility mutation. Ambiguous ownership or unreadable state prevents new hiding and surfaces recovery failure/manual restoration instructions; never infer ownership from PID/path text or blindly toggle. Standalone restore asks a validated live owner to restore through this protocol instead of racing its state writes; R0-05 defines safe fallback for an unresponsive owner.

## Event flow and threading

- Slint owns the main event loop. Component/model/image interaction stays there. Workers publish owned data via slint::invoke_from_event_loop or verified equivalent, with weak UI references so closed windows are not kept alive.
- Watcher -> bounded queue -> 250 ms debounce -> reconcile -> UI update. Overflow triggers full rescan.
- Icons: bounded workers with apartment-correct COM, owned pixels returned to UI for image conversion. Bound cache bytes and queued work.
- Hook: dedicated native message loop, preallocated bounded non-blocking enqueue and explicit full-queue handling; no I/O, persistent input history or unwinding across FFI.
- Display changes recompute from stored logical coordinates, not repeatedly scaled output.
- Explorer restart invalidates registrations: rediscover/reattach/verify/reconcile; failure enters safe degraded mode. Restart tests remain manual in a VM.
- OLE drops/context menus require documented COM apartments and owner-window/message routing; shared UI never calls native APIs. No async runtime is mandatory.

## Port and cancellation contracts

R1-01 specifies each port's owner, call thread, input/output ownership, fallible results, cancellation/shutdown semantics and capability precondition. Do not apply Send + Sync to every trait or move HWND/COM/UI state to workers merely to satisfy a generic executor. Apartment/thread-affine services are accessed through a dispatcher with explicit lifetime and request completion. Workers return owned neutral data; stale/cancelled responses carry an operation/generation identifier so they cannot update a replaced window, newer icon request or closed session.

OS-specific item launch, file-manager reveal, context menus, hotkeys and system appearance/animation queries also use capability-gated platform ports; they do not become UI calls to a convenience crate such as open. Port names beyond the existing list are selected with implementer review in R1-01, before two-sided implementation. Cancellation must distinguish a request no longer wanted by UI from native work that is still executing: retain native owners/resources until that work has actually stopped.

## Persistence, errors and observability

[Data model](./02-DATA-MODEL.md) defines durability. Save debounce is 500 ms. Atomic replacement is OS-aware FileSystem behaviour, not an assumption about std::fs::rename on Windows. Recovery state has separate durable ordering.

Native errors become contextual typed results at Shell boundary. Missing files/icons degrade to placeholders/redacted logs. Permission failures, unavailable shell, renderer failure and queue overflow have explicit outcomes.

Structured tracing uses a rolling writer with a flush guard; paths only at Debug, no filenames in normal diagnostics. Log failure cannot block recovery. Diagnostics include version, Windows build, backend/renderer, DPI, monitors, capabilities and host state, without user paths.

## Extension and validation

No plugins in v1; rule condition/action traits remain platform-free. macOS/Linux/mobile companion UI remain Phase 6 with Rust/Slint support revalidated first. R0-04–R0-10 prove host, recovery, rendering, drops, UIA and resource costs before production adoption. C# tests remain historical evidence. See [testing](../TESTING/00-TESTING-STRATEGY.md) and [migration tasks](../../TASKS/RUST-MIGRATION.md).
