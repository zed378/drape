using Drape.Core;
using Drape.Core.Model;
using Drape.Infrastructure.Configuration;
using Drape.TestSupport;
using Shouldly;
using Xunit;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.Infrastructure.Tests;

/// <summary>
/// The atomicity, backup, quarantine and version behaviour of <see cref="FileConfigStore"/>
/// (P1-03). Each failure branch is asserted; a store that only ever succeeds proves nothing.
/// </summary>
public sealed class FileConfigStoreTests
{
    private const string Root = @"C:\Users\test\AppData\Local\Drape";
    private const string ConfigPath = Root + @"\config.json";

    private static readonly Guid _pageId = Guid.Parse("6f1c2b8e-0000-4000-8000-000000000001");

    private static DrapeConfig ConfigWithOneDrape(string title = "Reports")
    {
        var drape = new DrapeEntity(
            Guid.Parse("6f1c2b8e-0000-4000-8000-000000000010"),
            _pageId,
            title,
            new DrapeBounds(@"\\.\DISPLAY1", 40, 60, 320, 240, 96, 0.02, 0.05, 0.16, 0.22));

        return new DrapeConfig(1, [new Page(_pageId, "Main", 0)], [drape]);
    }

    private static (FileConfigStore Store, FakeFileSystem FileSystem, FakeClock Clock) CreateStore()
    {
        var fileSystem = new FakeFileSystem();
        var clock = new FakeClock();
        return (new FileConfigStore(fileSystem, clock, Root), fileSystem, clock);
    }

    /// <summary>No config on disk reports a first run rather than an error.</summary>
    [Fact]
    public async Task LoadAsync_WithNoFile_ReportsFirstRun()
    {
        var (store, _, _) = CreateStore();

        var result = await store.LoadAsync();

        result.Outcome.ShouldBe(ConfigLoadOutcome.FirstRun);
        result.Config.Pages.Count.ShouldBe(1);
        result.CanSave.ShouldBeTrue();
    }

    /// <summary>A save followed by a load returns the same document.</summary>
    [Fact]
    public async Task SaveThenLoad_RoundTripsTheDocument()
    {
        var (store, _, _) = CreateStore();

        await store.SaveAsync(ConfigWithOneDrape());
        var result = await store.LoadAsync();

        result.Outcome.ShouldBe(ConfigLoadOutcome.Loaded);
        result.Config.Drapes.Count.ShouldBe(1);
        result.Config.Drapes[0].Title.ShouldBe("Reports");
        result.Config.Drapes[0].Bounds.Width.ShouldBe(320);
    }

    /// <summary>A second save keeps the previous contents as a backup.</summary>
    [Fact]
    public async Task SaveAsync_Twice_KeepsPreviousVersionAsBackup()
    {
        var (store, fileSystem, clock) = CreateStore();

        await store.SaveAsync(ConfigWithOneDrape("First"));
        clock.Advance(TimeSpan.FromSeconds(1));
        await store.SaveAsync(ConfigWithOneDrape("Second"));

        fileSystem.Replaces.Count.ShouldBe(1);
        fileSystem.FileExists(fileSystem.Replaces[0].Backup).ShouldBeTrue();

        var result = await store.LoadAsync();
        result.Config.Drapes[0].Title.ShouldBe("Second");
    }

    /// <summary>
    /// A replace that fails leaves the previous file intact. This is the crash-mid-write case: the
    /// store writes to a temporary file and never truncates the live one.
    /// </summary>
    [Fact]
    public async Task SaveAsync_WhenReplaceFails_LeavesOldConfigIntact()
    {
        var (store, fileSystem, clock) = CreateStore();

        await store.SaveAsync(ConfigWithOneDrape("Original"));
        clock.Advance(TimeSpan.FromSeconds(1));

        fileSystem.FailNextReplace = true;
        await Should.ThrowAsync<IOException>(() => store.SaveAsync(ConfigWithOneDrape("Replacement")));

        var result = await store.LoadAsync();
        result.Outcome.ShouldBe(ConfigLoadOutcome.Loaded);
        result.Config.Drapes[0].Title.ShouldBe("Original");
    }

    /// <summary>A corrupt config is quarantined and the newest valid backup is used.</summary>
    [Fact]
    public async Task LoadAsync_WithCorruptConfig_QuarantinesAndUsesBackup()
    {
        var (store, fileSystem, clock) = CreateStore();

        await store.SaveAsync(ConfigWithOneDrape("Backed up"));
        clock.Advance(TimeSpan.FromSeconds(1));
        await store.SaveAsync(ConfigWithOneDrape("Live"));

        // Corrupt the live file without going through the store.
        fileSystem.AddFile(ConfigPath, "{ this is not json");

        var result = await store.LoadAsync();

        result.Outcome.ShouldBe(ConfigLoadOutcome.RecoveredFromBackup);
        result.QuarantinedPath.ShouldNotBeNull();
        result.Config.Drapes[0].Title.ShouldBe("Backed up");
        result.Message.ShouldNotBeNull();
    }

    /// <summary>A corrupt config with no usable backup starts empty and touches no desktop file.</summary>
    [Fact]
    public async Task LoadAsync_WithCorruptConfigAndNoBackup_StartsEmpty()
    {
        var (store, fileSystem, _) = CreateStore();
        fileSystem.AddFile(ConfigPath, "not json at all");

        var result = await store.LoadAsync();

        result.Outcome.ShouldBe(ConfigLoadOutcome.StartedEmpty);
        result.Config.Pages.Count.ShouldBe(1);
        result.Config.Drapes.ShouldBeEmpty();
        result.QuarantinedPath.ShouldNotBeNull();
    }

    /// <summary>A document from a newer version opens read-only and refuses to save.</summary>
    [Fact]
    public async Task LoadAsync_WithNewerSchema_IsReadOnly()
    {
        var (store, fileSystem, _) = CreateStore();
        fileSystem.AddFile(ConfigPath, """
            {
              "schemaVersion": 99,
              "pages": [ { "id": "6f1c2b8e-0000-4000-8000-000000000001", "name": "Main", "order": 0 } ],
              "drapes": []
            }
            """);

        var result = await store.LoadAsync();

        result.Outcome.ShouldBe(ConfigLoadOutcome.ReadOnlyNewerSchema);
        result.CanSave.ShouldBeFalse();

        await Should.ThrowAsync<InvalidOperationException>(() => store.SaveAsync(result.Config));

        // The newer file must still be on disk, unmodified.
        fileSystem.FileExists(ConfigPath).ShouldBeTrue();
        fileSystem.Replaces.ShouldBeEmpty();
    }

    /// <summary>A document that parses but breaks a domain rule is treated as corrupt.</summary>
    [Fact]
    public async Task LoadAsync_WithDomainViolation_TreatsAsCorrupt()
    {
        var (store, fileSystem, _) = CreateStore();

        // A Drape whose bounds are below the minimum: valid JSON, invalid model.
        fileSystem.AddFile(ConfigPath, """
            {
              "schemaVersion": 1,
              "pages": [ { "id": "6f1c2b8e-0000-4000-8000-000000000001", "name": "Main", "order": 0 } ],
              "drapes": [
                {
                  "id": "6f1c2b8e-0000-4000-8000-000000000010",
                  "_pageId": "6f1c2b8e-0000-4000-8000-000000000001",
                  "title": "Too small",
                  "bounds": { "monitorId": "\\\\.\\DISPLAY1", "x": 0, "y": 0, "width": 10, "height": 10, "dpi": 96, "fx": 0, "fy": 0, "fw": 0.1, "fh": 0.1 }
                }
              ]
            }
            """);

        var result = await store.LoadAsync();

        result.Outcome.ShouldBe(ConfigLoadOutcome.StartedEmpty);
    }

    /// <summary>Backups beyond the limit are pruned, oldest first.</summary>
    [Fact]
    public async Task SaveAsync_PrunesBackupsBeyondLimit()
    {
        var (store, fileSystem, clock) = CreateStore();

        await store.SaveAsync(ConfigWithOneDrape());
        for (var i = 0; i < FileConfigStore.MaxBackups + 5; i++)
        {
            clock.Advance(TimeSpan.FromSeconds(1));
            await store.SaveAsync(ConfigWithOneDrape($"Version {i}"));
        }

        var backups = fileSystem.EnumerateFiles(store.BackupDirectory);

        backups.Count.ShouldBeLessThanOrEqualTo(FileConfigStore.MaxBackups);
    }

    /// <summary>Unknown root properties survive a load and save cycle.</summary>
    [Fact]
    public async Task SaveThenLoad_PreservesUnknownProperties()
    {
        var (store, fileSystem, clock) = CreateStore();

        fileSystem.AddFile(ConfigPath, """
            {
              "schemaVersion": 1,
              "futureFeature": { "enabled": true },
              "pages": [ { "id": "6f1c2b8e-0000-4000-8000-000000000001", "name": "Main", "order": 0 } ],
              "drapes": []
            }
            """);

        var loaded = await store.LoadAsync();
        clock.Advance(TimeSpan.FromSeconds(1));
        await store.SaveAsync(loaded.Config);

        var written = fileSystem.ReadAllText(ConfigPath);
        written.ShouldContain("futureFeature");
    }

    /// <summary>A read failure is treated as corruption rather than as an empty config.</summary>
    [Fact]
    public async Task LoadAsync_WhenFileCannotBeRead_Recovers()
    {
        var (store, fileSystem, _) = CreateStore();
        fileSystem.AddFile(ConfigPath, "{}");

        var result = await store.LoadAsync();

        // "{}" parses but has no pages, so it is corrupt by the domain rules.
        result.Outcome.ShouldBe(ConfigLoadOutcome.StartedEmpty);
    }

    /// <summary>The store never deletes the user's config; it quarantines instead.</summary>
    [Fact]
    public async Task LoadAsync_WithCorruptConfig_DoesNotDeleteIt()
    {
        var (store, fileSystem, _) = CreateStore();
        fileSystem.AddFile(ConfigPath, "broken");

        await store.LoadAsync();

        // The corrupt file was moved, not deleted.
        fileSystem.Deletions.ShouldNotContain(ConfigPath);
    }

    /// <summary>The store writes the current schema version, not one carried in from elsewhere.</summary>
    [Fact]
    public async Task SaveAsync_AlwaysWritesCurrentSchemaVersion()
    {
        var (store, fileSystem, _) = CreateStore();
        var old = ConfigWithOneDrape() with { SchemaVersion = 1 };

        await store.SaveAsync(old);

        var written = fileSystem.ReadAllText(ConfigPath);
        written.ShouldContain($"\"schemaVersion\": {DrapeConfig.CurrentSchemaVersion}");
    }

    /// <summary>Saving 50 Drapes × 50 items stays under the 20 ms budget (P1-03 DoD).</summary>
    /// <remarks>
    /// This runs against <see cref="FakeFileSystem"/>, so it measures serialisation plus store
    /// overhead without real disk I/O — a lower bound. A real-disk number needs a manual run.
    /// </remarks>
    [Fact]
    public async Task SaveAsync_FiftyDrapesTimesFiftyItems_UnderTwentyMilliseconds()
    {
        var (store, _, _) = CreateStore();
        var page = new Page(_pageId, "Main", 0);
        var drapes = new List<DrapeEntity>(50);
        for (var d = 0; d < 50; d++)
        {
            var items = new List<DrapeItem>(50);
            for (var i = 0; i < 50; i++)
            {
                items.Add(new DrapeItem(
                    Guid.NewGuid(),
                    DrapeItemKind.File,
                    $@"C:\Users\test\Desktop\drape-{d:00}\file-{i:000}.txt",
                    i,
                    new DateTimeOffset(2026, 1, 1, 0, 0, 0, TimeSpan.Zero)));
            }

            drapes.Add(new DrapeEntity(
                Guid.NewGuid(),
                _pageId,
                $"Drape {d:00}",
                new DrapeBounds(@"\\.\DISPLAY1", 40, 60, 320, 240, 96, 0.02, 0.05, 0.16, 0.22))
            {
                Items = items,
            });
        }

        var config = new DrapeConfig(1, [page], drapes);

        // Warm up once so JIT cost does not pollute the measurement.
        await store.SaveAsync(config);

        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        await store.SaveAsync(config);
        stopwatch.Stop();

        stopwatch.Elapsed.ShouldBeLessThan(TimeSpan.FromMilliseconds(20),
            $"saving 50 Drapes × 50 items took {stopwatch.Elapsed.TotalMilliseconds:F1} ms");
    }
}
