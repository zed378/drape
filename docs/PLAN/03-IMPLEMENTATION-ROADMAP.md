# 03 — Implementation Roadmap

Rough estimates for one full-time developer; adjust when several agents or people work in parallel. Task cards live in [`../../TASKS/`](../../TASKS/README.md); the live status is [`../../TASKS/PROGRESS.md`](../../TASKS/PROGRESS.md).
Dates are deliberately not fixed: the order and the exit criteria are what bind.

| Phase | Theme | Estimate | Task file |
|---|---|---|---|
| 0 | Foundation and spikes | ~2 weeks | [PHASE-0](../../TASKS/PHASE-0-FOUNDATION.md) |
| 1 | Core Drape | ~3 weeks | [PHASE-1](../../TASKS/PHASE-1-DRAPE-CORE.md) |
| 2 | Desktop integration (MVP) | ~3 weeks | [PHASE-2](../../TASKS/PHASE-2-DESKTOP-INTEGRATION.md) |
| 3 | Customisation and UX | ~3 weeks | [PHASE-3](../../TASKS/PHASE-3-CUSTOMIZATION.md) |
| 4 | Advanced features | ~4 weeks | [PHASE-4](../../TASKS/PHASE-4-ADVANCED.md) |
| 5 | Polish and release (v1.0, Windows) | ~3 weeks | [PHASE-5](../../TASKS/PHASE-5-POLISH-RELEASE.md) |
| 6 | Cross-platform expansion (post-v1) | to be estimated after the spikes | [PHASE-6](../../TASKS/PHASE-6-CROSS-PLATFORM.md) |

About 18 weeks to v1.0. Internal alpha after Phase 2, beta after Phase 4. Phase 6 is planned, not committed ([ADR-014](../../MEMORY/DECISIONS.md)).

## Exit Criteria

### Phase 0 — Foundation and spikes
- Solution and CI run; build and tests are green on a clean runner.
- Every spike (P0-03 … P0-07) has a written result; the host-layer decision (ADR-007) is accepted and the icon strategy ADR-006 is completed with the chosen technique.
- A prototype of `--restore-desktop` works.
- Stack question resolved ([ADR-012](../../MEMORY/DECISIONS.md), closes Q-01).

### Phase 1 — Core Drape
- A user can create, move, resize, rename and delete Drapes.
- Drag-drop of items from Explorer; open and remove items.
- Layout is saved atomically and restored at start; per-monitor DPI.
- Core coverage ≥ 80%.

### Phase 2 — Desktop integration (MVP)
- Native icons hidden and **always** restored (normal exit, crash, kill, reboot).
- Desktop sync, Inbox, quick-hide, right-drag creation, autostart.
- Recovers from Explorer restart, Win+D and display changes.
- The MVP acceptance criteria in [`01`](./01-PRODUCT-REQUIREMENTS.md) are met. Internal alpha release.

### Phase 3 — Customisation and UX
- Rollup, full style, snap, lock, shell context menu, ordering, settings window, light/dark/high-contrast themes.
- Keyboard accessibility and basic UIA.

### Phase 4 — Advanced features
- Folder portal, auto-sort rules, desktop pages, snapshots, export/import; tabs, search and hotkeys if time allows.
- Limited public beta.

### Phase 5 — Polish and release
- Full id-ID and en-US localisation, performance within the NFRs, installer + updater, code signing, user documentation, full environment testing.
- Release v1.0 (Windows).

### Phase 6 — Cross-platform expansion
- Platform spikes (macOS, Linux X11, Linux Wayland) each end in an ADR *before* the matching Shell project is implemented.
- Mobile scope is defined by a design card before any mobile code.
- Each platform reaches parity only for the capabilities its spike proved; unsupported capabilities are reported through `IPlatformCapabilities`, not faked.

## Critical Dependencies

```
P0-01 ─┬─ P0-02 (CI)
       ├─ P0-03 … P0-07 (parallel spikes) ─ P0-09 (ADRs) ─ P1-*
       └─ P0-08 (host/DI/tray) ──────────────────────────────┘
P1 ─ P2 ─ P3 ─ P4 ─ P5          P6 starts after P5 (spikes may start earlier, off the critical path)
```

## Schedule Risks

| Risk | Impact | Response |
|---|---|---|
| Host spike fails on Windows 11 24H2+ | Phase 0 slips a week, Phase 1 changes | Strategy B is the safety net |
| Avalonia cannot be re-parented / made transparent as needed | Host strategy narrows | Spike P0-03 and P0-06 decide early; fall back to strategy B |
| Shell context menu harder than expected | Phase 3 slips | Start with a minimal menu, refine in Phase 5 |
| Antivirus flags the hook | Release delayed | Start signing early; request vendor exclusions |
| Scope creep | Everything slips | New features go to the Backlog and need orchestrator approval |

## After v1 (ideas)

Virtual items (This PC, Recycle Bin), full ARM64, community themes, widgets, "Drape per project" tied to a working folder, plugins. Tracked in [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md).
