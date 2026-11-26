# R1-02: lossless schema-1 persistence service

Date: 2026-11-26. Integrator: Claude Code session as core-engineer. Authors: parallel `codex exec` agents with one owned file each. Status: partial REVIEW. Not committed; no merge or DONE is implied.

## Authority

The owner asked for R1-02 to start, then asked for the work to be split into small Codex tasks because the provider limits turns per session. [ADR-031](../DECISIONS.md) records the narrowed dependency.

## Change

| Area | Content |
|---|---|
| `drape-infrastructure/src/config/` | bounded pre-parse scan, RFC 3339 handling, classification and projection, identity-based merge, `JsonConfigStore` over the `FileSystem` port |
| `drape-core/src/ports.rs` | new `PathKeys` port |
| `drape-test-support` | `FakePathKeys`, `FakeFileSystem::seed_backup` and `cancel_on_acquire` |
| tests | 34 tests written before the modules plus review regressions, two hand-written fixtures, scanner and time unit tests |
| dependencies | `serde_json` 1.0.154 with `preserve_order` and `arbitrary_precision`; boundary rule and a mutation test for the development-only fake |

The same batch fixed cross-review findings 1 (icon doc comment) and 4 (middle, X-button and wheel messages now break a click sequence), and added an injected probe with queue, shutdown and panic tests to the icon worker. The worker no longer polls: `load` blocks its caller and `Cancellation` is not `Sync`, so cancellation during a call could never be observed. The per-request cancelled flag is now set only by tests and can be removed later.

## Reviews and defects found

| Source | Finding | Outcome |
|---|---|---|
| Design review | reserved parser key silently rewrites data; pruning could delete all valid backups; sub-millisecond membership order; unsafe path keys; duplicates inside a Drape; ranges treated as corrupt | spec changed before code |
| First merged build | `i64::is_multiple_of` unavailable on 1.90 | fixed |
| Tests | integer fields written as `0.0`, so new documents were refused by the re-read check | fixed |
| Integrator reading | an unchanged `missingSince: null` was removed on save | fixed with a test |
| Code review | duplicate property hid truncated JSON; differently cased and conflicting values projected instead of defaults; unreadable listed backup skipped; cancellation after writer acquisition still quarantined | five regressions failed first, then fixed |
| Integrator review | the icon worker drop test could not fail | bounded wait added |

Declined, with the specification clarified: an absent name or title stays read-only; a truncated document containing a reserved parser name stays read-only (bytes are preserved either way); a required property spelled with different case projects its stored value.

## Evidence

Offline on a fresh copy of the repository code after applying it, Rust 1.90.0.

- Format check and denied-warning Clippy pass; locked Release build passes.
- 181 workspace tests pass; 8 boundary tests pass.
- `cargo deny` advisories (local database), licences and sources pass.
- Line coverage for Core and Infrastructure together 91.46%; each `config/` module is above 95% of lines.
- Eight mutations each failed a named test: re-read verification removed; pruning without confirmed durability; temporary cleanup skipped; corrupt backups always removed; cancellation check before quarantine removed; newer schema treated as corrupt; unsupported treated as writable; worker drop without join.

## Not done and not determined

- No native Windows filesystem adapter, `PathKeys` or `GuidSource` implementation exists, so no real file has been read or written.
- App does not construct the store; debounce and shutdown flush belong to R1-05.
- Durability, replacement semantics, Windows 10 and 11 behaviour and power loss are untested.
- No compatibility run against files produced by the legacy application; fixtures are hand-written from its writer.
- Memory use for documents near the limits was not measured.

## Handoff core-engineer to architect and shell-engineer-windows, 2026-11-26

- Context: persistence logic is complete against the fake.
- Done: service, tests, two review rounds.
- Not done: everything listed above.
- Contract/interface changed: new `drape_core::ports::PathKeys`; `ConfigStore` unchanged.
- How to verify: `scripts/cargo-windows.cmd test --workspace --locked`.
- Risks / surprises: App must pass one `PathKeys` provider to enumeration, drop and the store, or saves are refused as unsupported.
