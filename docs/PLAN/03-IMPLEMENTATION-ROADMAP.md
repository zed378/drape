# 03 — Implementation Roadmap

Revised 2026-11-25 for [ADR-025](../../MEMORY/DECISIONS.md). Rust + Slint is the target; existing C# code/evidence is retained as history. The current owner request changes documentation only. No dates or old C# estimates are promised for the rewrite; re-estimate after Rust spikes. Status authority: [PROGRESS](../../TASKS/PROGRESS.md).

## Phases and replacement IDs

| Work | Tasks | Exit gate |
|---|---|---|
| Documentation baseline | R0-01 in [Rust migration](../../TASKS/RUST-MIGRATION.md) | Consistent docs, links/scope checks, record, independent review |
| Rust foundation | R0-02/R0-03 | Pinned workspace/lock/licences, real dependency-boundary checks and Rust CI |
| Windows evidence | R0-04–R0-10 | Slint host/rendering, native icons/Guard, icons/cache, hooks, drops/UIA, lifecycle/resource baseline and accepted ADRs |
| Core and first usable Drape | R1-01–R1-06 | Rust domain/config/layout/UI/services/parity baseline; meaningful tests and Core coverage >=80% |
| Desktop integration (MVP) | [P2](../../TASKS/PHASE-2-DESKTOP-INTEGRATION.md) | Safe icon hiding/recovery, desktop sync, Inbox, quick-hide, session resilience |
| Customisation/UX | [P3](../../TASKS/PHASE-3-CUSTOMIZATION.md) | Rollup/style/snap/lock/context menu/settings and keyboard/UIA/themes |
| Advanced features | [P4](../../TASKS/PHASE-4-ADVANCED.md) | Portals/rules/pages/snapshots/export/import; planned optional features |
| Polish/Windows release | [P5](../../TASKS/PHASE-5-POLISH-RELEASE.md) | Localisation, resource gates, signing/packaging/recovery and full matrix |
| Other platforms | [P6](../../TASKS/PHASE-6-CROSS-PLATFORM.md) | Post-v1; each platform's spike/ADR before implementation/support promise |

P0/P1 files are frozen historical C# snapshots. Their DONE/WIP/TODO entries never count as active Rust progress or satisfy new dependencies. Existing P2–P6 IDs remain prospective product work, with dependencies updated to R0/R1. Old harnesses may inform hypotheses but cannot prove Slint compatibility.

## Critical order

```text
R0-01 -> R0-02 -> R0-03 (CI)
               -> R0-04 (Slint host) -> R0-07 (renderer/drop/UIA)
               -> R0-05 (icons/Guard) -------------------------> R0-09 (lifecycle)
               -> R0-06 (icon extraction/cache)
               -> R0-08 (input hook)
R0-04..R0-09 -> R0-10 (accepted host/backend/recovery ADRs)
R0-02 -> R1-01 -> R1-02 (config), R1-03 (layout)
R0-06 + R0-10 + R1-01 + R1-03 -> R1-04 (panel/items)
R0-09 + R1-02 + R1-04 -> R1-05 -> R1-06 -> P2 -> P3 -> P4 -> P5 -> P6
```

This dependency diagram does not authorise starting any implementation today. Follow per-card prerequisites and WIP <=3; architecture contracts precede their implementers.

## Acceptance boundaries

- Foundation: actual Slint windows must meet desktop-host criteria on Windows 10 22H2/Windows 11 builds, mixed-DPI monitors, Win+D/peek, keyboard focus, manual Explorer restart, transparency and native drops. Select backend/renderer only with measurements.
- Safety: serialise recovery ownership before startup replay; validate active generation and authenticate App/Guard/CLI commands. Guard readiness and durable intent precede hiding, including quick-hide changes. Guard loss while App survives forces restoration/no rehide until rearmed. Restore normal/error/panic/abort/kill/next-start/CLI paths, preserving user-hidden state. Drop/panic hooks alone never satisfy recovery.
- Core/UI: create/move/resize/rename/delete Drapes, reference-only item drops/open/removal, atomic schema-compatible persistence and stable DPI/monitor layout. R1-06 names Rust evidence, not old tests.
- MVP: 3 Drapes persist across restart; desktop additions appear within 2 s; no user file moves; icon restore on normal/error/forced exit; layout/Win+D/Explorer recovery proven. Blur remains planned with fallback; Q-19 decides MVP priority.
- Release: App + Guard resource budgets, startup, measured footprint, keyboard/UIA/high contrast, signed installer/portable/update/uninstall recovery, notices and opt-in network behaviour. Q-20 sets size/tighter RAM targets after measurements; Q-22 validates packaging.
- Phase 6: no mobile desktop layer, no assumption that old Avalonia support tiers establish Rust/Slint minimums. Revalidate inherited OS targets/backend/accessibility/store policy before committing.

## Schedule risks and decisions

Host/transparent child-window failure, keyboard/drop/UIA integration, unsafe native lifetime bugs, renderer footprint, licence attribution and Rust updater compatibility are explicit risks (PR-1–PR-12). A degraded fallback must be labelled; do not call bottom-most windows Win+D-proof without evidence. Stack replacement does not permit deleting features, changing schema without migration or retiring legacy code silently.

Open decisions Q-19–Q-22 live in [BACKLOG](../../TASKS/BACKLOG.md). [Migration design](../ARCHITECTURE/03-RUST-MIGRATION.md) governs parity, measurement and code-retirement boundaries. No implementation or old-source deletion is part of this documentation update.
