# 01 — Platform Support

Status: **hypotheses, not findings.** Windows is the v1.0 platform ([ADR-014](../../MEMORY/DECISIONS.md)). Everything in the other columns is what each platform is *believed* to allow; each item turns into a fact only when the matching Phase 6 spike records it. Until then no feature promise is made for those platforms.

## Capability Matrix

Capabilities are exposed through `PlatformCapabilities` ([ADR-015](../../MEMORY/DECISIONS.md)); UI and Core test capabilities, never the operating system.

| Capability | Windows | macOS | Linux X11 | Linux Wayland | Android | iOS |
|---|---|---|---|---|---|---|
| Desktop-layer panel (above wallpaper, below apps) | ❓ R0-04, actual Slint host | ❓ P6-01 | ❓ P6-03 | ⚠️ compositor/backend-dependent, P6-04 | ❌ no supported desktop-layer API | ❌ no supported desktop-layer API |
| Hide native desktop icons | ❓ R0-05 | ❓ P6-02 (persistent Finder setting) | ❓ desktop-environment-specific | ❓ desktop-environment-specific | ❌ | ❌ |
| Global mouse hook (quick-hide, right-drag) | ❓ `WH_MOUSE_LL` integration, R0-08 | ❓ permission and API policy | ❓ session/input integration | ❌ no general global hook; compositor-specific alternatives uncommitted | ❌ | ❌ |
| Per-monitor DPI | ❓ native APIs exist, Slint/host validation pending | ❓ Retina integration | ⚠️ varies | ⚠️ varies | n/a | n/a |
| Acrylic / blur | ⚠️ OS/backend/host-dependent, R0-07 | ❓ native vibrancy integration | solid fallback planned | solid fallback planned | n/a | n/a |
| Shell item context menu | ❓ `IContextMenu` integration | ❓ | ❓ file-manager-specific | ❓ | n/a | n/a |
| Autostart | ❓ HKCU `Run` integration | ❓ login item | ❓ XDG autostart | ❓ | n/a | n/a |
| Crash recovery (Guard) | ❓ R0-05/R0-09 | ❓ | ❓ | ❓ | n/a | n/a |
| Global hotkeys | ❓ `RegisterHotKey` integration | ❓ | ❓ | ⚠️ portal/compositor support | n/a | n/a |

Legend: ⚠️ conditional hypothesis; ❌ outside the planned/supported platform API model; ❓ unverified integration. An OS exposing an API is not proof that the Rust/Slint application implements it. No row currently claims validated Rust support.

## Per-Platform Notes

### macOS (Phase 6)
- **Desktop layer.** Window level at the desktop-icon level (via `CGWindowLevelForKey` with the desktop-icon key) and collection behaviour that keeps the window out of Mission Control / Cmd-Tab. Spike P6-01.
- **Native icons.** Finder owns the desktop icons; the known switch is a persistent Finder preference plus a Finder restart, which touches user settings — it would need the same safeguards as ADR-006 (state record, Guard-equivalent, restore on every exit). Spike P6-02 decides whether it is acceptable at all.
- **Hooks.** A `CGEvent` tap needs the user to grant Accessibility/Input Monitoring permission; this is a privacy surface and must be documented ([`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md)).
- **Distribution.** Hardened runtime + notarisation for direct download; the Mac App Store sandbox probably forbids the needed behaviour (review in Phase 6).

### Linux (Phase 6)
- **X11.** Desktop-type or below/skip-taskbar window hints (`_NET_WM_WINDOW_TYPE_DESKTOP` or `_NET_WM_STATE_BELOW`). Spike P6-03.
- **Wayland.** Desktop-layer surfaces need a protocol such as layer-shell plus matching compositor and Slint/backend integration. Availability varies by compositor/version; do not infer support merely from a distribution or desktop name. Probe/document the tested environment in P6-04. Where unavailable, report the capability missing and keep ordinary-window mode explicitly degraded; never hide native icons on an unproved host.
- **Native icons.** Handled by the file manager or a shell extension and differs per desktop environment; likely only supported for a short list of environments, if at all.

### Android and iOS (Phase 6)
- No desktop layer exists. The app is a regular **shortcut-group management** UI sharing `drape-core` and `drape-ui` where sensible.
- What an "item" is on mobile (app shortcut, URL, file, contact) is a product question; design card P6-09 answers it before any mobile code. Until then mobile scope is undefined beyond the sentence above.
- Home-screen widgets (e.g. WidgetKit on iOS) are separate native work, out of scope unless the design card adds them.

## Minimum OS and Distribution (ADR-023)

| Platform | Minimum | Primary channel | Notes (assumptions until the Phase 6 spikes) |
|---|---|---|---|
| Windows | **Windows 10 22H2 (build 19045), x64** — owner decision; Windows 11 is the primary dev target | GitHub Releases: per-user installer + portable zip + winget; evaluate Velopack Rust integration in P5-03/Q-22 | No sunset date set; see PR-9 |
| macOS | macOS 14 Sonoma (Apple silicon first; Intel only if the build is trivial) | Notarised direct download (.dmg/.app) on GitHub Releases, updater integration deferred to Phase 6 validation | Developer ID signing and notarisation need a paid Apple Developer Program membership. Mac App Store is not planned: its sandbox probably forbids Accessibility/Finder manipulation (unverified) |
| Linux | Ubuntu 22.04 LTS and Debian 12 families, x64, as inherited planning targets; X11 and Wayland integration both require spikes | AppImage on GitHub Releases, packaging to validate in Phase 6 | Flatpak/Snap are not planned; actual sandbox/protocol/file-access limitations need validation if reconsidered. A `.deb` may follow |
| Android | Android 12 (API 31) | Google Play, plus a signed APK on GitHub Releases | Needs a developer account; policy checks in P6-09 |
| iOS | iOS 18 | App Store, beta through TestFlight | Needs a paid Apple Developer Program membership; policy checks in P6-09 |

The non-Windows minimums were inherited from ADR-023 and its former Avalonia assumptions. They remain planning targets, not verified Rust/Slint support. Revalidate toolchain, backend, accessibility, ABI and distribution support in Phase 6 before promising any minimum. Windows minimum remains the owner's requirement.

## Sequencing

1. **Phase 0–5:** Windows only, but with the cross-platform seams in place: Core/UI contain no platform code, platform services are interfaces, features are capability-gated, file locations and case rules come from the platform.
2. **Phase 6:** one spike per unknown, an ADR per spike, then the matching `drape-shell-<platform>` implementation, then CI + packaging for that platform. Platforms are independent and can run in parallel with different shell engineers.
3. A platform ships only the capabilities its spike proved. Missing capabilities are reported honestly in settings and the docs, not emulated.

## What Must Already Be True on Windows to Keep This Cheap

- First-party native integration/unsafe FFI stays in `drape-shell-windows` and its minimal Guard recovery adapter. App only selects adapters/bridges approved lifetimes; shared UI/Core do not call native APIs. Reviewed toolkit/watcher dependencies can use their own platform internals; that is distinct from first-party calls (architecture/dependency review).
- Core returns neutral data (`IconBitmap`), not UI-framework types.
- `DesktopLocations`, `Autostart`, `FileSystem` (case rules, atomic replace) and the data directory are injected.
- Strings, paths and keyboard shortcuts avoid Windows-only wording where an OS-neutral one exists ("trash" vs "Recycle Bin" is resolved at the UI-resource level).
