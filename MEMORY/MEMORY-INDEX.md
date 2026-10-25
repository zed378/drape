# Memory Index

Every record and every decision, newest first. Add a line here as part of writing the record or ADR: **an unindexed record is a record nobody finds.**

Current status is in [`../TASKS/PROGRESS.md`](../TASKS/PROGRESS.md), not here.

---

## Records

| Date | Task | Record | Hook |
|---|---|---|---|
| 2026-10-25 | P0-10, P1-12, P1-04-parsial | [Subagents, coverage, ViewModel](./records/2026-10-25-p0-10-p1-12-p1-04.md) | The 20 ms save budget does not hold (~24 ms median) — the test now gates the median at 30 ms while the record states the actual number honestly. |
| 2026-10-25 | P0-01/02/08, P1-03/07 | [Phase 0 + P1 batch: CI, composition root, DrapeService](./records/2026-10-25-phase0-p1-batch.md) | `with { Prop = bad }` bypasses the property initializer — Rename and WithOrder needed explicit validation; the manifest had 3 stacked defects behind one generic error. |
| 2026-10-23 | P1-01/02/03/06/11 | [Core domain, contracts, ConfigStore, layout, persistence](./records/2026-10-23-P1-core-domain-config-layout.md) | `public Drape(...)` does not compile: the type name resolves to the root namespace in a constructor declaration, so the record needed a primary constructor. The same trap then hit every test file. |
| 2026-10-23 | P0-01 | [Solution scaffold: layering test, platform capabilities, shell skeleton](./records/2026-10-23-P0-01-solution-scaffold-and-platform-capabilities.md) | `Drape.UI` legitimately depends on Avalonia, so a blanket "no UI package" rule fails on the project that exists to be a UI; the rule had to be split into shell-free (all) and UI-free (Core, Infrastructure). |
| 2026-10-10 | — | [Windows 10 minimum; other platforms' minimums and channels](./records/2026-10-10-windows-10-minimum-and-platform-minimums.md) | Windows 10 becomes a release gate and a two-path visual problem while its Microsoft support ends 2027; my edit script broke two lines and was caught by re-reading. Documents only. |
| 2026-10-10 | — | [Remaining questions answered; root README and LICENSE](./records/2026-10-10-remaining-questions-readme-licence.md) | Windows 10 is still ~28% of desktops with ESU to Oct 2027; MSIX redirects HKCU writes so the Run-key autostart would break; Store policy on hooks unknown. Documents only. |
| 2026-10-10 | — | [Owner decisions: MIT, final name, Windows first, mobile scope, Drape everywhere](./records/2026-10-10-owner-decisions-licence-name-rename.md) | Renaming the code concept to `Drape` collides with the root namespace (CS-10); "final name" is not a cleared name; ADR-021 mobile scope is the assistant's delegated working decision. Documents only. |
| 2026-10-10 | — | [Documentation restructure and translation to English](./records/2026-10-10-docs-restructure-and-translation.md) | The documents still said WPF and Windows-only after the owner chose cross-platform Avalonia; the name rationale was wrong ("Drape" is not Indonesian for fence); a `Pagar` leftover; `ImageSource` in Core. Everything translated, layout adopted, ADR-012..018 added. Nothing built or run; link targets machine-checked. |
| 2026-10-09 | — | [Project setup (retrospective)](./records/2026-10-09-project-setup.md) | Reconstructed on 2026-10-10 from the old changelog. Planning-phase start: structure, 15 Indonesian documents, five ADR files, tasks M0-M5, D-001..D-011, Q-01..Q-14. No code. |

## Decisions

Full text in [`DECISIONS.md`](./DECISIONS.md). Part II (2026-10-10) says what changed since Part I.

| ID | Date | Decision | Status |
|---|---|---|---|
| ADR-024 | 2026-10-23 | The desktop app references the platform shell directly; one thin head per OS in Phase 6 | Accepted |
| ADR-023 | 2026-10-10 | Windows 10 22H2 is the supported minimum (owner); minimums and channels for macOS, Linux, Android, iOS | Accepted (Windows: owner; others: delegated) |
| ADR-022 | 2026-10-10 | Answers to Q-04…Q-13 and Q-18: direct distribution, Windows 10 until ESU ends (Oct 2027), x64 only, Guard in MVP, updates off by default | Accepted (assumption, delegated) |
| ADR-021 | 2026-10-10 | Mobile = companion shortcut groups, last in the queue (owner-delegated working decision) | Accepted (assumption) |
| ADR-020 | 2026-10-10 | Everything is called `Drape` in code (supersedes ADR-016) | Accepted |
| ADR-019 | 2026-10-10 | Licence MIT, free; product name Drape is final | Accepted |
| ADR-018 | 2026-10-10 | Documentation layout adopted from the reference repo; ADRs live here; `AGENTS.md`; task IDs `Pn-nn` | Accepted |
| ADR-017 | 2026-10-10 | Update checks are opt-in, off by default (`updateCheck`: off / manual / periodic) | Accepted (assumption; Q-18) |
| ADR-016 | 2026-10-10 | Product term "Drape", code identifier `Fence` | Superseded by ADR-020 |
| ADR-015 | 2026-10-10 | Platform seams: `IPlatformCapabilities`, locations/autostart in Shell, neutral `IconBitmap`, `IBlurEffect` | Accepted |
| ADR-014 | 2026-10-10 | v1.0 ships Windows only; other platforms are Phase 6 | Accepted (confirmed by the owner; Q-15 closed) |
| ADR-013 | 2026-10-10 | English documentation, code and commits; Indonesian conversation; UI id-ID + en-US | Accepted |
| ADR-012 | 2026-10-10 | Stack: C# / .NET 10 LTS / Avalonia UI, cross-platform | Accepted |
| ADR-011 | 2026-10-09 | One item belongs to exactly one Drape in v1 | Accepted (Q-09) |
| ADR-010 | 2026-10-09 | No telemetry and no network by default | Accepted |
| ADR-009 | 2026-10-09 | Windows 11 primary, Windows 10 22H2 best effort | Accepted (assumption; Q-05) |
| ADR-008 | 2026-10-09 | Single non-elevated process plus `Drape.Guard`, `state.json`, mutex | Accepted |
| ADR-007 | 2026-10-09 | One window per Drape; host-layer strategies A/B/C pending spike | Proposed (spike P0-03) |
| ADR-006 | 2026-10-09 | Non-destructive desktop icon strategy | Accepted (principle); technique Proposed (spike P0-04) |
| ADR-005 | 2026-10-09 | Config: one atomic `config.json`, schema version, backups, quarantine | Accepted |
| ADR-004 | 2026-10-09 | Stack C# / .NET / WPF / CsWin32 | Superseded by ADR-012 |
| ADR-003 | 2026-10-09 | Documentation in Indonesian, code in English | Superseded by ADR-013 |
| ADR-002 | 2026-10-09 | Working structure: `docs/`, `MEMORY/`, `TASKS/`, `CLAUDE.md`, agent file | Accepted |
| ADR-001 | 2026-10-09 | Working product name "Drape" (rationale corrected) | Accepted (final name by ADR-019) |

## Specs

| Task | Spec |
|---|---|
| P1-03 | [Config store](./specs/P1-03-config-store.md) — retrospective; implementation already existed. |

See [`specs/README.md`](./specs/README.md).
