# Phase 6 — Cross-Platform Expansion

**Status:** TODO (planning phase; post-v1, **planned but not committed**. Windows-first is confirmed by the owner, ADR-014.)

**Minimum OS and channels (ADR-023):** macOS 14+, Ubuntu 22.04 / Debian 12 families (X11 guaranteed), Android 12+, iOS 18+; distribution per [`../docs/PLATFORM/01-PLATFORM-SUPPORT.md`](../docs/PLATFORM/01-PLATFORM-SUPPORT.md). A spike or implementation card states in its result which OS versions it actually tested.

**Goal:** Bring Drape to macOS, Linux and, if the design card says it is in scope at all, Android and iOS, without breaking the Windows product or the architecture rules.

**Ground rules for this phase**
- **Every platform needs a spike and an ADR before its `Drape.Shell.<Platform>` project is implemented.** Platform behaviour listed in [`../docs/PLATFORM/01-PLATFORM-SUPPORT.md`](../docs/PLATFORM/01-PLATFORM-SUPPORT.md) is a **hypothesis** until the matching spike records it, naming the OS build, hardware and display setup.
- **Capability-gating.** UI and Core ask `IPlatformCapabilities` (ADR-015); they never test the operating system. A platform ships only the capabilities its spike proved.
- **No faking.** A capability that a platform cannot provide is reported as missing in settings and docs, and Drape degrades to ordinary windows; it is never emulated to look as if it worked.
- **Desktop-safety rules apply on every platform.** Anything that hides native icons needs a state record, a restore on every exit path and a Guard equivalent (ADR-006, ADR-008).
- **All content here derives from the hypotheses in `docs/PLATFORM/01-PLATFORM-SUPPORT.md`.** If a spike contradicts them, the deviation protocol in [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md) applies: ADR first, then amend the docs.
- Platforms are independent and can proceed in parallel with different shell engineers. Windows keeps working at every step (the Windows CI and QA matrix must stay green).

**Exit criteria:** see Phase 6 in [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md). Spec refs: `docs/PLATFORM/01-PLATFORM-SUPPORT.md`, ADR-014 and ADR-015 in `MEMORY/DECISIONS.md`.

---

### P6-01 — Spike: macOS desktop-layer window

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-mac |
| **Depends on** | P0-06 |
| **Spec refs** | docs/PLATFORM/01-PLATFORM-SUPPORT.md (macOS) · docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md · ADR-007, ADR-014, ADR-015 in MEMORY/DECISIONS.md |
| **Spec required** | yes |

**Why:** Whether an Avalonia window can sit above the wallpaper and below normal windows on macOS is unknown; promising it before testing would repeat the Windows desktop-host risk on a platform we know less about.

Prove or refute: an Avalonia window set to the desktop-icon window level (via `CGWindowLevelForKey`) with a collection behaviour that keeps it out of Mission Control and Cmd-Tab, transparent, click-through where needed, correct on Retina and multiple displays, across Spaces and fullscreen apps. Spec (hypotheses and pass criteria) first in `MEMORY/specs/P6-01-macos-desktop-layer.md`; output is a record plus an ADR.

**Definition of Done**
- [ ] each of the host criteria is marked pass/fail with the macOS version, hardware (Intel/Apple silicon) and display setup named
- [ ] behaviour is recorded for Spaces switching, fullscreen apps, display hot-plug, and light/dark appearance
- [ ] an ADR states go / no-go / go-with-limits; the capability matrix in `docs/PLATFORM/01-PLATFORM-SUPPORT.md` is amended through the deviation protocol
- [ ] **QA matrix added:** macOS version x chip x display scale x Spaces/fullscreen scenarios recorded in `docs/TESTING/00-TESTING-STRATEGY.md`

**Abuse cases**
- Declaring success because the window "renders" on the developer's Mac without testing Spaces or fullscreen
- A throwaway P/Invoke prototype copied into `Drape.Shell.Mac` unchanged

---

### P6-02 — Spike: macOS hide Finder desktop icons and recovery

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 (safety) |
| **Estimate** | 3 days |
| **Role** | shell-engineer-mac |
| **Depends on** | — |
| **Spec refs** | docs/PLATFORM/01-PLATFORM-SUPPORT.md (macOS) · docs/PLATFORM/00-WINDOWS-INTEGRATION.md (for the Windows safeguards to mirror) · docs/SECURITY/00-SECURITY-AND-PRIVACY.md · ADR-006, ADR-008 in MEMORY/DECISIONS.md |
| **Spec required** | yes |

**Why:** The known way to hide icons is a persistent Finder preference plus a Finder restart. That changes user settings and can strand the user, so whether it is acceptable at all is the question, not an implementation detail.

Evaluate the options (persistent Finder preference and restart, covering the icons with Drape's own window, or not hiding icons and only overlaying) for: persistence across logout/reboot/crash, Finder restart side effects (open windows, other apps), recovery when Drape dies, recovery when the user changed the preference themselves, and user-visible disclosure. **Safety rules apply in full:** no automated test may restart the developer's Finder; Finder tests run in a VM or test account only; record how a stranded user recovers by hand.

**Definition of Done**
- [ ] each option is assessed against every exit path (normal quit, crash, kill, logout, reboot, power loss) with the observed result
- [ ] the "persistent preference" option is explicitly accepted or rejected, with the restart risk written out
- [ ] a documented manual recovery procedure was executed once on a VM and works
- [ ] an ADR states go / no-go / go-with-limits (including "do not hide, overlay only"); the capability matrix is amended through the deviation protocol
- [ ] **QA matrix added:** a macOS "icon hide / restore on every exit path" matrix (VM or test account only) in `docs/TESTING/00-TESTING-STRATEGY.md`

**Abuse cases**
- Testing only the clean-quit path and calling the option safe
- Writing the Finder preference in a test on the developer's own account and "restoring it afterwards"

---

### P6-03 — Spike: Linux X11 desktop-layer window

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-linux |
| **Depends on** | P0-06 |
| **Spec refs** | docs/PLATFORM/01-PLATFORM-SUPPORT.md (Linux X11) · docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md · ADR-007, ADR-014, ADR-015 in MEMORY/DECISIONS.md |
| **Spec required** | yes |

**Why:** X11 window-type hints behave differently per window manager and desktop environment; "works on GNOME" says nothing about KDE or XFCE.

Prove or refute: an Avalonia window with `_NET_WM_WINDOW_TYPE_DESKTOP` or `_NET_WM_STATE_BELOW` plus skip-taskbar/pager hints stays above the wallpaper and below normal windows, is transparent (compositor present and absent), and handles multi-monitor and fractional scaling. Also record how native desktop icons are handled per environment (file-manager-owned) and whether hiding them is feasible at all. Spec first in `MEMORY/specs/P6-03-linux-x11-desktop-layer.md`.

**Definition of Done**
- [ ] a result table per desktop environment/window manager (at least GNOME-on-X11, KDE Plasma, XFCE) with versions named
- [ ] behaviour recorded with and without a compositor, on multi-monitor, and at fractional scaling
- [ ] the native-icon question is answered per environment, with "not supported" a valid answer
- [ ] an ADR states go / no-go / go-with-limits per environment; the capability matrix is amended through the deviation protocol
- [ ] **QA matrix added:** distribution x desktop environment x compositor x scale scenarios in `docs/TESTING/00-TESTING-STRATEGY.md`

**Abuse cases**
- Generalising from one distribution to "Linux"
- Hiding icons by killing or disabling the user's file manager or desktop process

---

### P6-04 — Spike: Linux Wayland layer-shell feasibility per compositor

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | shell-engineer-linux |
| **Depends on** | P0-06 |
| **Spec refs** | docs/PLATFORM/01-PLATFORM-SUPPORT.md (Linux Wayland) · docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md · ADR-007, ADR-014, ADR-015 in MEMORY/DECISIONS.md |
| **Spec required** | yes |

**Why:** Wayland has no global window-level API; the panel is possible only where the compositor offers `wlr-layer-shell`, which stock GNOME does not.

Determine, per compositor, whether a background-layer surface can host an Avalonia window (or whether a separate native surface is needed), how input and transparency work, and what the fallback is where it is missing. Include global-hook and hotkey implications (no global mouse hook by design; portal-based hotkeys). Spec first in `MEMORY/specs/P6-04-wayland-layer-shell.md`.

**Definition of Done**
- [ ] a compositor feasibility table (at least wlroots-based such as Sway, KDE Plasma, GNOME/Mutter) with versions and protocol support named
- [ ] the fallback ("ordinary windows, capability reported missing") is demonstrated, not assumed
- [ ] quick-hide / right-drag alternatives without a global hook are assessed
- [ ] an ADR states go / no-go / go-with-limits per compositor; the capability matrix is amended through the deviation protocol
- [ ] **QA matrix added:** compositor x scale x input scenarios in `docs/TESTING/00-TESTING-STRATEGY.md`

**Abuse cases**
- Reporting layer-shell success on a compositor but never running on stock GNOME
- Faking "desktop layer" with an always-on-bottom normal window and calling the capability present

---

### P6-05 — Implement `Drape.Shell.Mac`

| | |
|---|---|
| **Status** | BLOCKED (waits on P6-01, P6-02; Windows ships first) |
| **Priority** | P1 |
| **Estimate** | 5 days |
| **Role** | shell-engineer-mac |
| **Depends on** | P6-01, P6-02 |
| **Spec refs** | docs/PLATFORM/01-PLATFORM-SUPPORT.md · docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md · ADR-007, ADR-015 in MEMORY/DECISIONS.md · ADRs from P6-01 and P6-02 |
| **Spec required** | no |

**Why:** This is where the spike findings become a shipped platform layer, restricted to what the spikes proved.

Implement the interfaces from Core for macOS: desktop host, icon extraction (returning `IconBitmap`), `IDesktopLocations`, `IAutostart` (login item), `IPlatformCapabilities`, and icon hiding only if P6-02 accepted it. P/Invoke/Cocoa only in `Drape.Shell.Mac`; native handles wrapped in `SafeHandle`/`IDisposable`.

**Definition of Done**
- [ ] each implemented interface has contract tests shared with the Windows implementation where behaviour is platform-neutral
- [ ] `IPlatformCapabilities` reports exactly what the spikes proved; unsupported capabilities are reported missing and the UI degrades accordingly
- [ ] the architecture tests pass: no Cocoa/native code outside `Drape.Shell.Mac`, no OS checks in UI/Core
- [ ] autostart enable/disable round-trips and leaves no stray login item after disable
- [ ] **QA matrix added:** the macOS rows defined in P6-01/P6-02 are run against this implementation

**Abuse cases**
- Reporting a capability as present in `IPlatformCapabilities` because the code exists, without the spike evidence
- A Windows-only assumption (case-insensitive paths, `Desktop` location) leaking into shared code

---

### P6-06 — Implement `Drape.Shell.Linux`

| | |
|---|---|
| **Status** | BLOCKED (waits on P6-03, P6-04; Windows ships first) |
| **Priority** | P1 |
| **Estimate** | 5 days |
| **Role** | shell-engineer-linux |
| **Depends on** | P6-03, P6-04 |
| **Spec refs** | docs/PLATFORM/01-PLATFORM-SUPPORT.md · docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md · ADR-007, ADR-015 in MEMORY/DECISIONS.md · ADRs from P6-03 and P6-04 |
| **Spec required** | no |

**Why:** The Linux layer must work on the environments the spikes proved and say honestly that it does not on the rest.

Implement the Core interfaces for Linux: X11 host and (where P6-04 says go) Wayland layer-shell host, icon extraction via the freedesktop icon theme, `IDesktopLocations` (XDG user dirs), `IAutostart` (XDG autostart), `IPlatformCapabilities`, with per-environment detection.

**Definition of Done**
- [ ] capability detection selects X11 / layer-shell / ordinary-window fallback at runtime and reports it
- [ ] on an unsupported compositor Drape runs as ordinary windows, with the capability reported missing in settings
- [ ] XDG user directories (including localised and redirected Desktop) are used; no hard-coded `~/Desktop`
- [ ] architecture tests pass: no X11/Wayland/GTK code outside `Drape.Shell.Linux`
- [ ] **QA matrix added:** the distribution x environment x compositor rows from P6-03/P6-04 are run against this implementation

**Abuse cases**
- A single code path assuming X11 that silently does nothing on Wayland
- Treating the `~/Desktop` path as universal

---

### P6-07 — Guard equivalents for macOS and Linux

| | |
|---|---|
| **Status** | BLOCKED (waits on P6-05 and P6-06) |
| **Priority** | P0 (safety) |
| **Estimate** | 4 days |
| **Role** | shell-engineer-mac |
| **Depends on** | P6-05 (macOS) · P6-06 (Linux) |
| **Spec refs** | docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md · docs/PLATFORM/01-PLATFORM-SUPPORT.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md · ADR-006, ADR-008 in MEMORY/DECISIONS.md |
| **Spec required** | yes |

**Why:** If a platform can hide native icons, a crash must not leave them hidden. A platform that cannot hide icons safely must not hide them at all.

Implements the `Drape.Guard` behaviour for each platform that allows icon hiding after its spike: a tiny watchdog that restores the native icon state when the main process dies, plus the `--restore-desktop` path and the next-start recovery. Owned by `shell-engineer-mac` for macOS; the Linux side (only for environments where icon hiding was accepted) is done by `shell-engineer-linux`, and the card is closed only when both are done or the platform's spike ruled hiding out (recorded). Spec first in `MEMORY/specs/P6-07-guard-mac-linux.md`.

**Definition of Done**
- [ ] a restore happens on normal quit, crash, kill, logout and next start, each exercised on a VM or test account
- [ ] `--restore-desktop` works with the main app not running
- [ ] if a platform's spike ruled icon hiding out, the card records that and the code contains no hiding path for it
- [ ] the Guard is minimal (no network, no UI, no user-file access) and survives its parent being killed
- [ ] **QA matrix added:** exit-path x platform recovery matrix in `docs/TESTING/00-TESTING-STRATEGY.md`

**Abuse cases**
- A Guard that restores only when the parent exits cleanly
- Testing restore by restarting the developer's own Finder or file manager

---

### P6-08 — CI matrix and packaging for macOS and Linux

| | |
|---|---|
| **Status** | BLOCKED (waits on P6-05 and P6-06; portable-project CI can start earlier) |
| **Priority** | P1 |
| **Estimate** | 4 days |
| **Role** | release-engineer |
| **Depends on** | P6-05, P6-06 |
| **Spec refs** | docs/DEVOPS/00-BUILD-AND-RELEASE.md · docs/PLATFORM/01-PLATFORM-SUPPORT.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md · ADR-014 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** A platform that is not built and tested on every change rots; and unsigned or un-notarised macOS builds are blocked by the OS.

Extend CI: build and test the portable projects (`Drape.Core`, `Drape.Infrastructure`, `Drape.UI`) on ubuntu and macos runners as well as windows; build the platform shells on their own runners; headless UI tests where possible. Packaging per platform with Velopack (or the Q-04 outcome), a signing and notarisation plan for macOS (hardened runtime, notarisation, key handling in CI), and a Linux packaging decision (AppImage/deb/rpm) recorded in an ADR.

**Definition of Done**
- [ ] CI builds and tests the portable projects on windows, ubuntu and macos; a failure on any blocks merge
- [ ] each shell project builds on its native runner with `-warnaserror`
- [ ] macOS artefacts are signed and notarised in CI with keys stored as CI secrets (none in repo or logs); the plan is documented even if notarisation waits on certificates
- [ ] the update check remains off by default on every platform (ADR-017)
- [ ] **QA matrix added:** the install/upgrade/uninstall rows per platform (including icon restore on uninstall where relevant)

**Abuse cases**
- Marking the matrix green by skipping the tests that cannot run on a runner
- A signing key committed "temporarily" for the first notarisation

---

### P6-09 — Design: mobile scope (what is an item on Android and iOS)

| | |
|---|---|
| **Status** | TODO (starting position is ADR-021; this card details or overturns it) |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | architect |
| **Depends on** | — |
| **Spec refs** | docs/PLATFORM/01-PLATFORM-SUPPORT.md (Android and iOS) · docs/PLAN/02-FEATURE-SPECIFICATION.md · docs/UI-UX/00-UI-UX-SPEC.md · ADR-014, ADR-021 in MEMORY/DECISIONS.md |
| **Spec required** | yes |

**Why:** Mobile has no desktop layer, so "Drape" there is a different product shape. What an item *is* (app shortcut, URL, file, contact) is a product question, and no mobile code is allowed until it is answered.

Detail the decision already taken in ADR-021 (mobile is a companion: named groups of launchable references — Android app/URL/picked file, iOS URL/Shortcut/picked file — with no desktop layer, widgets or sync). Verify it against store policies and platform limits, define storage, permissions, the parts shared with `Drape.Core`/`Drape.UI` and the privacy surface, and decide whether home-screen widgets stay out. The output is a spec in `MEMORY/specs/P6-09-mobile-scope.md` and, if it overturns ADR-021, a new ADR. The answer may still be "out of scope", in which case P6-10 and P6-11 become `DROPPED` with the reason.

**Definition of Done**
- [ ] ADR-021 is confirmed or superseded, with the store-policy checks named (Android package visibility, iOS app-listing limits)
- [ ] the spec defines item types, minimum OS versions, permissions, what is shared with `Drape.Core`/`Drape.UI`, and what is explicitly out of scope
- [ ] the feature-spec and platform-support docs are amended through the deviation protocol
- [ ] **QA matrix added (if in scope):** device classes, OS versions and permission scenarios for Android and iOS

**Abuse cases**
- Defining mobile scope by copying the desktop feature list and marking it "adapted"
- Starting P6-10/P6-11 before this spec exists

---

### P6-10 — Android entry point and shortcut-group UI

| | |
|---|---|
| **Status** | BLOCKED (waits on P6-09) |
| **Priority** | P2 |
| **Estimate** | 5 days |
| **Role** | mobile-engineer |
| **Depends on** | P6-09 |
| **Spec refs** | MEMORY/specs/P6-09-mobile-scope.md · docs/PLATFORM/01-PLATFORM-SUPPORT.md (Android) · docs/UI-UX/00-UI-UX-SPEC.md · ADR-012, ADR-015 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** Android reuses Core and parts of the UI but has no desktop layer; the app is an ordinary shortcut-group manager that must say so honestly.

`Drape.App.Android` entry point (Android 12 / API 31 and later, the range Avalonia supports at Tier 2 or better) with a shortcut-group management UI using shared `Drape.Core` and `Drape.UI` where sensible, and an Android implementation of the platform services. `IPlatformCapabilities` reports no desktop layer, no hide-icons, no global hook.

**Definition of Done**
- [ ] the app builds and runs on API 31 and a current API level; groups and items from the P6-09 spec can be created, edited and persisted
- [ ] desktop-only features are absent or disabled via capabilities, not via OS checks
- [ ] only permissions justified in the P6-09 spec are requested; none at install time without need
- [ ] **QA matrix added:** API levels x screen sizes x orientation x light/dark on at least one physical device and one emulator

**Abuse cases**
- Shipping a desktop Drape UI scaled down and calling it a mobile UI
- Requesting broad storage or accessibility permissions "for later"

---

### P6-11 — iOS entry point and shortcut-group UI

| | |
|---|---|
| **Status** | BLOCKED (waits on P6-09) |
| **Priority** | P2 |
| **Estimate** | 5 days |
| **Role** | mobile-engineer |
| **Depends on** | P6-09 |
| **Spec refs** | MEMORY/specs/P6-09-mobile-scope.md · docs/PLATFORM/01-PLATFORM-SUPPORT.md (iOS) · docs/UI-UX/00-UI-UX-SPEC.md · ADR-012, ADR-015 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** iOS sandboxing is stricter than Android's; what an item can be is limited by the platform, and the UI must not promise more.

`Drape.App.iOS` entry point (iOS 18 and later, the range Avalonia supports at Tier 2 or better) with a shortcut-group management UI using shared `Drape.Core` and `Drape.UI` where sensible, and an iOS implementation of the platform services. `IPlatformCapabilities` reports no desktop layer, no hide-icons, no global hook. Home-screen widgets (WidgetKit) are separate native work and stay out unless P6-09 adds them.

**Definition of Done**
- [ ] the app builds and runs on iOS 18 and a current iOS version; groups and items from the P6-09 spec can be created, edited and persisted
- [ ] desktop-only features are absent or disabled via capabilities, not via OS checks
- [ ] the privacy manifest and permission strings match what the app really does
- [ ] **QA matrix added:** iOS versions x device classes (iPhone, iPad) x orientation x light/dark on at least one physical device and one simulator

**Abuse cases**
- Listing a capability in the store description that the sandbox does not allow
- Claiming iOS support from a simulator-only run

---
