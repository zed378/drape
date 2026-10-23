using Drape.Core;
using Drape.Core.Layout;
using Drape.Core.Model;
using Shouldly;
using Xunit;

namespace Drape.Core.Tests;

/// <summary>
/// The clamp, minimum-size and visible-sliver rules from P1-06, plus the monitor resolution from
/// P1-10. Every failure branch is covered.
/// </summary>
public sealed class LayoutServiceTests
{
    private static readonly Rectangle _workArea = new(0, 0, 1920, 1040);

    /// <summary>A position well inside the work area is left alone.</summary>
    [Fact]
    public void ClampPosition_InsideWorkArea_IsUnchanged()
    {
        var (x, y) = LayoutService.ClampPosition(100, 100, 300, 200, _workArea);

        x.ShouldBe(100);
        y.ShouldBe(100);
    }

    /// <summary>A Drape dragged past the right edge keeps the 40 DIP sliver visible.</summary>
    [Fact]
    public void ClampPosition_PastRightEdge_KeepsVisibleSliver()
    {
        var (x, _) = LayoutService.ClampPosition(5000, 100, 300, 200, _workArea);

        x.ShouldBe(_workArea.Right - LayoutService.MinimumVisibleExtent);
    }

    /// <summary>A Drape dragged past the bottom edge keeps the sliver visible.</summary>
    [Fact]
    public void ClampPosition_PastBottomEdge_KeepsVisibleSliver()
    {
        var (_, y) = LayoutService.ClampPosition(100, 5000, 300, 200, _workArea);

        y.ShouldBe(_workArea.Bottom - LayoutService.MinimumVisibleExtent);
    }

    /// <summary>
    /// A Drape dragged far past the left edge may hang off it, but not entirely: the sliver stays.
    /// </summary>
    [Fact]
    public void ClampPosition_PastLeftEdge_KeepsVisibleSliver()
    {
        var (x, _) = LayoutService.ClampPosition(-5000, 100, 300, 200, _workArea);

        x.ShouldBe(_workArea.X - 300 + LayoutService.MinimumVisibleExtent);
    }

    /// <summary>
    /// A work area narrower than the sliver must not produce a coordinate outside the work area.
    /// </summary>
    [Fact]
    public void ClampPosition_WithTinyWorkArea_DoesNotEscape()
    {
        var tiny = new Rectangle(0, 0, 20, 20);

        var (x, y) = LayoutService.ClampPosition(500, 500, 300, 200, tiny);

        x.ShouldBe(0);
        y.ShouldBe(0);
    }

    /// <summary>A size below the minimum is raised to the minimum.</summary>
    [Fact]
    public void ClampSize_BelowMinimum_RaisesToMinimum()
    {
        var (width, height) = LayoutService.ClampSize(10, 10, _workArea);

        width.ShouldBe(DrapeBounds.MinWidth);
        height.ShouldBe(DrapeBounds.MinHeight);
    }

    /// <summary>A size larger than the work area is reduced to the work area.</summary>
    [Fact]
    public void ClampSize_LargerThanWorkArea_IsReducedToWorkArea()
    {
        var (width, height) = LayoutService.ClampSize(5000, 5000, _workArea);

        width.ShouldBe(_workArea.Width);
        height.ShouldBe(_workArea.Height);
    }

    /// <summary>Clamp applies the size first, so the position uses the final width.</summary>
    [Fact]
    public void Clamp_ShrinksThenRepositions()
    {
        var bounds = new DrapeBounds(@"\\.\DISPLAY1", 1900, 1030, 5000, 5000, 96, 0.9, 0.9, 0.9, 0.9);

        var clamped = LayoutService.Clamp(bounds, _workArea);

        clamped.Width.ShouldBe(_workArea.Width);
        clamped.Height.ShouldBe(_workArea.Height);
        clamped.X.ShouldBe(_workArea.Right - LayoutService.MinimumVisibleExtent);
    }

    /// <summary>Fractions are derived from the work area and stay within 0..1.</summary>
    [Fact]
    public void ToFractions_ComputesProportions()
    {
        var (fx, fy, fw, fh) = LayoutService.ToFractions(960, 520, 192, 104, _workArea);

        fx.ShouldBe(0.5, 0.0001);
        fy.ShouldBe(0.5, 0.0001);
        fw.ShouldBe(0.1, 0.0001);
        fh.ShouldBe(0.1, 0.0001);
    }

    /// <summary>A zero-sized work area does not divide by zero.</summary>
    [Fact]
    public void ToFractions_WithZeroWorkArea_DoesNotThrow()
    {
        var (fx, fy, fw, fh) = LayoutService.ToFractions(10, 10, 10, 10, new Rectangle(0, 0, 0, 0));

        fx.ShouldBe(1);
        fw.ShouldBe(1);
        fy.ShouldBe(1);
        fh.ShouldBe(1);
    }

    private static MonitorInfo Monitor(string id, double x, double y, double w, double h, bool primary = false)
        => new(id, id, new Rectangle(x, y, w, h), new Rectangle(x, y, w, h - 40), 96, 96, 1.0, primary);

    /// <summary>When the stored monitor exists, the bounds are used as-is.</summary>
    [Fact]
    public void Resolve_WithMonitorPresent_ReturnsExact()
    {
        var monitors = new[] { Monitor(@"\\.\DISPLAY1", 0, 0, 1920, 1080, primary: true) };
        var stored = new DrapeBounds(@"\\.\DISPLAY1", 100, 100, 300, 200, 96, 0.05, 0.05, 0.15, 0.15);

        var result = MonitorResolver.Resolve(stored, monitors);

        result.Resolution.ShouldBe(MonitorResolution.Exact);
        result.Bounds.X.ShouldBe(100);
    }

    /// <summary>When the stored monitor is gone, the fractions place the Drape on the fallback.</summary>
    [Fact]
    public void Resolve_WithMonitorMissing_ScalesOntoFallback()
    {
        // The helper's work area is 1000 wide and 960 tall (a 40 DIP taskbar is subtracted), so a
        // 0.5 fraction lands at x=500, y=480.
        var monitors = new[] { Monitor(@"\\.\DISPLAY2", 0, 0, 1000, 1000, primary: true) };
        var stored = new DrapeBounds(@"\\.\DISPLAY1", 100, 100, 300, 200, 96, 0.5, 0.5, 0.2, 0.2);

        var result = MonitorResolver.Resolve(stored, monitors);

        result.Resolution.ShouldBe(MonitorResolution.Fallback);
        result.Bounds.X.ShouldBe(500, 0.001);
        result.Bounds.Y.ShouldBe(480, 0.001);
        result.Bounds.Width.ShouldBe(200, 0.001);
    }

    /// <summary>
    /// A shrunk work area clamps the Drape so the visible sliver stays on screen. The Drape is
    /// deliberately allowed to hang off the right edge; requiring it to fit entirely would break
    /// the sliver rule that makes an oversized Drape reachable.
    /// </summary>
    [Fact]
    public void Resolve_WithShrunkWorkArea_ReturnsClamped()
    {
        var monitors = new[] { Monitor(@"\\.\DISPLAY1", 0, 0, 800, 600, primary: true) };
        var stored = new DrapeBounds(@"\\.\DISPLAY1", 1500, 900, 300, 200, 96, 0.8, 0.8, 0.2, 0.2);

        var result = MonitorResolver.Resolve(stored, monitors);

        result.Resolution.ShouldBe(MonitorResolution.Clamped);

        // At least the visible sliver of the Drape must remain inside the work area.
        (result.Bounds.X + result.Bounds.Width - 800).ShouldBeLessThanOrEqualTo(300 - LayoutService.MinimumVisibleExtent);
    }

    /// <summary>Resolving against no monitors is an error, not a silent empty result.</summary>
    [Fact]
    public void Resolve_WithNoMonitors_Throws()
    {
        var stored = new DrapeBounds(@"\\.\DISPLAY1", 0, 0, 300, 200, 96, 0, 0, 0.1, 0.1);

        Should.Throw<ArgumentException>(() => MonitorResolver.Resolve(stored, []));
    }

    /// <summary>Ten resolution cycles must not drift the position.</summary>
    [Fact]
    public void Resolve_TenCycles_DoesNotDrift()
    {
        var stored = new DrapeBounds(@"\\.\DISPLAY1", 100, 100, 300, 200, 96, 0.052083, 0.096154, 0.15625, 0.192308);
        var monitors = new[] { Monitor(@"\\.\DISPLAY1", 0, 0, 1920, 1040, primary: true) };

        var current = stored;
        for (var i = 0; i < 10; i++)
        {
            current = MonitorResolver.Resolve(current, monitors).Bounds;
        }

        current.X.ShouldBe(100, 0.0001);
        current.Y.ShouldBe(100, 0.0001);
        current.Width.ShouldBe(300, 0.0001);
    }

    /// <summary>A DPI change updates the recorded DPI without moving the Drape in DIP.</summary>
    [Fact]
    public void Rescale_WithNewDpi_KeepsDipPosition()
    {
        var bounds = new DrapeBounds(@"\\.\DISPLAY1", 100, 100, 300, 200, 96, 0.05, 0.05, 0.15, 0.15);

        var rescaled = MonitorResolver.Rescale(bounds, 144, new Rectangle(0, 0, 1280, 680));

        rescaled.Dpi.ShouldBe(144);
        rescaled.X.ShouldBe(100);
    }

    /// <summary>A non-positive DPI is rejected.</summary>
    [Fact]
    public void Rescale_WithZeroDpi_Throws()
    {
        var bounds = new DrapeBounds(@"\\.\DISPLAY1", 100, 100, 300, 200, 96, 0.05, 0.05, 0.15, 0.15);

        Should.Throw<ArgumentOutOfRangeException>(() => MonitorResolver.Rescale(bounds, 0, _workArea));
    }
}
