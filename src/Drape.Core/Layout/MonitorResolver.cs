using Drape.Core.Model;

namespace Drape.Core.Layout;

/// <summary>
/// Resolves stored bounds against the monitors that exist now, scaling onto a fallback when the
/// stored monitor is gone (P1-10, <c>docs/PLATFORM/00-WINDOWS-INTEGRATION.md</c> §8).
/// </summary>
/// <remarks>
/// The fallback uses the stored fractions rather than the stored absolute position: a Drape that
/// was at 60% across a 2560-wide monitor should stay at 60% across a 1920-wide one, not at a
/// coordinate that may now be off-screen entirely.
/// </remarks>
public static class MonitorResolver
{
    /// <summary>
    /// Resolves stored bounds against the current monitor set.
    /// </summary>
    /// <param name="stored">Bounds read from the config.</param>
    /// <param name="monitors">Monitors that exist now; must not be empty.</param>
    /// <returns>Usable bounds and how they were derived.</returns>
    /// <exception cref="ArgumentException"><paramref name="monitors"/> is empty.</exception>
    public static BoundsResolution Resolve(DrapeBounds stored, IReadOnlyList<MonitorInfo> monitors)
    {
        ArgumentNullException.ThrowIfNull(monitors);

        if (monitors.Count == 0)
        {
            throw new ArgumentException("At least one monitor is required to resolve bounds.", nameof(monitors));
        }

        var target = FindMonitor(stored.MonitorId, monitors);

        if (target is { } exact)
        {
            var workArea = exact.WorkingArea;
            var clamped = LayoutService.Clamp(stored, workArea);
            var resolution = clamped == stored ? MonitorResolution.Exact : MonitorResolution.Clamped;
            return new BoundsResolution(clamped, resolution);
        }

        var fallback = PickFallback(monitors);
        var area = fallback.WorkingArea;

        // Reconstruct from fractions, then clamp: the fractions may still place the Drape partly
        // outside if the aspect ratio changed.
        var scaled = stored
            .WithSize(stored.Fw * area.Width, stored.Fh * area.Height)
            .WithPosition(area.X + (stored.Fx * area.Width), area.Y + (stored.Fy * area.Height));

        var relocated = LayoutService.Clamp(scaled, area);
        return new BoundsResolution(relocated, MonitorResolution.Fallback);
    }

    /// <summary>
    /// Rescales a Drape when the DPI of its monitor changed, so it keeps its apparent physical size.
    /// </summary>
    /// <param name="bounds">Bounds recorded at <see cref="DrapeBounds.Dpi"/>.</param>
    /// <param name="newDpi">The monitor's current DPI.</param>
    /// <param name="workArea">The monitor work area in DIP at the new DPI.</param>
    /// <returns>Bounds scaled to the new DPI and clamped to the work area.</returns>
    public static DrapeBounds Rescale(DrapeBounds bounds, double newDpi, Rectangle workArea)
    {
        if (newDpi <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(newDpi), newDpi, "DPI must be positive.");
        }

        if (bounds.Dpi == newDpi)
        {
            return LayoutService.Clamp(bounds, workArea);
        }

        // DIP are already DPI-independent, so the absolute values do not change with DPI, and
        // neither do the fractions. Only the recorded DPI is updated, and the result is clamped to
        // the new work area, which may be smaller at a higher scale.
        return LayoutService.Clamp(bounds.WithDpi(newDpi), workArea);
    }

    private static MonitorInfo? FindMonitor(string monitorId, IReadOnlyList<MonitorInfo> monitors)
    {
        foreach (var monitor in monitors)
        {
            if (string.Equals(monitor.Id, monitorId, StringComparison.OrdinalIgnoreCase)
                || string.Equals(monitor.DeviceName, monitorId, StringComparison.OrdinalIgnoreCase))
            {
                return monitor;
            }
        }

        return null;
    }

    private static MonitorInfo PickFallback(IReadOnlyList<MonitorInfo> monitors)
    {
        foreach (var monitor in monitors)
        {
            if (monitor.IsPrimary)
            {
                return monitor;
            }
        }

        return monitors[0];
    }
}
