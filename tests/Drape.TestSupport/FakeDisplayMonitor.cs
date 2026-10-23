using Drape.Core;
using Drape.Core.Layout;
using Drape.Core.Model;

namespace Drape.TestSupport;

/// <summary>A display service over a fixed monitor list, using the real resolver.</summary>
public sealed class FakeDisplayMonitor : IDisplayMonitor
{
    private readonly List<MonitorInfo> _monitors;

    /// <summary>Creates a display service.</summary>
    /// <param name="monitors">The monitors to report; at least one is required.</param>
    public FakeDisplayMonitor(IEnumerable<MonitorInfo> monitors)
    {
        _monitors = [.. monitors];
        if (_monitors.Count == 0)
        {
            throw new ArgumentException("At least one monitor is required.", nameof(monitors));
        }
    }

    /// <summary>Creates a single 1920x1080 monitor at 96 DPI.</summary>
    public static FakeDisplayMonitor SingleMonitor() => new(
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
    ]);

    /// <summary>Replaces the monitor list, to simulate a display change.</summary>
    /// <param name="monitors">The new monitors.</param>
    public void SetMonitors(IEnumerable<MonitorInfo> monitors)
    {
        _monitors.Clear();
        _monitors.AddRange(monitors);
    }

    /// <inheritdoc />
    public Task<IReadOnlyList<MonitorInfo>> GetMonitorsAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        return Task.FromResult<IReadOnlyList<MonitorInfo>>(_monitors);
    }

    /// <inheritdoc />
    public BoundsResolution Resolve(DrapeBounds stored, IReadOnlyList<MonitorInfo> monitors)
        => MonitorResolver.Resolve(stored, monitors);
}
