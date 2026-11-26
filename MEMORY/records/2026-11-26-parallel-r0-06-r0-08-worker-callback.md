# R0-06 icon worker and R0-08 deferred callback read

Date: 2026-11-26. Status: partial REVIEW. Written by the Claude Code reviewer because the implementing Codex session finished the slice and then reached its turn limit before adding this record; the board already linked to this file.

## Change

- R0-06: `crates/drape-shell-windows/src/icon_worker.rs` adds one serial worker with a request queue bounded from 1 through 32. Each request creates its own COM apartment through the probe. The response carries the operation and item identity with raw bottom-up DIB bytes; channel order and alpha stay unverified, so it is not a Core `IconBitmap`.
- R0-08: `crates/drape-shell-windows/src/hook_callback.rs` adds `handle_with`, which reads the message only after a non-negative hook code and still forwards when the read panics.

## Evidence

The implementing session reported format, Clippy, Release build, 110 workspace tests and 7 boundary tests passing. The [cross-review](./2026-11-26-claude-cross-review.md) reran those gates on a copy of the same sources and obtained the same counts. No tests-first or mutation evidence for this slice was available to the author of this record.

Tests present for the slice: `worker_rejects_unbounded_or_empty_queue_limits`, `pre_cancelled_request_never_reaches_the_native_worker`, `message_read_panic_latches_a_reset_and_still_forwards`.

## Open

Findings 1, 2 and 4 of the cross-review apply to this code and are not fixed. Native hook installation, icon conversion and UI integration, hydration, latency and the Windows 10/11 VM, DPI and monitor matrix are not verified.
