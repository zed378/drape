# 01 — Platform Support

Status: **hypotheses, not findings.** Windows is the v1.0 platform ([ADR-014](../../MEMORY/DECISIONS.md)). Everything in the other columns is what each platform is *believed* to allow; each item turns into a fact only when the matching Phase 6 spike records it. Until then no feature promise is made for those platforms.

## Capability Matrix

Capabilities are exposed through `IPlatformCapabilities` ([ADR-015](../../MEMORY/DECISIONS.md)); UI and Core test capabilities, never the operating system.

| Capability | Windows | macOS | Linux X11 | Linux Wayland | Android | iOS |
|---|---|---|---|---|---|---|
| Desktop-layer panel (above wallpaper, below apps) | ✅ spike P0-03 | ❓ spike P6-01 | ❓ spike P6-03 | ⚠️ compositor-dependent, spike P6-04 | ❌ no concept | ❌ no concept |
| Hide native desktop icons | ✅ spike P0-04 | ❓ spike P6-02 (persistent Finder setting) | ❓ desktop-environment-specific | ❓ desktop-environment-specific | ❌ | ❌ |
| Global mouse hook (quick-hide, right-drag) | ✅ `WH_MOUSE_LL` | ❓ needs user-granted permission | ❓ X11 input extension | ❌ no global hook by design | ❌ | ❌ |
| Per-monitor DPI | ✅ | ✅ (Retina scaling) | ⚠️ varies | ⚠️ varies | n/a | n/a |
| Acrylic / blur | ✅ Windows 10/11 | ✅ vibrancy | ❌ solid fallback | ❌ solid fallback | n/a | n/a |
| Shell item context menu | ✅ `IContextMenu` | ❓ | ❓ file-manager specific | ❓ | n/a | n/a |
| Autostart | ✅ HKCU `Run` | ❓ login item | ❓ XDG autostart | ❓ | n/a | n/a |
| Crash recovery (Guard) | ✅ | ❓ | ❓ | ❓ | n/a | n/a |
| Global hotkeys | ✅ `RegisterHotKey` | ❓ | ❓ | ⚠️ portal-based | n/a | n/a |

Legend: ✅ expected to work, to be verified; ⚠️ partial/conditional; ❌ not possible or not applicable; ❓ unknown until its spike.

## Per-Platform Notes

### macOS (Phase 6)
- **Desktop layer.** Window level at the desktop-icon level (via `CGWindowLevelForKey` with the desktop-icon key) and collection behaviour that keeps the window out of Mission Control / Cmd-Tab. Spike P6-01.
- **Native icons.** Finder owns the desktop icons; the known switch is a persistent Finder preference plus a Finder restart, which touches user settings — it would need the same safeguards as ADR-006 (state record, Guard-equivalent, restore on every exit). Spike P6-02 decides whether it is acceptable at all.
- **Hooks.** A `CGEvent` tap needs the user to grant Accessibility/Input Monitoring permission; this is a privacy surface and must be documented ([`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md)).
- **Distribution.** Hardened runtime + notarisation for direct download; the Mac App Store sandbox probably forbids the needed behaviour (review in Phase 6).

### Linux (Phase 6)
- **X11.** Desktop-type or below/skip-taskbar window hints (`_NET_WM_WINDOW_TYPE_DESKTOP` or `_NET_WM_STATE_BELOW`). Spike P6-03.
- **Wayland.** Needs `wlr-layer-shell`, which wlroots-based compositors and KDE Plasma offer but stock GNOME does not. Where unavailable, the capability is reported as missing and Drape runs as ordinary windows. Spike P6-04.
- **Native icons.** Handled by the file manager or a shell extension and differs per desktop environment; likely only supported for a short list of environments, if at all.

### Android and iOS (Phase 6)
- No desktop layer exists. The app is a regular **shortcut-group management** UI sharing `Drape.Core` and `Drape.UI` where sensible.
- What an "item" is on mobile (app shortcut, URL, file, contact) is a product question; design card P6-09 answers it before any mobile code. Until then mobile scope is undefined beyond the sentence above.
- Home-screen widgets (e.g. WidgetKit on iOS) are separate native work, out of scope unless the design card adds them.

## Minimum OS and Distribution (ADR-023)

| Platform | Minimum | Primary channel | Notes (assumptions until the Phase 6 spikes) |
|---|---|---|---|
| Windows | **Windows 10 22H2 (build 19045), x64** — owner decision; Windows 11 is the primary dev target | GitHub Releases: Velopack per-user installer + portable zip + winget ([ADR-022](../../MEMORY/DECISIONS.md)) | No sunset date set; see PR-9 |
| macOS | macOS 14 Sonoma (Apple silicon first; Intel only if the build is trivial) | Notarised direct download (.dmg/.app) on GitHub Releases, updates via Velopack | Developer ID signing and notarisation need a paid Apple Developer Program membership. Mac App Store is not planned: its sandbox probably forbids Accessibility/Finder manipulation (unverified) |
| Linux | Ubuntu 22.04 LTS and Debian 12 families, x64; X11 session guaranteed, Wayland via capability | AppImage on GitHub Releases via Velopack | Flatpak/Snap are not planned: sandboxing conflicts with desktop-window hints and file access. A `.deb` may follow |
| Android | Android 12 (API 31) | Google Play, plus a signed APK on GitHub Releases | Needs a developer account; policy checks in P6-09 |
| iOS | iOS 18 | App Store, beta through TestFlight | Needs a paid Apple Developer Program membership; policy checks in P6-09 |

The non-Windows minimums are the oldest versions Avalonia lists at Tier 2 or better ([supported platforms](https://docs.avaloniaui.net/docs/supported-platforms)), mirroring the owner's "lowest supported version" choice for Windows. Older versions may work but are untested and unsupported.

## Sequencing

1. **Phase 0–5:** Windows only, but with the cross-platform seams in place: Core/UI contain no platform code, platform services are interfaces, features are capability-gated, file locations and case rules come from the platform.
2. **Phase 6:** one spike per unknown, an ADR per spike, then the matching `Drape.Shell.<Platform>` implementation, then CI + packaging for that platform. Platforms are independent and can run in parallel with different shell engineers.
3. A platform ships only the capabilities its spike proved. Missing capabilities are reported honestly in settings and the docs, not emulated.

## What Must Already Be True on Windows to Keep This Cheap

- No `System.Windows`, `Microsoft.Win32` or P/Invoke outside `Drape.Shell.Windows` / `Drape.Guard` (architecture test).
- Core returns neutral data (`IconBitmap`), not UI-framework types.
- `IDesktopLocations`, `IAutostart`, `IFileSystem` (case rules, atomic replace) and the data directory are injected.
- Strings, paths and keyboard shortcuts avoid Windows-only wording where an OS-neutral one exists ("trash" vs "Recycle Bin" is resolved at the UI-resource level).
