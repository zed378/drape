# 01 — Tech Stack

Status: **Accepted.** The cross-platform choice of Avalonia UI was made by the project owner ([ADR-012](../../MEMORY/DECISIONS.md), which supersedes the original WPF assumption in ADR-004).

## Main Choices

| Area | Choice | Alternatives considered | Why |
|---|---|---|---|
| Language | C# | Rust, C++ | High productivity; mature interop on every OS |
| Runtime | .NET 10 LTS | .NET 8 (support ends late 2026) | Newest LTS. Re-verify at P0-01 |
| UI | **Avalonia UI** | .NET MAUI (no Linux), WPF (Windows only), Uno Platform, Tauri | Cross-platform (Windows/macOS/Linux/Android/iOS), WPF-like XAML, access to native window handles, MIT licence |
| MVVM | CommunityToolkit.Mvvm | Prism, ReactiveUI | Light, source generators, Avalonia-compatible |
| DI / hosting | Microsoft.Extensions.Hosting | Autofac | Standard and sufficient |
| Windows P/Invoke | Microsoft.Windows.CsWin32 | Manual P/Invoke, Vanara | Type-safe, generated from official Win32 metadata; only in `Drape.Shell.Windows` |
| macOS interop | P/Invoke into `libobjc` / CoreGraphics | — | `NSWindow`, window levels; only in `Drape.Shell.Mac` |
| Linux interop | X11 binding / GTK interop | — | Desktop window-type hints; only in `Drape.Shell.Linux` |
| Serialisation | System.Text.Json (source-generated) | Newtonsoft.Json | Built in, fast, trim-friendly |
| Logging | Serilog (+ File sink) | Microsoft.Extensions.Logging alone | Rolling files, structured logs |
| Tray | Avalonia `TrayIcon` | Per-platform tray libraries | Cross-platform; verify behaviour per OS in P0-08 |
| Tests | xUnit, NSubstitute, Shouldly | NUnit, Moq, FluentAssertions | Clean licences, widely used |
| UI tests | Avalonia.Headless + FlaUI (Windows) | WinAppDriver | Headless tests for cross-platform UI logic |
| Installer / update | Velopack (Windows/macOS/Linux) | MSIX, WiX, pkg | Per-user install, delta updates, no admin; chosen in ADR-022 (direct distribution; Store/MSIX deferred) |
| Mobile distribution | Google Play + App Store | Sideload | Standard channels (Phase 6) |
| CI | GitHub Actions (windows / macos / ubuntu) | Azure DevOps | Runners for every platform |

## NuGet Packages (initial proposal; lock versions in `Directory.Packages.props`)

| Package | Used in |
|---|---|
| `Avalonia`, `Avalonia.Desktop`, `Avalonia.Themes.Fluent` | UI / App.Desktop |
| `Avalonia.Android`, `Avalonia.iOS` | App.Android / App.iOS (Phase 6) |
| `CommunityToolkit.Mvvm` | UI |
| `Microsoft.Extensions.Hosting` | App |
| `Serilog`, `Serilog.Extensions.Hosting`, `Serilog.Sinks.File` | App, Infrastructure |
| `Microsoft.Windows.CsWin32` | Shell.Windows |
| `xunit`, `xunit.runner.visualstudio`, `NSubstitute`, `Shouldly`, `Microsoft.NET.Test.Sdk` | Tests |
| `Avalonia.Headless.XUnit` | Tests (headless UI) |
| `FlaUI.UIA3` | E2E (Windows) |
| `Meziantou.Analyzer`, `Roslynator.Analyzers` (optional) | All |

Rule: add a new package only through a task or architect approval. Check the licence (MIT/Apache-2.0/BSD) and maintenance activity. Verify current versions on NuGet — do not rely on memory.

## Development Tools

- **IDE:** Visual Studio 2022/2026 (".NET desktop development" workload plus the Avalonia extension), JetBrains Rider, or VS Code + C# Dev Kit. No MAUI workload is needed.
- **Desktop development:** Windows 11 (primary); macOS (Sonoma+) and Ubuntu 22.04+ when Phase 6 starts.
- **VMs:** Hyper-V for Windows; Parallels/UTM for macOS; used for risky scenarios.
- **Windows tools:** Spy++, Inspect.exe, Accessibility Insights, Process Explorer, dotMemory / `dotnet-counters`.
- **macOS tools:** Xcode Instruments, Accessibility Inspector, `lldb`.
- **Linux tools:** GTK Inspector, `xprop`, `xwininfo`, `arandr`.

## Solution Setup

- `global.json` pins the SDK. `Directory.Build.props` sets `Nullable=enable`, `TreatWarningsAsErrors=true`, `LangVersion=latest`, analyzers.
- `Directory.Packages.props` enables Central Package Management.
- `app.manifest` (Windows): `PerMonitorV2`, `longPathAware`, supported OS Windows 10/11, `asInvoker`.
- Target frameworks (verify exact monikers at P0-01):
  - `Drape.Core`, `Drape.Infrastructure`: `net10.0` (platform-agnostic)
  - `Drape.UI`: `net10.0` (Avalonia; no platform APIs)
  - `Drape.Shell.Windows`: `net10.0-windows10.0.19041.0`
  - `Drape.Shell.Mac`: `net10.0-macos` (requires the macOS workload; Phase 6)
  - `Drape.Shell.Linux`: `net10.0` (Phase 6)
  - `Drape.App.Desktop`: how it references the platform Shell projects (multi-target, conditional references, or per-OS heads) is decided in P0-01
  - `Drape.App.Android`: `net10.0-android`; `Drape.App.iOS`: `net10.0-ios` (Phase 6)
  - `Drape.Guard`: tiny; Windows first, other platforms in Phase 6

## Technology Notes

- **Trimming / NativeAOT.** Avalonia supports trimming and NativeAOT on desktop given compiled bindings and no reflection-based XAML loading; evaluate per platform in Phase 5. Do not assume it.
- **Visual effects.** Blur/acrylic differs per platform: DWM/Windows backdrop on Windows, `NSVisualEffectView` on macOS, none in GTK Linux (solid fallback). Wrapped in `IBlurEffect`.
- **Desktop layer.** The core feature is unavailable on Android and iOS; those apps present a regular shortcut-group management UI ([`../PLATFORM/01-PLATFORM-SUPPORT.md`](../PLATFORM/01-PLATFORM-SUPPORT.md)).
- **Wayland.** `wlr-layer-shell` exists on wlroots-based compositors (e.g. Sway) and KDE Plasma, but not stock GNOME. Needs a spike (P6-04).
- **Mobile.** Starts only after the Windows v1.0 ships ([ADR-014](../../MEMORY/DECISIONS.md)).
