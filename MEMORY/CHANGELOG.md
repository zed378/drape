# Changelog

Significant changes to the project, newest first. Coarser than [`MEMORY-INDEX.md`](./MEMORY-INDEX.md); every entry links to a record or an ADR.

Format loosely follows Keep a Changelog. Dates are absolute. There is no released version. Early C# code exists; Rust + Slint is the target from ADR-025, with implementation beginning at R0-02.

---

## Unreleased

### 2026-11-26 - R0-05 recovery specification revision ([record](./records/2026-11-26-R0-05-spec-revision.md))

- **Changed:** closed state machine, record validation matrix and restart rules for icon recovery; uninstall wording aligned with [ADR-032](./DECISIONS.md).
- **Open:** five Core review findings, owner questions Q-24 to Q-26 and all native evidence. The spec is not accepted.

### 2026-11-26 - R1-02 persistence service ([record](./records/2026-11-26-R1-02-persistence-service.md))

- **Added:** lossless schema-1 load and save with bounded parsing, quarantine, backup recovery and pruning, verified against the filesystem fake; `PathKeys` port.
- **Fixed:** middle, X-button and wheel messages break a click sequence; the icon doc comment no longer promises an error on a cache miss; the icon worker is testable and stops polling.
- **Pending:** native filesystem adapter, application wiring and Windows evidence.

### 2026-11-26 - Icon worker and deferred callback read ([record](./records/2026-11-26-parallel-r0-06-r0-08-worker-callback.md))

- **Added:** bounded serial shell icon worker returning raw DIB data; callback data read only after the hook code check. Native integration and Windows VM evidence remain pending.

### 2026-11-26 - R1-01 configuration store contract ([record](./records/2026-11-26-R1-01-config-store-contract.md))

- **Added:** platform-free `ConfigStore` port, a shared save gate that refuses stale, read-only and indeterminate saves, and a scripted fake.
- **Reviewed:** different-role architect review; open findings on other slices are in the [cross-review](./records/2026-11-26-claude-cross-review.md). No native store or Windows behaviour is claimed.

### 2026-11-26 - Bounded shell adapter QA checkpoint ([record](./records/2026-11-26-parallel-r0-06-r0-08-follow-up.md))

- R0-06 metadata-to-snapshot and R0-08 callback-policy slices pass focused and workspace QA. The callback reset-order test now observes the enable Reset before a later sample; both tasks remain partial until native integration and Windows VM evidence.

### 2026-11-26 - Metadata budget and pure consumer follow-up ([record](./records/2026-11-26-parallel-r0-06-r0-08-follow-up.md))
- **Fixed:** every observed metadata entry consumes scan budget before filename validation; pre-context hook samples are discarded before recognition.
- **Added:** explicit serialized consumer cancellation/reset and final enabled/generation revalidation coverage.
- **Verified:** independent QA, 99 workspace tests, three behavioral guard mutations, locked Release, formatting and denied-warning Clippy. Native performance, hook installation and Windows matrix remain pending.

### 2026-11-26 - Parallel continuation validation ([record](./records/2026-11-25-autonomous-continuation.md))
- **Verified:** filesystem fake and pure hook-state subsets; Release, format, Clippy, 88 tests, seven architecture checks, 89.27% Core coverage and offline dependency audit pass.
- **Clarified:** concurrent discontinuity ordering and consumer stale-intent checks; native hook and VM evidence remain pending.

### 2026-11-25 — Passive enumeration and persistence contracts ([record](./records/2026-11-25-enumeration-persistence-contracts.md))
- **Added:** bounded read-only Windows desktop metadata enumeration and platform-neutral root-confined filesystem contract declarations.
- **Pending:** sequential QA results, native enumeration matrix, comparison-key integration, lossless config parser/service and native filesystem adapter.

### 2026-11-25 — Icon cache and snapshot continuation ([record](./records/2026-11-25-icon-cache-validation-continuation.md))
- **Added:** worker-owned icon LRU cache with pixel-byte and entry-count limits, size/revision keys, atomic oversize rejection and item invalidation.
- **Reviewed:** immutable snapshot domain values and actual legacy JSON spellings/timestamp precision as persistence-fixture prerequisites.
- **Pending:** final sequential QA results in the record; native enumeration, hydration/alpha, Windows matrix and runtime cache integration remain unverified.

### 2026-11-25 — Parallel Rust layout and host diagnostics ([record](./records/2026-11-25-parallel-layout-host-continuation.md))
- **Added:** pure monitor-relative layout, snap and recovery development over the reviewed domain subset, plus an isolated safe Slint native-handle diagnostic.
- **Verified locally:** locked Release build, formatting, denied-warning Clippy, 38 workspace tests, 86.75% Core coverage, a real no-drift mutation check and current advisory/licence/source audit; sole QA records named evidence.
- **Probe verified locally:** isolated locked/offline Release, format, Clippy, two CLI tests and compiled help/rejected-mutation arguments; no GUI launch.
- **Pending:** real Windows desktop hosting/recovery, persisted-layout integration, interactive matrix and merge.

### 2026-11-25 — Rust workspace baseline ([record](./records/2026-11-25-R0-02-rust-workspace.md))
- **Added:** seven-crate workspace, pinned Rust/Slint and lockfile, compiled baseline UI with attribution, minimal contracts and renderer-independent CLI routing.
- **Verified:** actual Cargo boundaries and forbidden dev/build/optional dependency mutations; final build/test evidence is recorded in the task record.
- **Fixed:** Visual Studio component discovery and Slint-generated unsafe isolation (ADR-028).
- **Unavailable:** native desktop integration and recovery; Guard cannot announce readiness. No legacy source retirement or release claim.

### 2026-11-25 — Multi-specialist documentation review ([record](./records/2026-11-25-R0-01-specialist-documentation-review.md))
- **Reviewed:** architecture/engineering, Windows/security and QA/release/UI specialists with disjoint ownership and final cross-document passes.
- **Fixed (ADR-026):** recovery ownership/liveness/IPC, native callback/handle/COM contracts, last-committed durability, keyboard-only entry, destructive shell verbs, full test matrix, Rust release/update acceptance and future-platform recovery gates.
- **Added:** Q-23 feature representation/specification prerequisite; review evidence and whole-set checks.
- **Removed:** irrelevant plugin dashboard/savings/footer and unavailable-command mandates, inaccurate active support/durability claims and leftover .NET tooling assumptions. Historical evidence retained.
- **Not implemented:** no source/build/CI change, application test, runtime safety result, commit/PR or merge.

### 2026-11-25 — Rust + Slint target, documentation only ([record](./records/2026-11-25-R0-01-rust-slint-documentation.md))
- **Decided (ADR-025):** owner selects Rust + Slint, no WebView/browser engine; RAM/performance/memory safety/distribution size priorities.
- **Changed:** active architecture/standards/testing/release/UI/platform docs, roadmap/task dependencies and agent guidance; fresh R0/R1 replacement tasks; legacy C# cards/specs/runbooks/baselines labelled historical.
- **Clarified:** Guard readiness and durable intent before hiding; restore independent of config/log success; Slint licence/attribution separate from Drape MIT; schema 1 compatibility preserved.
- **Not implemented/tested:** no Rust workspace, source port/removal, build/config/CI changes or application tests. Q-19–Q-22 and independent documentation review remain open.

### 2026-10-10 — Windows 10 22H2 is the minimum supported OS ([record](./records/2026-10-10-windows-10-minimum-and-platform-minimums.md))
- **Decided (ADR-023):** owner: Windows 10 22H2 (build 19045) is the minimum and fully supported; Windows 11 is the primary dev target. Assistant: macOS 14, Ubuntu 22.04/Debian 12, Android 12, iOS 18, with a distribution channel per platform.
- **Added:** risk PR-9, backlog B-10/B-11, a minimum-OS-and-distribution table in the platform document.
- **Changed:** Windows 10 is now a release gate, not best effort.

### 2026-10-10 — Every remaining open question answered; README and LICENSE added ([record](./records/2026-10-10-remaining-questions-readme-licence.md))
- **Decided (ADR-022):** direct distribution (GitHub Releases + Velopack + winget), Windows 10 best effort until 12 Oct 2027, x64 only in v1.0, Guard required from the MVP, update checks off by default, id-ID/en-US only, GitHub Actions, one Drape per item, no virtual items, own look.
- **Added:** root `README.md`, `LICENSE` (MIT).
- **Changed:** mobile minimums now follow Avalonia support tiers (Android 12+, iOS 18+).
- **Not verified:** research from web pages only; nothing run.

### 2026-10-10 — Owner decisions: MIT, final name, Windows first, `Fence` renamed to `Drape` ([record](./records/2026-10-10-owner-decisions-licence-name-rename.md))
- **Decided:** MIT licence, free (ADR-019); name "Drape" final but not trademark-searched; Windows first (ADR-014 confirmed); mobile is a companion shortcut-group app, last in the queue (ADR-021).
- **Changed:** every `Fence*` identifier is now `Drape*` (ADR-020, supersedes ADR-016); naming rule CS-10 handles the namespace clash.
- **Closed:** Q-02, Q-03, Q-15, Q-16, Q-17.
- **Not verified:** documents only; nothing compiled.

### 2026-10-10 — Documentation restructured and translated to English ([record](./records/2026-10-10-docs-restructure-and-translation.md))
- **Changed:** all documentation moved from Indonesian to English (ADR-013) and into the layout of the reference repository (ADR-018): `docs/` by category, ADRs in `MEMORY/DECISIONS.md`, `TASKS/` as `PROGRESS` + `BACKLOG` + `PHASE-n`, task IDs `Pn-nn`, `AGENT.md` renamed `AGENTS.md`.
- **Changed:** the stack is recorded as C# / .NET 10 / Avalonia, cross-platform (ADR-012); v1.0 is Windows only and other platforms are Phase 6 (ADR-014).
- **Added:** `docs/PLATFORM/01-PLATFORM-SUPPORT.md`, Phase 6, task P2-13, ADR-012 to ADR-018, open questions Q-15 to Q-18.
- **Fixed (documents only):** several contradictions and layer violations, listed in the record. The name rationale for "Drape" was corrected (ADR-001).
- **Removed:** `docs/adr/`, `TASKS/BOARD.md`, `TASKS/DONE.md`, the M0-M5 task files and the old Indonesian `MEMORY/` files (content merged).
- **Not verified:** nothing was built or run, and no platform claim was tested.

### 2026-10-09 — Planning phase begins ([record](./records/2026-10-09-project-setup.md))
- **Added:** project structure (`docs/`, `MEMORY/`, `TASKS/`, `CLAUDE.md`, agent file), the first set of documents (in Indonesian), five ADR files, task files M0-M5, decisions D-001 to D-011 (now ADR-001 to ADR-011) and open questions Q-01 to Q-14.
- No code, no repository history, no solution.

## 2026-11-25 — Parallel Rust continuation (local work)

- R0-03 adds sequential Windows Rust CI, negative quality-gate probes, coverage/security policy and pinned helper tools; cargo-deny compatibility with current CVSS 4.0 advisory data requires the 0.21.0 helper update.
- R1-01 adds validated platform-free domain data, explicit reference-only repairs, bounded owned icon pixels, neutral synchronous enumeration/icon ports and deterministic fakes. Native/recovery/filesystem contracts and full parity remain deferred acceptance.
- ADR-029 records the owner-requested parallel implementation and exclusive sequential QA execution policy. See the parallel continuation and QA records for verification and remaining limits; no merge, DONE or native recovery claim.
