# Phase 4 — Advanced Features

**Status:** TODO (planning phase; no code yet)

**Goal:** The differentiating features: folder portals, auto-sort rules, desktop pages, snapshots and export/import. Tabs, search and global hotkeys are optional (feature matrix marks them "Later") and are done only if time allows. Spec: `docs/PLAN/02-FEATURE-SPECIFICATION.md` (F-20..F-27). Priority order within the phase: portals and rules first.

**Exit criteria:** see Phase 4 in [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md). Card conventions: [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md).

---

### P4-01 — Folder portal

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 4 days |
| **Role** | core-engineer |
| **Depends on** | P3-10 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-20) · docs/ARCHITECTURE/02-DATA-MODEL.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md |
| **Spec required** | no |

**Why:** A portal mirrors a live folder; the folder is the source of truth and Drape must never silently modify it.

A Drape with `kind=portal` mirrors a folder's contents (optional recursion/filter), updated through the file watcher. Items are not stored in config. Dropping copies or moves with confirmation.

**Definition of Done**
- [ ] a folder of 5,000 files stays responsive (virtualised list; named measurement)
- [ ] a drop into a portal asks for confirmation and never overwrites without asking
- [ ] watcher bursts (copying 1,000 files) are coalesced and do not flood the UI thread
- [ ] icon loading for portal items avoids triggering cloud-file hydration (OneDrive)

**Abuse cases**
- Performance proven with 50 files, extrapolated to 5,000
- "Confirmation" implemented as a dialog that defaults to overwrite

---

### P4-02 — Rule engine (Core)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | core-engineer |
| **Depends on** | P2-07 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-21) · docs/ARCHITECTURE/02-DATA-MODEL.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md |
| **Spec required** | yes |

**Why:** User-authored patterns run against every new file; a pathological regex or a rule that keeps re-moving items would make the desktop unusable.

`RuleEngine`, `IRuleCondition`, `IRuleAction`. Conditions: extension, name pattern (glob/regex with a timeout), age, size, type. Priorities and `pinnedByUser`. Spec first in `MEMORY/specs/P4-02-rule-engine.md`.

**Definition of Done**
- [ ] unit tests for priority order and combined conditions (AND/OR), including conflicting rules
- [ ] a catastrophic-backtracking regex is cut off by the evaluation timeout and reported, not hung
- [ ] an item moved manually by the user is never re-sorted (`pinnedByUser`)
- [ ] applying rules is deterministic: the same input state always yields the same result

**Abuse cases**
- Timeout enforced only in tests via a tiny input, never against a real backtracking pattern
- `pinnedByUser` cleared on every app restart because it was not persisted

---

### P4-03 — Rules UI

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P4-02, P3-08 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-21) · docs/UI-UX/00-UI-UX-SPEC.md |
| **Spec required** | no |

**Why:** Auto-sort moves things around; the user must see what will happen before it happens, and be able to take it back.

Rules tab: create/edit/reorder, impact preview, "Apply now" button.

**Definition of Done**
- [ ] the preview lists the items that would move (and where) before anything is applied
- [ ] the last application can be undone in one step
- [ ] the tab is keyboard-operable and exposes automation names (consistent with P3-09)

**Abuse cases**
- A preview computed by a different code path than the real apply, so they can disagree
- Undo that restores membership but not order or position

---

### P4-04 — Desktop pages

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 4 days |
| **Role** | ui-engineer |
| **Depends on** | P3-10 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-22) · docs/ARCHITECTURE/02-DATA-MODEL.md · docs/UI-UX/00-UI-UX-SPEC.md |
| **Spec required** | no |

**Why:** Pages multiply the number of windows; switching must be fast and must not lose or duplicate Drapes.

Several pages, switching via tray/hotkey/screen edge; a Drape can "stick to all pages".

**Definition of Done**
- [ ] page switch < 150 ms without flicker (named measurement with Drape count)
- [ ] pages and Drape membership are persisted
- [ ] a sticky Drape appears on every page and is stored once (no duplicated model)
- [ ] hidden-page windows are not rendered (CPU stays within NFR-01)

**Abuse cases**
- Meeting 150 ms by keeping all pages' windows alive and merely toggling opacity
- A sticky Drape implemented as copies per page that drift apart

---

### P4-05 — Layout snapshots

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 3 days |
| **Role** | core-engineer |
| **Depends on** | P3-10 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-23) · docs/ARCHITECTURE/02-DATA-MODEL.md |
| **Spec required** | no |

**Why:** Snapshots are the user's undo for "I rearranged everything"; a restore that silently drops items is worse than none.

Save/restore of positions, style and membership; preview and confirmation; handles items that no longer exist.

**Definition of Done**
- [ ] restore returns the layout exactly (hand-written fixture compared field by field) and reports each missing item
- [ ] restore requires confirmation and keeps a backup of the layout it replaces
- [ ] a snapshot taken on one monitor setup restores sensibly on another (clamped to visible area)

**Abuse cases**
- A round-trip test that serialises and deserialises the same object and so cannot see a dropped field
- Missing items silently omitted rather than reported

---

### P4-06 — Config export / import

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 2 days |
| **Role** | core-engineer |
| **Depends on** | P3-08 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-26) · docs/ARCHITECTURE/02-DATA-MODEL.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md · ADR-005 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** An exported file is a document that leaves the machine; it must not leak paths unless the user chose so, and an import must not destroy the current layout.

Export to a `.drape.json` file with an "anonymise paths" option; import in merge or replace mode with a preview. Imports go through the same schema-version validation and migration as the ConfigStore.

**Definition of Done**
- [ ] the anonymise option removes absolute paths and the user name (asserted on the output file)
- [ ] import never overwrites without confirmation and always creates a backup first
- [ ] a malformed, oversized or newer-schema file is rejected with a clear message and changes nothing
- [ ] imported paths are validated; no path outside what the user could add manually is trusted blindly

**Abuse cases**
- Anonymisation that only rewrites the `Users\<name>` prefix but leaves network paths and file names
- Import that deserialises first and validates after, applying partial state on failure

---

### P4-07 — Tabs inside a Drape (optional)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 4 days |
| **Role** | ui-engineer |
| **Depends on** | P4-01 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-24) · docs/UI-UX/00-UI-UX-SPEC.md |
| **Spec required** | no |

**Why:** Optional; takes the phase beyond budget unless portals and rules are already done.

Tabs per Drape (item list or portal); dragging an item onto a tab moves it.

**Definition of Done**
- [ ] tabs are persisted; dragging onto a tab header moves the item
- [ ] a Drape with one tab looks and behaves exactly like today's Drape (no regression)

**Abuse cases**
- Tab state kept in the view only, lost on restart
- Starting this card while P4-01/P4-02 are still open, against the phase priority

---

### P4-08 — Search inside a Drape (optional)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 1 day |
| **Role** | ui-engineer |
| **Depends on** | P3-09 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-25) · docs/UI-UX/00-UI-UX-SPEC.md |
| **Spec required** | no |

**Why:** Quick filtering makes large Drapes (and portals) usable; it must not steal typing from other apps.

Typing while a Drape has focus filters its items; Esc cancels.

**Definition of Done**
- [ ] filtering is instant for 1,000 items (named measurement)
- [ ] the filter box is reachable and announced by Narrator; Esc restores the full list and focus
- [ ] keystrokes are handled only when the Drape has focus (no global keyboard hook)

**Abuse cases**
- Implementing "type anywhere to search" with a low-level keyboard hook
- Filter state that persists invisibly and leaves items looking deleted

---

### P4-09 — Global hotkeys (optional)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P2 |
| **Estimate** | 2 days |
| **Role** | shell-engineer-windows |
| **Depends on** | P3-09 |
| **Spec refs** | docs/PLAN/02-FEATURE-SPECIFICATION.md (F-27) · docs/PLATFORM/00-WINDOWS-INTEGRATION.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md |
| **Spec required** | no |

**Why:** A keyboard hook would see every keystroke the user types, including passwords; only registered hotkeys are acceptable.

`RegisterHotKey` for toggle show/hide, page switch and focus a Drape. No keyboard hooks.

**Definition of Done**
- [ ] a hotkey conflict is handled with a clear message and a way to choose another key
- [ ] there is no keyboard hook in the code (`SetWindowsHookEx` with `WH_KEYBOARD(_LL)` absent; verified by an architecture/grep test and by review)
- [ ] the registration is released on every exit path
- [ ] the hotkey capability is exposed through `IPlatformCapabilities` so other platforms can report it differently

**Abuse cases**
- A "temporary" `WH_KEYBOARD_LL` hook to detect conflicts
- Hotkeys logged (key combo plus active window title) for debugging and left in release builds
