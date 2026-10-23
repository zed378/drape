using Drape.Core;
using Drape.Shell.Windows.Dialogs;
using Drape.Shell.Windows.Display;

namespace Drape.Shell.Windows;

/// <summary>
/// Windows implementation of <see cref="IPlatformCapabilities"/>.
/// </summary>
/// <remarks>
/// <para>
/// Every capability reported here is a statement about what this shell project can actually do,
/// not about what the operating system might support in theory. Anything not proven by a spike
/// is reported as unavailable so the UI degrades instead of failing at runtime
/// (<c>TASKS/PHASE-6-CROSS-PLATFORM.md</c>, ADR-015).
/// </para>
/// <para>
/// The desktop-level window host and the icon manager are separate services; this type only
/// reports whether they exist so Core and UI can gate on it without an OS check (CS-21).
/// </para>
/// </remarks>
public sealed class WindowsPlatformCapabilities : IPlatformCapabilities
{
    /// <summary>
    /// Icon hiding uses the shell's own toggle, which is available on every supported Windows
    /// build (Windows 10 22H2 and later, ADR-023).
    /// </summary>
    public bool CanHideDesktopIcons => true;

    /// <summary>
    /// A window can be placed in the desktop layer through the WorkerW re-parenting strategy
    /// under investigation in spike P0-03. Until that spike is accepted (ADR-007 is still
    /// <c>Proposed</c>), this reports <c>false</c>: claiming it would let the UI offer a panel
    /// that has nowhere to render.
    /// </summary>
    public bool SupportsDesktopLevelWindow => false;

    /// <inheritdoc />
    public IReadOnlyList<MonitorInfo> GetMonitors() => MonitorEnumerator.GetMonitors();

    /// <inheritdoc />
    public MonitorInfo GetPrimaryMonitor() => MonitorEnumerator.GetPrimaryMonitor();

    /// <inheritdoc />
    public Task<string?> PickFolderAsync(string? initialPath = null, CancellationToken ct = default)
        => ShellFileDialog.PickFolderAsync(initialPath, ct);

    /// <inheritdoc />
    public Task<IReadOnlyList<string>?> PickFilesAsync(
        string? initialPath = null,
        string? filter = null,
        CancellationToken ct = default)
        => ShellFileDialog.PickFilesAsync(initialPath, filter, ct);
}
