# Phase 3 — Customisation and UX

**Status:** TODO (planning phase; no code yet)

**Goal:** Drape is comfortable for daily use and can be tailored: rollup, full styling, themes, snap, lock, the platform's own item context menu, ordering, a complete settings window and keyboard/screen-reader accessibility. Spec: `docs/PLAN/02-FEATURE-SPECIFICATION.md` (F-14..F-19), `docs/UI-UX/00-UI-UX-SPEC.md`.

**Exit criteria:** see Phase 3 in [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md). Card conventions: [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md).

---

### P3-01 — Rollup

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P2-12 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-14) · docs/UI-UX/00-UI-UX-SPEC.md |
| **Spec required** | no |

**Why:** A Drape that collapses to its title bar gives the user back the wallpaper without losing the layout; the pre-rollup size must survive exactly.

Double-click on the title rolls up/unrolls. Options: auto-rollup, and unroll on hover (300 ms). Height is persisted. The 150 ms animation honours the system "reduce animations" setting.

**Definition of Done**
- [ ] the height before rollup is restored exactly, including after an app restart
- [ ] no animation when the system animation setting is off (read via the platform service, not an OS check in UI)
- [ ] hover-unroll does not trigger while dragging an item across the Drape

**Abuse cases**
- Restoring "approximately" by recomputing height from item count instead of storing the original
- Honouring the animation setting only at startup, not when it changes at runtime

---

### P3-02 — Drape style (tint, opacity, blur, corners, icons, font)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P0-06, P2-12 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-15) · docs/ARCHITECTURE/02-DATA-MODEL.md · docs/PLATFORM/01-PLATFORM-SUPPORT.md · ADR-015 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** Style is the feature users notice first; it must degrade gracefully where blur is not available.

The `Style` model and a per-Drape settings UI plus a global default, with live preview. Blur goes through `IBlurEffect` (from P0-06); whether blur may be used is decided by `IPlatformCapabilities`, never by an OS version check.

**Definition of Done**
- [ ] every `Style` property can be changed and is persisted
- [ ] blur falls back to a solid colour when the capability is absent, when the system is in high-contrast mode, or when the effect fails
- [ ] no `OperatingSystem.Is*` call in UI/Core (architecture test stays green)

**Abuse cases**
- Checking "Windows 11 build >= X" in the ViewModel instead of asking `IPlatformCapabilities`
- Applying the fallback only on the settings preview, not on the real Drape window

---

### P3-03 — Light / dark / high-contrast themes and system accent

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P3-02 |
| **Spec refs** | docs/UI-UX/00-UI-UX-SPEC.md · docs/PLATFORM/01-PLATFORM-SUPPORT.md · ADR-015 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** A panel over an arbitrary wallpaper is unreadable unless text contrast is actively managed.

A `ThemeService` built on Avalonia theme variants follows the system theme, accent and high-contrast state at runtime (through the platform service, gated by capabilities). Option "Ensure contrast".

**Definition of Done**
- [ ] a system theme/accent/high-contrast change applies without restart
- [ ] text contrast >= 4.5:1, checked against several test wallpapers (light, dark, busy, photo)
- [ ] high-contrast mode disables blur and tint (consistent with P3-02)

**Abuse cases**
- Verifying contrast on one dark wallpaper only
- Hard-coded colours in views that bypass the theme resources

---

### P3-04 — Snap

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 2 days |
| **Role** | core-engineer |
| **Depends on** | P1-06 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-16) · docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md |
| **Spec required** | no |

**Why:** Snap is pure geometry; it belongs in Core where it can be exhaustively unit-tested.

Snap algorithm in `LayoutService`: monitor edges, other Drapes, optional grid; threshold 8 DIP; holding Alt disables it. The UI draws guide lines.

**Definition of Done**
- [ ] unit tests cover edges, other Drapes, grid, threshold boundary (7/8/9 DIP), Alt override and multi-monitor
- [ ] guide lines appear without flicker (headless UI test plus manual check at 150% DPI)

**Abuse cases**
- Tests that compute the expected position with the same formula as the implementation
- Snap computed in physical pixels and compared against a DIP threshold

---

### P3-05 — Lock a Drape

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 1 day |
| **Role** | ui-engineer |
| **Depends on** | P2-12 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-17) · docs/UI-UX/00-UI-UX-SPEC.md |
| **Spec required** | no |

**Why:** Users who finish a layout want it to stay finished.

A locked Drape cannot be moved or resized; an option allows adding items to it.

**Definition of Done**
- [ ] resize cursor and title-bar drag are disabled; a lock indicator is shown
- [ ] the lock state is persisted and survives restart
- [ ] a locked Drape still accepts drops when the "allow adding" option is on, and rejects them when off

**Abuse cases**
- Disabling only the visual affordance while the programmatic move (snapshot restore, page switch) can still alter bounds without being intended
- Lock flag stored in the view, not in the model

---

### P3-06 — Shell context menu for items

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 4 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P2-12 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-18) · docs/PLATFORM/00-WINDOWS-INTEGRATION.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md |
| **Spec required** | yes |

**Why:** Third-party shell extensions run inside our process when we host the native menu; a slow or crashing handler must never freeze the desktop panel.

`IShellContextMenu` implemented with `IContextMenu2/3` on a separate STA thread with a timeout; Drape's own commands are added; a minimal fallback menu is used when the native one fails. Spec first in `MEMORY/specs/P3-06-shell-context-menu.md`.

**Definition of Done**
- [ ] submenus (Send to, Open with) work
- [ ] a slow or crashing third-party handler does not freeze the UI (test with a deliberately slow fake handler and a timeout)
- [ ] the menu is correct at 150% DPI and in dark theme
- [ ] all COM objects and PIDLs are released (leak test over repeated open/close)
- [ ] "Delete" uses the platform's own confirmation and Recycle Bin (never a direct delete)

**Abuse cases**
- Hosting the menu on the UI thread and calling it "async" because the call is awaited
- Catching the handler's exception but leaving the STA thread and its COM objects leaked

---

### P3-07 — Item ordering and drag-to-reorder

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | ui-engineer |
| **Depends on** | P1-09 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-19) · docs/ARCHITECTURE/02-DATA-MODEL.md |
| **Spec required** | no |

**Why:** Unstable sorting makes icons "jump" for no visible reason, which users read as data corruption.

Modes: manual, name, type, modified date. Drag to reorder in manual mode. Optional folders-first.

**Definition of Done**
- [ ] manual order is persisted
- [ ] sorting is stable (items with equal keys never reorder between runs); a unit test uses many equal keys
- [ ] name sort follows platform case rules via the injected file-system service, not a hard-coded comparison

**Abuse cases**
- A test with distinct keys only, which cannot detect instability
- Switching from manual to name sort and back destroys the saved manual order

---

### P3-08 — Complete settings window

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P3-02, P2-11 |
| **Spec refs** | docs/UI-UX/00-UI-UX-SPEC.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md |
| **Spec required** | no |

**Why:** Settings that need a restart or a hand-edited file are settings users never find.

Tabs: General, Default Drape, Desktop, Advanced (diagnostics without paths), About. "Restore defaults" button.

**Definition of Done**
- [ ] every setting applies immediately and is persisted
- [ ] "Copy diagnostic info" contains no user path, user name or file name (test with a fixture that has a path-bearing config)
- [ ] settings that the current platform cannot honour are hidden or disabled based on `IPlatformCapabilities`, with an explanation

**Abuse cases**
- Diagnostics that "mask" paths by truncating them but still leak the user name
- Restore defaults that also wipes Drapes and item membership

---

### P3-09 — Keyboard accessibility and screen reader support

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P3-06 |
| **Spec refs** | docs/UI-UX/00-UI-UX-SPEC.md · docs/TESTING/00-TESTING-STRATEGY.md |
| **Spec required** | no |

**Why:** A desktop surface that only works with a mouse excludes a whole class of users.

Keyboard focus, arrow-key navigation, Avalonia `AutomationProperties` and automation peers, visible focus ring, "Focus a Drape" hotkey (prototype).

**Definition of Done**
- [ ] every core function (open, select, move between Drapes, context menu, rollup, settings) is reachable by keyboard
- [ ] an Accessibility Insights scan reports no critical issues
- [ ] Narrator announces each Drape's name and each item's name and type
- [ ] focus ring contrast meets the theme requirements in P3-03, including high-contrast mode

**Abuse cases**
- Setting `AutomationProperties.Name` to the file path or to a generic "item"
- Tab order that works only after the mouse has focused the window once

---

### P3-10 — Phase 3 testing and interaction polish

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 2 days |
| **Role** | qa |
| **Depends on** | P3-01..P3-09 |
| **Spec refs** | docs/TESTING/00-TESTING-STRATEGY.md · docs/PLAN/01-PRODUCT-REQUIREMENTS.md (NFR-01..03) |
| **Spec required** | no |

**Why:** Phase 4 builds on this surface; regressions found now are cheap, later they are expensive.

Full regression, DPI/theme matrix, performance check; UX findings recorded for Phase 5.

**Definition of Done**
- [ ] the full manual checklist was run; no regression against the MVP criteria
- [ ] performance within the NFR budgets (named measurements with build, DPI and monitor setup)
- [ ] UX findings are written to `MEMORY/records/` and linked from `TASKS/PROGRESS.md`

**Abuse cases**
- "Regression passed" with no named test or checklist item
- Measuring performance with 3 Drapes when the budget is defined for 20 x 20
