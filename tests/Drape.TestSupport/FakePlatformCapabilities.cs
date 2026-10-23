using Drape.Core;

namespace Drape.TestSupport;

/// <summary>
/// In-memory <see cref="IPlatformCapabilities"/> for tests.
/// </summary>
/// <remarks>
/// Defaults describe a platform with nothing special: no desktop-level window, no icon hiding,
/// one 1920x1080 monitor at 96 DPI. Tests override only what they care about, which keeps the
/// arrange block honest about the capability under test (CS-44).
/// </remarks>
public sealed class FakePlatformCapabilities : IPlatformCapabilities
{
    private readonly List<MonitorInfo> _monitors;

    /// <summary>Creates a fake with a single 1920x1080 primary monitor at 100% scaling.</summary>
    public FakePlatformCapabilities()
        : this(
        [
            new MonitorInfo(
                Id: @"\\.\DISPLAY1",
                DeviceName: @"\\.\DISPLAY1",
                Bounds: new Rectangle(0, 0, 1920, 1080),
                WorkingArea: new Rectangle(0, 0, 1920, 1040),
                DpiX: 96,
                DpiY: 96,
                ScaleFactor: 1.0,
                IsPrimary: true),
        ])
    {
    }

    /// <summary>Creates a fake over an explicit monitor list.</summary>
    /// <param name="monitors">Monitors to report; must contain at least one.</param>
    public FakePlatformCapabilities(IEnumerable<MonitorInfo> monitors)
    {
        ArgumentNullException.ThrowIfNull(monitors);
        _monitors = [.. monitors];
        if (_monitors.Count == 0)
        {
            throw new ArgumentException("A platform always reports at least one monitor.", nameof(monitors));
        }
    }

    /// <inheritdoc />
    public bool CanHideDesktopIcons { get; init; }

    /// <inheritdoc />
    public bool SupportsDesktopLevelWindow { get; init; }

    /// <summary>Path returned by <see cref="PickFolderAsync"/>; <c>null</c> simulates a cancel.</summary>
    public string? FolderPickResult { get; init; }

    /// <summary>Paths returned by <see cref="PickFilesAsync"/>; <c>null</c> simulates a cancel.</summary>
    public IReadOnlyList<string>? FilePickResult { get; init; }

    /// <summary>Records every initial path passed to a picker, so tests can assert on it.</summary>
    public List<string?> PickerCalls { get; } = [];

    /// <inheritdoc />
    public IReadOnlyList<MonitorInfo> GetMonitors() => _monitors;

    /// <inheritdoc />
    public MonitorInfo GetPrimaryMonitor() => _monitors.FirstOrDefault(m => m.IsPrimary, _monitors[0]);

    /// <inheritdoc />
    public Task<string?> PickFolderAsync(string? initialPath = null, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        PickerCalls.Add(initialPath);
        return Task.FromResult(FolderPickResult);
    }

    /// <inheritdoc />
    public Task<IReadOnlyList<string>?> PickFilesAsync(
        string? initialPath = null,
        string? filter = null,
        CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        PickerCalls.Add(initialPath);
        return Task.FromResult(FilePickResult);
    }
}
