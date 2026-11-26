# Cross-review of the uncommitted R0/R1 continuation

Date: 2026-11-26. Reviewer: Claude Code session, different-role review requested by the owner. Subject: working tree on `feat/R0-06-enumeration-contracts` over `24f2433`, `crates/` as copied at 14:26 local time. Status: findings recorded, none fixed here. No DONE, commit or merge is implied.

## Gates re-run on an isolated copy

The copy and its target directory live outside the repository, built offline with the repository-local Rust 1.90.0.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | pass |
| `cargo test --workspace --locked --offline` | 110 passed, 0 failed |
| `cargo build --workspace --release --locked --offline` | pass |
| `python scripts/check-rust-boundaries.py` | 7 passed |
| `cargo llvm-cov -p drape-core --summary-only` | 89.27% lines (807/904) |

The latest [follow-up record](./2026-11-26-parallel-r0-06-r0-08-follow-up.md) reports 107 tests and 5 `hook_callback` tests. `icon_worker.rs`, `handle_with` and `message_read_panic_latches_a_reset_and_still_forwards` were added later and had no record at snapshot time.

Mutation checks reported by earlier records were not repeated. Offline dependency and secret scans were not run.

## Findings

### 1. Memory-only icon requests can return a generic icon (R0-06)

`shell_items.rs` calls `GetImage` with `SIIGBF_MEMORYONLY`. Microsoft documents that this flag [can fall back to a per-class icon](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitemimagefactory-getimage) when a per-instance icon is not cached. The doc comment on `probe_shell_icon` says a cache miss returns an error. The response cannot distinguish the two cases and no extraction path exists, so a caller could cache a class icon as the item's icon. Correct the comment and define a second, off-UI extraction step with its own hydration guard.

### 2. Icon worker shutdown can stall (R0-06)

`ShellIconWorker::drop` sends `Shutdown` with a blocking send and joins. A shell call that does not return keeps drop waiting. The owner of this worker must never sit on a path that precedes desktop restoration. `load` also blocks its caller and polls every 10 ms, so the queue bound matters only with that many caller threads. The two tests cover constructor bounds and a pre-cancelled request; full queue, in-flight cancellation and shutdown have no test and the probe call has no injection seam. The apartment is single-threaded and the thread pumps no messages; confirm that against the COM contract.

### 3. One unusual desktop root fails the whole scan (R0-06, not verified natively)

`probe_desktop_metadata` returns `Unavailable` when either root carries a reparse, offline or recall attribute. A redirected (OneDrive or junction) Desktop may carry one, which would also discard the public desktop entries. Both roots on the review machine carry none of these flags, so this needs a VM case.

### 4. Gesture sequence ignores some buttons (R0-08)

`HookCallback::admit` drops middle, X-button and wheel messages without marking a discontinuity, so left click, middle click, left click can complete a double-click. Swapped primary buttons (`SM_SWAPBUTTON`) appear in neither code nor spec. Which button values `WH_MOUSE_LL` reports under swapping was not established here and needs a native test.

### 5. Task state disagrees across files

- `RUST-MIGRATION.md` marks R1-01 WIP while the board lists R0-05, R0-06 and R0-08 as WIP: four claimed cards against a limit of three.
- The R1-01 card reports a persistence consumer slice in progress; `drape-infrastructure` contains no persistence code.
- R0-05 is claimed by the orchestrator; the card role is shell-engineer-windows.
- The follow-up record calls R0-06 and R0-08 partial REVIEW; the board lists them WIP. The board header still says updated 2026-11-25.
- The R1-02 spec lists no open question; the backlog lists alias and input-limit prerequisites for it.

### 6. Changelog tail

`MEMORY/CHANGELOG.md` ends with the literal text `\r\n`.

### 7. Uncommitted volume

Work for R0-05, R0-06, R0-08, R1-01 and R1-02 sits uncommitted in one tree (36 changed or new project files, plus one `.agents` rule file). Committing needs the owner's approval; until then a stray reset or clean loses it. `.tmp-*` scripts and `.tmp-rust-qa-secrets/` remain at the repository root (ignored by Git).

## Not determined

No native API, hook, GUI or desktop operation was executed by this review. Windows 10 22H2 and Windows 11 behaviour, DPI and monitor matrix, hydration, latency and resource retention remain open. Native findings above come from source reading and, for finding 1, vendor documentation.
