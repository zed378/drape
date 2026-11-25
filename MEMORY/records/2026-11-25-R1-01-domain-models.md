# R1-01 — Validated domain foundation and read-only adapter contracts

Date: 2026-11-25. Owner: core-engineer. Status: REVIEW (partial); architect-approved subset, targeted QA green, full acceptance incomplete.

## Authority and scope

The owner requested parallel implementation with exactly one testing agent and a sequential testing queue. The orchestrator assigned R1-01 alongside R0-03. Architect approved this std-only subset before implementation and reviewed its identity/path/read-only boundaries. No new dependencies, Cargo manifest changes, legacy source edits, native operations, random IDs or environmental time were introduced. Existing PlatformCapabilities, TimestampMillis and Clock remain compatible.

Spec refs: docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md, 02-DATA-MODEL.md, 03-RUST-MIGRATION.md; docs/ENGINEERING/00-CODING-STANDARDS.md; ADR-025–028.

## Implemented contracts

- `crates/drape-core/src/domain.rs`: non-nil Guid (hyphenated syntax, case-insensitive hex, no UUID-version restriction), opaque adapter-supplied PathKey, Bounds, Page, DrapeItem, Drape, DomainConfig, style/state/settings enums and data, explicit DomainError/Repair, and owned IconBitmap.
- Stored literal paths use OsString/OsStr without environment expansion, lowercase/Unicode normalization, canonicalization or metadata lookup. PathKey comparison is exact and the platform adapter selects filesystem-specific semantics. File IDs are opaque observation data, never sufficient identity for a merge.
- Numeric constructors reject non-finite values, widths below 120 DIP, heights below 80 DIP, nonpositive DPI, fractions outside 0..1, and invalid style ranges. Bounds remain stored logical coordinates; monitor clamping/rescaling and work-area accessibility remain R1-03.
- Strict DomainConfig validation rejects duplicate page/Drape/item IDs, unknown page references, empty pages and duplicate membership. Explicit schema-1 reconciliation requires caller-provided valid fallback page when needed, reports repairs, retains newest supplied addedAt membership and uses persisted Drape vector order then GUID for exact ties. A repeated item GUID with different opaque keys is corrupt identity and is rejected instead of removing an unrelated reference. No timestamps/IDs are synthesized. Caller retains original config/backup and reports repairs.
- Future schema configs are read-only through every transition: add/replace/remove Drape and settings update return ReadOnly. `reconcile` takes explicitly schema-1 domain arrays, not an arbitrary-version config document. DomainConfig is not a lossless wire document; R1-02 must retain unknown raw fields/discriminators and use explicit wire names before any round-trip claim.
- IconBitmap owns top-down straight-alpha RGBA8/BGRA8 bytes, permits arbitrary byte row padding, validates dimensions/stride/buffer length, uses checked arithmetic and accepts a caller byte budget. `required_bytes` lets adapters validate before decode allocation. No UI image/native handle enters Core.
- `crates/drape-core/src/ports.rs`: synchronous DesktopLocations, DesktopItemSource and IconProvider, plus owned entry/request/response values, OperationId, cooperative Cancellation and redacted PortError. Port rustdoc defines caller-selected thread/apartment, capability rejection, limits, ownership, cancellation/shutdown and stale-response rejection by consumers. Cancellation is same-thread cooperative data, not proof native execution has stopped. No blanket Send+Sync bound or hidden executor exists. OperationId is not an authenticated recovery ownership token.
- `crates/drape-test-support/src/lib.rs`: finite scripted FakeDesktopLocations/FakeDesktopItemSource/FakeIconProvider with owned data, explicit errors, limits/cancellation and exhausted-script failure; FixedClock retained unchanged.

## Legacy parity matrix

| Legacy source/behaviour | Rust implementation and named test |
|---|---|
| Model/Page.cs, Drape.cs non-empty identity/title | Guid, Page, Drape; `guid_rejects_nil_malformed_and_noncanonical_identity`, `panel_edits_validate_and_only_remove_references` |
| Model/DrapeBounds.cs minimum dimensions/DPI/fractions | Bounds; `bounds_reject_all_nonfinite_values_and_invalid_dimensions` additionally rejects non-finite coordinates/sizes/DPI |
| Model/DrapeItem.cs path, missing observation and reference removal | DrapeItem; `literal_paths_and_adapter_comparison_keys_are_preserved`, `missing_observations_preserve_original_time_until_present`; adapter-supplied comparison replaces legacy unconditional case-insensitive comparison per current spec |
| Model/DrapeConfig.cs page/ID/membership invariants | DomainConfig; `config_rejects_empty_pages_duplicate_ids_unknown_pages_and_membership`, `config_edits_preserve_settings_and_validate_replacement` |
| Data-model reconciliation contract (not claimed legacy parity) | `membership_repair_uses_newest_timestamp_then_stable_panel_order`, `missing_page_repair_requires_caller_supplied_default_and_reports_changes`, `conflicting_identity_is_rejected_instead_of_removing_unrelated_reference` |
| Model/DrapeStyle.cs ranges/defaults; Settings offline defaults | `styles_reject_nonfinite_render_values`, `newer_schema_is_read_only_and_defaults_are_offline` |
| Neutral bitmap boundary replacing UI/native objects | `bitmap_rejects_invalid_stride_overflow_and_resource_limit` |
| New deterministic port fakes | `locations_are_scripted_owned_and_never_read_real_desktop`, `snapshots_enforce_limit_cancellation_and_preserve_generation`, `icon_requests_report_scripted_errors_and_cancel_without_consuming_script` |

## Evidence

All executable validation is owned by the single QA agent; implementation agents do not run tests/builds/format/lint/coverage. Tests were authored before logic.

- QA red: `scripts/cargo-windows.cmd test -p drape-core --test domain --locked` exited 1 with E0432 unresolved `drape_core::domain`, before domain implementation.
- QA red: `scripts/cargo-windows.cmd test -p drape-test-support --test ports --locked` exited 1 with E0583 missing `ports` module file, before port/fake implementation.
- QA green: locked targeted Core/test-support run passed 18 tests (13 domain, 3 port/fake, 2 baseline). Includes conflicting identity rejection, future-schema read-only, invalid bounds, duplicate membership, cancellation and operation generation. Final workspace gates, actual coverage and security scans remain pending.

## Remaining R1-01 acceptance / not determined

R1-01 is partial: FileSystem/ConfigStore, host/recovery/Guard, hooks/context-menu, display, autostart/blur and other native operation contracts/fakes remain unimplemented pending the relevant spike/spec/implementer reviews. Rule/Snapshot and later portal/tab/pinning semantics are not invented here; Q-23 remains open. No R1-02 wire parsing/schema serialization, config durability/migration, R1-03 layout, service/UI integration, native behaviour, coverage threshold, performance or Windows matrix acceptance is claimed. No commit, merge or DONE status is implied.

### Handoff core-engineer → qa / architect · 2026-11-25

- Context: owner authorised parallel implementation with one sequential testing agent.
- Done: approved std-only domain/three read-only ports plus deterministic fakes and tests-first definitions.
- Not done: remaining R1-01 ports and acceptance above, green verification and independent review.
- Contract/interface changed: additive domain and ports modules; existing baseline interfaces retained.
- How to verify: QA runs locked Core/test-support tests, format, Clippy, workspace/boundary gates sequentially; report named test failures to owner agent.
- Risks / surprises: legacy path comparison and some numeric validation conflict with current spec; Rust follows reviewed spec rather than reproducing unsafe assumptions. Lossless wire preservation is separate work.
