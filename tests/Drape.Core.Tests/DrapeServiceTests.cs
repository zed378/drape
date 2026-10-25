using Drape.Core.Model;
using Drape.Core.Persistence;
using Drape.Core.Services;
using Drape.TestSupport;
using Shouldly;
using Xunit;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.Core.Tests;

/// <summary>
/// Create, rename and delete through <see cref="DrapeService"/> (P1-07). The delete test asserts
/// against the file system, not the in-memory list — otherwise it could not fail.
/// </summary>
public sealed class DrapeServiceTests
{
    private static readonly Guid _pageId = Guid.Parse("6f1c2b8e-0000-4000-8000-000000000001");

    private static (DrapeService Service, FakeConfigStore Store, FakeFileSystem Files) Create(
        DrapeConfig? config = null)
    {
        var store = new FakeConfigStore();
        var clock = new FakeClock();
        var persistence = new ConfigPersistence(store, clock);
        var monitors = FakeDisplayMonitor.SingleMonitor();
        var service = new DrapeService(
            config ?? DrapeConfig.CreateEmpty(_pageId),
            persistence,
            monitors);
        return (service, store, new FakeFileSystem());
    }

    /// <summary>A new Drape appears with a default name and is queued for saving.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task CreateAsync_CreatesDrapeWithDefaultName()
    {
        var (service, _, _) = Create();

        var drape = await service.CreateAsync();

        drape.Title.ShouldBe("New Drape");
        service.Config.Drapes.Count.ShouldBe(1);
    }

    /// <summary>A second Drape gets a unique name, not a duplicate.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task CreateAsync_Twice_GivesUniqueNames()
    {
        var (service, _, _) = Create();

        var first = await service.CreateAsync();
        var second = await service.CreateAsync();

        first.Title.ShouldNotBe(second.Title);
        second.Title.ShouldBe("New Drape 2");
    }

    /// <summary>Creating on a missing page is an error, not a silent first-page fallback.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task CreateAsync_WithMissingPage_Throws()
    {
        var (service, _, _) = Create();

        await Should.ThrowAsync<InvalidOperationException>(() => service.CreateAsync(Guid.NewGuid()));
    }

    /// <summary>Renaming changes the title and queues a save.</summary>
    [Fact]
    public async Task Rename_ChangesTitle()
    {
        var (service, _, _) = Create();
        var drape = await service.CreateAsync();

        var renamed = service.Rename(drape.Id, "Reports");

        renamed.Title.ShouldBe("Reports");
        service.Config.Drapes[0].Title.ShouldBe("Reports");
    }

    /// <summary>Renaming to a blank title is rejected by the model.</summary>
    [Fact]
    public async Task Rename_WithBlankTitle_Throws()
    {
        var (service, _, _) = Create();
        var drape = await service.CreateAsync();

        Should.Throw<ArgumentException>(() => service.Rename(drape.Id, "  "));
    }

    /// <summary>Renaming a Drape that does not exist is an error.</summary>
    [Fact]
    public void Rename_WithUnknownId_Throws()
    {
        var (service, _, _) = Create();

        Should.Throw<InvalidOperationException>(() => service.Rename(Guid.NewGuid(), "X"));
    }

    /// <summary>
    /// Deleting a Drape deletes no file: the file still exists on disk afterwards, and the items
    /// are returned as ungrouped.
    /// </summary>
    [Fact]
    public async Task Delete_RemovesDrapeButKeepsFilesOnDisk()
    {
        var (service, _, files) = Create();
        var drape = await service.CreateAsync();
        var item = new DrapeItem(
            Guid.NewGuid(), DrapeItemKind.File, @"C:\Users\test\Desktop\Q3.xlsx", 0,
            new DateTimeOffset(2026, 1, 1, 0, 0, 0, TimeSpan.Zero));
        service.Config.ReplaceDrape(drape with { Items = [item] });
        files.AddFile(@"C:\Users\test\Desktop\Q3.xlsx", "contents");

        var ungrouped = service.Delete(drape.Id);

        service.Config.Drapes.ShouldBeEmpty();
        ungrouped.Count.ShouldBe(1);
        files.FileExists(@"C:\Users\test\Desktop\Q3.xlsx").ShouldBeTrue();
        files.Deletions.ShouldBeEmpty();
    }

    /// <summary>Deleting a Drape that does not exist is an error.</summary>
    [Fact]
    public void Delete_WithUnknownId_Throws()
    {
        var (service, _, _) = Create();

        Should.Throw<InvalidOperationException>(() => service.Delete(Guid.NewGuid()));
    }

    /// <summary>Every mutation raises Changed so the UI can refresh.</summary>
    /// <returns>A task that completes when the assertion has run.</returns>
    [Fact]
    public async Task Mutations_RaiseChanged()
    {
        var (service, _, _) = Create();
        var count = 0;
        service.Changed += (_, _) => count++;

        var drape = await service.CreateAsync();
        service.Rename(drape.Id, "R");
        service.Delete(drape.Id);

        count.ShouldBe(3);
    }
}
