using Drape.Core.Model;
using Shouldly;
using Xunit;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.Core.Tests;

/// <summary>
/// Validation rules on the domain records. Every rejection branch is asserted; a suite that only
/// builds valid objects proves nothing (P1-01 abuse case).
/// </summary>
/// <remarks>
/// The entity is aliased to <c>DrapeEntity</c> per CS-10: inside this namespace the simple name
/// <c>Drape</c> resolves to the root namespace, not to the type.
/// </remarks>
public sealed class DrapeModelTests
{
    private static readonly Guid PageId = Guid.Parse("6f1c2b8e-0000-4000-8000-000000000001");

    private static DrapeBounds Bounds() => new(
        monitorId: @"\\.\DISPLAY1",
        x: 40,
        y: 60,
        width: 320,
        height: 240,
        dpi: 96,
        fx: 0.02,
        fy: 0.05,
        fw: 0.16,
        fh: 0.22);

    private static DrapeItem Item(string path, int order = 0, Guid? id = null) => new(
        id ?? Guid.NewGuid(),
        DrapeItemKind.File,
        path,
        order,
        new DateTimeOffset(2026, 1, 1, 0, 0, 0, TimeSpan.Zero));

    /// <summary>A Drape with an empty title is rejected.</summary>
    /// <param name="title">The blank title under test.</param>
    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Drape_WithBlankTitle_Throws(string title)
    {
        Should.Throw<ArgumentException>(() => new DrapeEntity(Guid.NewGuid(), PageId, title, Bounds()));
    }

    /// <summary>A Drape with an empty id is rejected.</summary>
    [Fact]
    public void Drape_WithEmptyId_Throws()
    {
        Should.Throw<ArgumentException>(() => new DrapeEntity(Guid.Empty, PageId, "Reports", Bounds()));
    }

    /// <summary>A Drape whose page id is empty is rejected.</summary>
    [Fact]
    public void Drape_WithEmptyPageId_Throws()
    {
        Should.Throw<ArgumentException>(() => new DrapeEntity(Guid.NewGuid(), Guid.Empty, "Reports", Bounds()));
    }

    /// <summary>Bounds narrower than the documented minimum are rejected.</summary>
    [Fact]
    public void Bounds_WithWidthBelowMinimum_Throws()
    {
        Should.Throw<ArgumentOutOfRangeException>(() => new DrapeBounds(
            @"\\.\DISPLAY1", 0, 0, DrapeBounds.MinWidth - 1, 200, 96, 0, 0, 0.1, 0.1));
    }

    /// <summary>Bounds shorter than the documented minimum are rejected.</summary>
    [Fact]
    public void Bounds_WithHeightBelowMinimum_Throws()
    {
        Should.Throw<ArgumentOutOfRangeException>(() => new DrapeBounds(
            @"\\.\DISPLAY1", 0, 0, 200, DrapeBounds.MinHeight - 1, 96, 0, 0, 0.1, 0.1));
    }

    /// <summary>Bounds exactly at the minimum are accepted, so the boundary is inclusive.</summary>
    [Fact]
    public void Bounds_AtMinimumSize_IsAccepted()
    {
        var bounds = new DrapeBounds(@"\\.\DISPLAY1", 0, 0, DrapeBounds.MinWidth, DrapeBounds.MinHeight, 96, 0, 0, 0.1, 0.1);

        bounds.Width.ShouldBe(DrapeBounds.MinWidth);
        bounds.Height.ShouldBe(DrapeBounds.MinHeight);
    }

    /// <summary>A fraction outside 0..1 is rejected rather than silently clamped.</summary>
    /// <param name="fraction">The out-of-range fraction under test.</param>
    [Theory]
    [InlineData(-0.1)]
    [InlineData(1.1)]
    public void Bounds_WithFractionOutsideRange_Throws(double fraction)
    {
        Should.Throw<ArgumentOutOfRangeException>(() => new DrapeBounds(
            @"\\.\DISPLAY1", 0, 0, 200, 200, 96, fraction, 0, 0.1, 0.1));
    }

    /// <summary>Two items with the same id are rejected.</summary>
    [Fact]
    public void Drape_WithDuplicateItemIds_Throws()
    {
        var sharedId = Guid.NewGuid();
        var drape = new DrapeEntity(Guid.NewGuid(), PageId, "Reports", Bounds());

        Should.Throw<ArgumentException>(() => drape with
        {
            Items = [Item(@"C:\a.txt", 0, sharedId), Item(@"C:\b.txt", 1, sharedId)],
        });
    }

    /// <summary>The same path twice in one Drape is rejected, case-insensitively.</summary>
    [Fact]
    public void Drape_WithDuplicatePaths_Throws()
    {
        var drape = new DrapeEntity(Guid.NewGuid(), PageId, "Reports", Bounds());

        Should.Throw<ArgumentException>(() => drape with
        {
            Items = [Item(@"C:\A.txt", 0), Item(@"c:\a.txt", 1)],
        });
    }

    /// <summary>Adding an item appends it and assigns the next order.</summary>
    [Fact]
    public void AddItem_AppendsAndAssignsOrder()
    {
        var drape = new DrapeEntity(Guid.NewGuid(), PageId, "Reports", Bounds());

        var withItem = drape.AddItem(Item(@"C:\a.txt"));

        withItem.Items.Count.ShouldBe(1);
        withItem.Items[0].Order.ShouldBe(0);
    }

    /// <summary>Adding an item whose path already exists is rejected.</summary>
    [Fact]
    public void AddItem_WithDuplicatePath_Throws()
    {
        var drape = new DrapeEntity(Guid.NewGuid(), PageId, "Reports", Bounds()).AddItem(Item(@"C:\a.txt"));

        Should.Throw<InvalidOperationException>(() => drape.AddItem(Item(@"C:\A.TXT")));
    }

    /// <summary>Removing an item renumbers the rest, leaving no gaps.</summary>
    [Fact]
    public void RemoveItem_RenumbersRemainingItems()
    {
        var first = Item(@"C:\a.txt");
        var second = Item(@"C:\b.txt", 1);
        var third = Item(@"C:\c.txt", 2);
        var drape = new DrapeEntity(Guid.NewGuid(), PageId, "Reports", Bounds())
        {
            Items = [first, second, third],
        };

        var after = drape.RemoveItem(second.Id);

        after.Items.Count.ShouldBe(2);
        after.Items[0].Path.ShouldBe(@"C:\a.txt");
        after.Items[1].Path.ShouldBe(@"C:\c.txt");
        after.Items[1].Order.ShouldBe(1);
    }

    /// <summary>The item list is a read-only view, not the caller's mutable collection.</summary>
    [Fact]
    public void Items_ExposesReadOnlyView()
    {
        var source = new List<DrapeItem> { Item(@"C:\a.txt") };
        var drape = new DrapeEntity(Guid.NewGuid(), PageId, "Reports", Bounds()) { Items = source };

        source.Add(Item(@"C:\b.txt", 1));

        drape.Items.Count.ShouldBe(1);
    }

    /// <summary>An item with a blank path is rejected.</summary>
    [Fact]
    public void Item_WithBlankPath_Throws()
    {
        Should.Throw<ArgumentException>(() => Item("  "));
    }

    /// <summary>An item with a negative order is rejected.</summary>
    [Fact]
    public void Item_WithNegativeOrder_Throws()
    {
        Should.Throw<ArgumentOutOfRangeException>(() => Item(@"C:\a.txt", -1));
    }

    /// <summary>Marking missing twice keeps the first timestamp, so the 24-hour window is honest.</summary>
    [Fact]
    public void MarkMissing_KeepsFirstTimestamp()
    {
        var item = Item(@"C:\a.txt");
        var first = new DateTimeOffset(2026, 1, 1, 10, 0, 0, TimeSpan.Zero);
        var later = first.AddHours(5);

        var once = item.MarkMissing(first);
        var twice = once.MarkMissing(later);

        twice.MissingSince.ShouldBe(first);
    }

    /// <summary>A config with no pages is rejected.</summary>
    [Fact]
    public void Config_WithNoPages_Throws()
    {
        Should.Throw<ArgumentException>(() => new DrapeConfig(1, []));
    }

    /// <summary>Two pages with the same id are rejected.</summary>
    [Fact]
    public void Config_WithDuplicatePageIds_Throws()
    {
        var page = new Page(PageId, "Main", 0);

        Should.Throw<ArgumentException>(() => new DrapeConfig(1, [page, page]));
    }

    /// <summary>A Drape pointing at a page that does not exist is rejected.</summary>
    [Fact]
    public void Config_WithDrapeOnMissingPage_Throws()
    {
        var drape = new DrapeEntity(Guid.NewGuid(), Guid.NewGuid(), "Reports", Bounds());

        Should.Throw<ArgumentException>(() => new DrapeConfig(1, [new Page(PageId, "Main", 0)], [drape]));
    }

    /// <summary>ADR-011: the same path in two Drapes is rejected.</summary>
    [Fact]
    public void Config_WithItemInTwoDrapes_Throws()
    {
        var page = new Page(PageId, "Main", 0);
        var first = new DrapeEntity(Guid.NewGuid(), PageId, "A", Bounds()) { Items = [Item(@"C:\a.txt")] };
        var second = new DrapeEntity(Guid.NewGuid(), PageId, "B", Bounds()) { Items = [Item(@"C:\a.txt")] };

        var error = Should.Throw<ArgumentException>(() => new DrapeConfig(1, [page], [first, second]));
        error.Message.ShouldContain("ADR-011");
    }

    /// <summary>A schema version of zero or less is rejected.</summary>
    /// <param name="version">The invalid schema version under test.</param>
    [Theory]
    [InlineData(0)]
    [InlineData(-1)]
    public void Config_WithNonPositiveSchemaVersion_Throws(int version)
    {
        Should.Throw<ArgumentOutOfRangeException>(() => new DrapeConfig(version, [new Page(PageId, "Main", 0)]));
    }

    /// <summary>Removing a Drape removes only that Drape.</summary>
    [Fact]
    public void RemoveDrape_RemovesOnlyTheDrape()
    {
        var page = new Page(PageId, "Main", 0);
        var keep = new DrapeEntity(Guid.NewGuid(), PageId, "Keep", Bounds());
        var remove = new DrapeEntity(Guid.NewGuid(), PageId, "Remove", Bounds()) { Items = [Item(@"C:\a.txt")] };
        var config = new DrapeConfig(1, [page], [keep, remove]);

        var after = config.RemoveDrape(remove.Id);

        after.Drapes.Count.ShouldBe(1);
        after.Drapes[0].Title.ShouldBe("Keep");
    }

    /// <summary>Replacing a Drape that does not exist is an error, not a silent insert.</summary>
    [Fact]
    public void ReplaceDrape_WithUnknownId_Throws()
    {
        var config = new DrapeConfig(1, [new Page(PageId, "Main", 0)]);
        var unknown = new DrapeEntity(Guid.NewGuid(), PageId, "X", Bounds());

        Should.Throw<InvalidOperationException>(() => config.ReplaceDrape(unknown));
    }

    /// <summary>Style validation rejects an opacity outside 0..1.</summary>
    [Fact]
    public void Style_WithOpacityOutsideRange_Throws()
    {
        var invalid = DrapeStyle.Default with { Opacity = 1.5 };

        Should.Throw<ArgumentOutOfRangeException>(() => invalid.Validate());
    }

    /// <summary>Style validation rejects an icon size below the supported minimum.</summary>
    [Fact]
    public void Style_WithIconSizeBelowMinimum_Throws()
    {
        var invalid = DrapeStyle.Default with { IconSize = 4 };

        Should.Throw<ArgumentOutOfRangeException>(() => invalid.Validate());
    }

    /// <summary>The default style is valid, so the defaults cannot drift into an invalid state.</summary>
    [Fact]
    public void DefaultStyle_IsValid()
    {
        DrapeStyle.Default.Validate().ShouldBeTrue();
    }

    /// <summary>An icon bitmap whose buffer does not match its dimensions is rejected.</summary>
    [Fact]
    public void IconBitmap_WithMismatchedBuffer_Throws()
    {
        Should.Throw<ArgumentException>(() => new IconBitmap(2, 2, new byte[4]));
    }
}
