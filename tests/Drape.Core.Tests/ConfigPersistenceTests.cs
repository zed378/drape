using Drape.Core.Model;
using Drape.Core.Persistence;
using Drape.TestSupport;
using Shouldly;
using Xunit;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.Core.Tests;

/// <summary>
/// The debounce and shutdown-flush behaviour from P1-11. The clock is fake, so no test sleeps and
/// the save path actually runs (the abuse case in the card is a clock that never advances).
/// </summary>
public sealed class ConfigPersistenceTests
{
    private static readonly Guid _pageId = Guid.Parse("6f1c2b8e-0000-4000-8000-000000000001");

    private static DrapeConfig ConfigWithTitle(string title)
    {
        var drape = new DrapeEntity(
            Guid.Parse("6f1c2b8e-0000-4000-8000-000000000010"),
            _pageId,
            title,
            new DrapeBounds(@"\\.\DISPLAY1", 40, 60, 320, 240, 96, 0.02, 0.05, 0.16, 0.22));

        return new DrapeConfig(1, [new Page(_pageId, "Main", 0)], [drape]);
    }

    /// <summary>Nothing is saved before the debounce interval elapses.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task SaveIfDue_BeforeInterval_DoesNotSave()
    {
        var store = new FakeConfigStore();
        var clock = new FakeClock();
        var persistence = new ConfigPersistence(store, clock);

        persistence.Queue(ConfigWithTitle("A"));
        clock.AdvanceMilliseconds(400);

        (await persistence.SaveIfDueAsync()).ShouldBeFalse();
        store.SaveCount.ShouldBe(0);
    }

    /// <summary>The document is saved once the interval elapses.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task SaveIfDue_AfterInterval_Saves()
    {
        var store = new FakeConfigStore();
        var clock = new FakeClock();
        var persistence = new ConfigPersistence(store, clock);

        persistence.Queue(ConfigWithTitle("A"));
        clock.AdvanceMilliseconds(500);

        (await persistence.SaveIfDueAsync()).ShouldBeTrue();
        store.SaveCount.ShouldBe(1);
        store.LastSaved!.Drapes[0].Title.ShouldBe("A");
    }

    /// <summary>A second queue within the interval resets it and only the last document is written.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task Queue_TwiceWithinInterval_WritesOnlyTheLast()
    {
        var store = new FakeConfigStore();
        var clock = new FakeClock();
        var persistence = new ConfigPersistence(store, clock);

        persistence.Queue(ConfigWithTitle("First"));
        clock.AdvanceMilliseconds(300);
        persistence.Queue(ConfigWithTitle("Second"));
        clock.AdvanceMilliseconds(300);

        (await persistence.SaveIfDueAsync()).ShouldBeFalse();
        clock.AdvanceMilliseconds(200);
        (await persistence.SaveIfDueAsync()).ShouldBeTrue();

        store.SaveCount.ShouldBe(1);
        store.LastSaved!.Drapes[0].Title.ShouldBe("Second");
    }

    /// <summary>
    /// A change made just before shutdown is persisted by the flush. This is the named test the
    /// card asks for.
    /// </summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task Flush_BeforeDebounceElapses_StillSaves()
    {
        var store = new FakeConfigStore();
        var clock = new FakeClock();
        var persistence = new ConfigPersistence(store, clock);

        persistence.Queue(ConfigWithTitle("Unsaved"));
        clock.AdvanceMilliseconds(10);

        (await persistence.FlushAsync()).ShouldBeTrue();
        store.SaveCount.ShouldBe(1);
        store.LastSaved!.Drapes[0].Title.ShouldBe("Unsaved");
    }

    /// <summary>Flushing with nothing pending does not write.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task Flush_WithNothingPending_DoesNotSave()
    {
        var store = new FakeConfigStore();
        var persistence = new ConfigPersistence(store, new FakeClock());

        (await persistence.FlushAsync()).ShouldBeFalse();
        store.SaveCount.ShouldBe(0);
    }

    /// <summary>After a successful save the pending flag clears.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task SaveIfDue_ClearsPendingFlag()
    {
        var store = new FakeConfigStore();
        var clock = new FakeClock();
        var persistence = new ConfigPersistence(store, clock);

        persistence.Queue(ConfigWithTitle("A"));
        persistence.HasPendingChanges.ShouldBeTrue();

        clock.AdvanceMilliseconds(500);
        await persistence.SaveIfDueAsync();

        persistence.HasPendingChanges.ShouldBeFalse();
    }

    /// <summary>A failed save propagates instead of being swallowed (CS-14).</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task Flush_WhenStoreFails_Throws()
    {
        var store = new FakeConfigStore { FailSave = true };
        var persistence = new ConfigPersistence(store, new FakeClock());

        persistence.Queue(ConfigWithTitle("A"));

        await Should.ThrowAsync<IOException>(() => persistence.FlushAsync());
    }
}
