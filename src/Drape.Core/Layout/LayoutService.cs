using Drape.Core.Model;

namespace Drape.Core.Layout;

/// <summary>
/// The arithmetic of moving and resizing a Drape: clamping, minimum size and the visible sliver.
/// </summary>
/// <remarks>
/// <para>
/// This lives in Core so the rules are testable (P1-06). If the view clamped instead, the rules
/// would be untestable and would drift between the drag path and the restore path.
/// </para>
/// <para>
/// All values are DIP relative to a monitor work area. The work area excludes the taskbar, so a
/// Drape clamped to it can never end up under the taskbar.
/// </para>
/// </remarks>
public static class LayoutService
{
    /// <summary>
    /// How much of a Drape must stay inside the work area, in DIP. Without a guaranteed sliver a
    /// Drape dragged past an edge becomes unreachable, because its title bar — the only thing that
    /// can be grabbed — is off-screen.
    /// </summary>
    public const double MinimumVisibleExtent = 40;

    /// <summary>
    /// Clamps a position so at least <see cref="MinimumVisibleExtent"/> of the Drape stays inside the
    /// work area.
    /// </summary>
    /// <param name="x">Desired left edge in DIP.</param>
    /// <param name="y">Desired top edge in DIP.</param>
    /// <param name="width">Drape width in DIP.</param>
    /// <param name="height">Drape height in DIP.</param>
    /// <param name="workArea">The monitor work area in DIP.</param>
    /// <returns>The clamped position.</returns>
    public static (double X, double Y) ClampPosition(
        double x,
        double y,
        double width,
        double height,
        Rectangle workArea)
    {
        var visibleX = Math.Min(MinimumVisibleExtent, width);
        var visibleY = Math.Min(MinimumVisibleExtent, height);

        // The Drape may hang off the left and top edges, but not so far that fewer than the visible
        // sliver remains on screen.
        var minX = workArea.X - width + visibleX;
        var minY = workArea.Y - height + visibleY;
        var maxX = workArea.Right - visibleX;
        var maxY = workArea.Bottom - visibleY;

        // A work area smaller than the sliver would make minX > maxX; prefer the left/top edge so
        // the Drape does not jump to a negative coordinate.
        var clampedX = maxX < minX ? workArea.X : Math.Clamp(x, minX, maxX);
        var clampedY = maxY < minY ? workArea.Y : Math.Clamp(y, minY, maxY);

        return (clampedX, clampedY);
    }

    /// <summary>
    /// Clamps a size to the minimum and to the work area, so a Drape can never be larger than the
    /// screen it is on.
    /// </summary>
    /// <param name="width">Desired width in DIP.</param>
    /// <param name="height">Desired height in DIP.</param>
    /// <param name="workArea">The monitor work area in DIP.</param>
    /// <returns>The clamped size.</returns>
    public static (double Width, double Height) ClampSize(double width, double height, Rectangle workArea)
    {
        var maxWidth = Math.Max(DrapeBounds.MinWidth, workArea.Width);
        var maxHeight = Math.Max(DrapeBounds.MinHeight, workArea.Height);

        return (Math.Clamp(width, DrapeBounds.MinWidth, maxWidth),
                Math.Clamp(height, DrapeBounds.MinHeight, maxHeight));
    }

    /// <summary>
    /// Applies <see cref="ClampSize"/> then <see cref="ClampPosition"/>, in that order: the position
    /// clamp needs the final size to decide how far the Drape may hang off an edge.
    /// </summary>
    /// <param name="bounds">Desired bounds.</param>
    /// <param name="workArea">The monitor work area in DIP.</param>
    /// <returns>Bounds that are entirely valid for the work area.</returns>
    public static DrapeBounds Clamp(DrapeBounds bounds, Rectangle workArea)
    {
        var (width, height) = ClampSize(bounds.Width, bounds.Height, workArea);
        var (x, y) = ClampPosition(bounds.X, bounds.Y, width, height, workArea);
        return bounds.WithSize(width, height).WithPosition(x, y);
    }

    /// <summary>Computes the fractions stored alongside absolute bounds, for the fallback path.</summary>
    /// <param name="x">Left edge in DIP.</param>
    /// <param name="y">Top edge in DIP.</param>
    /// <param name="width">Width in DIP.</param>
    /// <param name="height">Height in DIP.</param>
    /// <param name="workArea">The monitor work area in DIP.</param>
    /// <returns>Fractions clamped to 0..1, safe to divide by.</returns>
    public static (double Fx, double Fy, double Fw, double Fh) ToFractions(
        double x,
        double y,
        double width,
        double height,
        Rectangle workArea)
    {
        // A zero-sized work area would divide by zero; a monitor always has a size, but the fake and
        // a mid-change monitor list might not.
        var areaWidth = workArea.Width > 0 ? workArea.Width : 1;
        var areaHeight = workArea.Height > 0 ? workArea.Height : 1;

        return (ToFraction(x, areaWidth),
                ToFraction(y, areaHeight),
                ToFraction(width, areaWidth),
                ToFraction(height, areaHeight));
    }

    private static double ToFraction(double value, double extent)
        => Math.Clamp(value / extent, 0, 1);
}
