namespace Drape.Core.Model;

/// <summary>
/// Where a Drape sits and how big it is, stored relative to a monitor's work area
/// (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>).
/// </summary>
/// <remarks>
/// <para>
/// The absolute values are in device-independent pixels relative to the monitor's work area. The
/// <c>Fx</c>/<c>Fy</c>/<c>Fw</c>/<c>Fh</c> fractions are the fallback used when the stored monitor is
/// gone: they place the Drape proportionally on whatever monitor is available instead of dropping it
/// at the origin.
/// </para>
/// <para>
/// Always recompute from the stored values, never from the current on-screen position
/// (<c>docs/PLATFORM/00-WINDOWS-INTEGRATION.md</c> §8): repeated scaling otherwise drifts.
/// </para>
/// </remarks>
public readonly record struct DrapeBounds
{
    /// <summary>The smallest width a Drape may have, in DIP.</summary>
    public const double MinWidth = 120;

    /// <summary>The smallest height a Drape may have, in DIP.</summary>
    public const double MinHeight = 80;

    /// <summary>Creates bounds, rejecting sizes below the minimum.</summary>
    /// <param name="monitorId">Stable monitor identifier, never a display index.</param>
    /// <param name="x">Left edge in DIP relative to the monitor work area.</param>
    /// <param name="y">Top edge in DIP relative to the monitor work area.</param>
    /// <param name="width">Width in DIP; must be at least <see cref="MinWidth"/>.</param>
    /// <param name="height">Height in DIP; must be at least <see cref="MinHeight"/>.</param>
    /// <param name="dpi">The monitor DPI the values were recorded at.</param>
    /// <param name="fx">Horizontal position as a fraction of the work area, 0..1.</param>
    /// <param name="fy">Vertical position as a fraction of the work area, 0..1.</param>
    /// <param name="fw">Width as a fraction of the work area, 0..1.</param>
    /// <param name="fh">Height as a fraction of the work area, 0..1.</param>
    /// <exception cref="ArgumentOutOfRangeException">
    /// The size is below the minimum, or a fraction is outside 0..1.
    /// </exception>
    public DrapeBounds(
        string monitorId,
        double x,
        double y,
        double width,
        double height,
        double dpi,
        double fx,
        double fy,
        double fw,
        double fh)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(monitorId);

        if (width < MinWidth)
        {
            throw new ArgumentOutOfRangeException(nameof(width), width, $"A Drape must be at least {MinWidth} DIP wide.");
        }

        if (height < MinHeight)
        {
            throw new ArgumentOutOfRangeException(nameof(height), height, $"A Drape must be at least {MinHeight} DIP tall.");
        }

        if (dpi <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(dpi), dpi, "DPI must be positive.");
        }

        MonitorId = monitorId;
        X = x;
        Y = y;
        Width = width;
        Height = height;
        Dpi = dpi;
        Fx = ValidateFraction(fx, nameof(fx));
        Fy = ValidateFraction(fy, nameof(fy));
        Fw = ValidateFraction(fw, nameof(fw));
        Fh = ValidateFraction(fh, nameof(fh));
    }

    /// <summary>Stable monitor identifier (device name or a hashed DISPLAY_DEVICE id, never an index).</summary>
    public string MonitorId { get; }

    /// <summary>Left edge in DIP relative to the monitor work area.</summary>
    public double X { get; }

    /// <summary>Top edge in DIP relative to the monitor work area.</summary>
    public double Y { get; }

    /// <summary>Width in DIP.</summary>
    public double Width { get; }

    /// <summary>Height in DIP.</summary>
    public double Height { get; }

    /// <summary>The monitor DPI the values were recorded at.</summary>
    public double Dpi { get; }

    /// <summary>Horizontal position as a fraction of the work area, 0..1.</summary>
    public double Fx { get; }

    /// <summary>Vertical position as a fraction of the work area, 0..1.</summary>
    public double Fy { get; }

    /// <summary>Width as a fraction of the work area, 0..1.</summary>
    public double Fw { get; }

    /// <summary>Height as a fraction of the work area, 0..1.</summary>
    public double Fh { get; }

    /// <summary>Returns a copy moved to a new position, keeping the size, DPI and fractions.</summary>
    /// <param name="x">New left edge in DIP.</param>
    /// <param name="y">New top edge in DIP.</param>
    /// <returns>The moved bounds.</returns>
    public DrapeBounds WithPosition(double x, double y) => new(MonitorId, x, y, Width, Height, Dpi, Fx, Fy, Fw, Fh);

    /// <summary>Returns a copy with a new size, rejecting anything below the minimum.</summary>
    /// <param name="width">New width in DIP.</param>
    /// <param name="height">New height in DIP.</param>
    /// <returns>The resized bounds.</returns>
    public DrapeBounds WithSize(double width, double height)
        => new(MonitorId, X, Y, width, height, Dpi, Fx, Fy, Fw, Fh);

    /// <summary>Returns a copy recording a different DPI.</summary>
    /// <param name="dpi">The DPI to record; must be positive.</param>
    /// <returns>The bounds with the new DPI.</returns>
    public DrapeBounds WithDpi(double dpi) => new(MonitorId, X, Y, Width, Height, dpi, Fx, Fy, Fw, Fh);

    /// <summary>Returns a copy recording the fractions, after a move or resize.</summary>
    /// <param name="fx">Horizontal fraction, 0..1.</param>
    /// <param name="fy">Vertical fraction, 0..1.</param>
    /// <param name="fw">Width fraction, 0..1.</param>
    /// <param name="fh">Height fraction, 0..1.</param>
    /// <returns>The bounds with the new fractions.</returns>
    public DrapeBounds WithFractions(double fx, double fy, double fw, double fh)
        => new(MonitorId, X, Y, Width, Height, Dpi, fx, fy, fw, fh);

    /// <summary>Returns a copy on a different monitor, keeping position, size and fractions.</summary>
    /// <param name="monitorId">The new monitor identifier.</param>
    /// <returns>The bounds on the new monitor.</returns>
    public DrapeBounds WithMonitor(string monitorId)
        => new(monitorId, X, Y, Width, Height, Dpi, Fx, Fy, Fw, Fh);

    private static double ValidateFraction(double value, string name)
    {
        if (double.IsNaN(value) || value < 0 || value > 1)
        {
            throw new ArgumentOutOfRangeException(name, value, "A fraction must be between 0 and 1.");
        }

        return value;
    }
}
