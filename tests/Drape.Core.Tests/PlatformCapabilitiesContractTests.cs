using Drape.Core;
using Drape.TestSupport;
using Shouldly;
using Xunit;

namespace Drape.Core.Tests;

/// <summary>
/// Contract expectations that every <see cref="IPlatformCapabilities"/> implementation owes its
/// callers, checked against the shared fake so the rules are written down once.
/// </summary>
public sealed class PlatformCapabilitiesContractTests
{
    /// <summary>The primary monitor is the one the platform flags, not simply the first one.</summary>
    [Fact]
    public void GetPrimaryMonitor_ReturnsMonitorFlaggedPrimary()
    {
        var capabilities = new FakePlatformCapabilities(
        [
            new MonitorInfo("display-2", @"\\.\DISPLAY2", new Rectangle(1920, 0, 1280, 1024), new Rectangle(1920, 0, 1280, 984), 96, 96, 1.0, IsPrimary: false),
            new MonitorInfo("display-1", @"\\.\DISPLAY1", new Rectangle(0, 0, 1920, 1080), new Rectangle(0, 0, 1920, 1040), 144, 144, 1.5, IsPrimary: true),
        ]);

        var primary = capabilities.GetPrimaryMonitor();

        primary.Id.ShouldBe("display-1");
        primary.ScaleFactor.ShouldBe(1.5);
    }

    /// <summary>Every attached monitor is reported, including ones with a different DPI.</summary>
    [Fact]
    public void GetMonitors_ReturnsEveryConfiguredMonitor()
    {
        var capabilities = new FakePlatformCapabilities(
        [
            new MonitorInfo("display-1", @"\\.\DISPLAY1", new Rectangle(0, 0, 1920, 1080), new Rectangle(0, 0, 1920, 1040), 96, 96, 1.0, IsPrimary: true),
            new MonitorInfo("display-2", @"\\.\DISPLAY2", new Rectangle(1920, 0, 2560, 1440), new Rectangle(1920, 0, 2560, 1400), 192, 192, 2.0, IsPrimary: false),
        ]);

        capabilities.GetMonitors().Count.ShouldBe(2);
    }

    /// <summary>A platform that flags nothing still has to name a primary monitor.</summary>
    [Fact]
    public void GetPrimaryMonitor_WithNoPrimaryFlagged_FallsBackToFirstMonitor()
    {
        var capabilities = new FakePlatformCapabilities(
        [
            new MonitorInfo("display-1", @"\\.\DISPLAY1", new Rectangle(0, 0, 800, 600), new Rectangle(0, 0, 800, 560), 96, 96, 1.0, IsPrimary: false),
        ]);

        capabilities.GetPrimaryMonitor().Id.ShouldBe("display-1");
    }

    /// <summary>An empty monitor list is a contract violation, not an empty session.</summary>
    [Fact]
    public void Constructor_WithEmptyMonitorList_Throws()
    {
        Should.Throw<ArgumentException>(() => new FakePlatformCapabilities([]));
    }

    /// <summary>Pickers honour cancellation instead of opening a dialog.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task PickFolderAsync_CancelledToken_Throws()
    {
        var capabilities = new FakePlatformCapabilities();
        using var cts = new CancellationTokenSource();
        await cts.CancelAsync();

        await Should.ThrowAsync<OperationCanceledException>(
            () => capabilities.PickFolderAsync(@"C:\Users", cts.Token));
    }

    /// <summary>The initial path reaches the picker and the chosen paths come back unchanged.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task PickFilesAsync_RecordsInitialPathAndReturnsResult()
    {
        var capabilities = new FakePlatformCapabilities
        {
            FilePickResult = [@"C:\Users\Zed\Documents\a.txt"],
        };

        var result = await capabilities.PickFilesAsync(@"C:\Users\Zed\Documents", "*.txt", CancellationToken.None);

        result.ShouldBe([@"C:\Users\Zed\Documents\a.txt"]);
        capabilities.PickerCalls.ShouldBe([@"C:\Users\Zed\Documents"]);
    }

    /// <summary>The fake's defaults must describe a platform that offers nothing extra.</summary>
    [Fact]
    public void Defaults_DescribeAPlainPlatform()
    {
        var capabilities = new FakePlatformCapabilities();

        capabilities.CanHideDesktopIcons.ShouldBeFalse();
        capabilities.SupportsDesktopLevelWindow.ShouldBeFalse();
        capabilities.GetMonitors().Count.ShouldBe(1);
        capabilities.GetMonitors()[0].ScaleFactor.ShouldBe(1.0);
    }
}
