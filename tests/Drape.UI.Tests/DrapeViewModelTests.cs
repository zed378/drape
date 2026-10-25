using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Headless;
using Avalonia.Markup.Xaml;
using Drape.Core.Model;
using Drape.Core.Persistence;
using Drape.Core.Services;
using Drape.TestSupport;
using Drape.UI.ViewModels;
using Drape.UI.Views;
using Shouldly;
using Xunit;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.UI.Tests;

/// <summary>
/// View-model behaviour and window properties, run headless (P1-04 DoD: headless UI test over
/// view-model-bound content). No native calls; the window is never attached to a desktop host.
/// </summary>
public sealed class DrapeViewModelTests
{
    private static readonly Guid _pageId = Guid.Parse("6f1c2b8e-0000-4000-8000-000000000001");

    private static (DrapeViewModel ViewModel, DrapeService Service) Create(string title = "Reports")
    {
        var store = new FakeConfigStore();
        var persistence = new ConfigPersistence(store, new FakeClock());
        var service = new DrapeService(
            DrapeConfig.CreateEmpty(_pageId), persistence, FakeDisplayMonitor.SingleMonitor());
        var drape = service.CreateAsync().GetAwaiter().GetResult();
        drape = service.Rename(drape.Id, title);
        var viewModel = new DrapeViewModel(drape, service);
        viewModel.Initialise();
        return (viewModel, service);
    }

    /// <summary>Initialise copies the title into the bound property.</summary>
    [Fact]
    public void Initialise_CopiesTitle()
    {
        var (viewModel, _) = Create("Reports");

        viewModel.Title.ShouldBe("Reports");
    }

    /// <summary>ToggleRollup flips the rolled-up flag.</summary>
    [Fact]
    public void ToggleRollup_FlipsFlag()
    {
        var (viewModel, _) = Create();

        viewModel.ToggleRollupCommand.Execute(null);

        viewModel.IsRolledUp.ShouldBeTrue();
    }

    /// <summary>Rename through the view-model updates both the domain and the bound title.</summary>
    [Fact]
    public void Rename_UpdatesDomainAndBinding()
    {
        var (viewModel, service) = Create();

        viewModel.Rename("Q3");

        viewModel.Title.ShouldBe("Q3");
        service.Config.Drapes[0].Title.ShouldBe("Q3");
    }

    /// <summary>Refresh replaces the item list with the current domain items.</summary>
    [Fact]
    public void Refresh_ReflectsDomainItems()
    {
        var (viewModel, service) = Create();
        var drape = service.Config.Drapes[0];
        var updated = drape.AddItem(new DrapeItem(
            Guid.NewGuid(), DrapeItemKind.File, @"C:\a.txt", 0,
            new DateTimeOffset(2026, 1, 1, 0, 0, 0, TimeSpan.Zero)));

        viewModel.Refresh(updated);

        viewModel.Items.Count.ShouldBe(1);
        viewModel.Items[0].DisplayName.ShouldBe("a.txt");
    }

    /// <summary>A missing item is dimmed with a tooltip, per the UI spec.</summary>
    [Fact]
    public void MissingItem_IsDimmedWithTooltip()
    {
        var (viewModel, _) = Create();
        var missing = new DrapeItem(
            Guid.NewGuid(), DrapeItemKind.File, @"C:\gone.txt", 0,
            new DateTimeOffset(2026, 1, 1, 0, 0, 0, TimeSpan.Zero))
            .MarkMissing(new DateTimeOffset(2026, 1, 2, 0, 0, 0, TimeSpan.Zero));
        var drape = new DrapeEntity(
            Guid.NewGuid(), _pageId, "X",
            new DrapeBounds(@"\\.\DISPLAY1", 0, 0, 320, 240, 96, 0, 0, 0.1, 0.1))
        {
            Items = [missing],
        };

        viewModel.Refresh(drape);

        viewModel.Items[0].IsMissing.ShouldBeTrue();
        viewModel.Items[0].DimOpacity.ShouldBe(0.5);
        viewModel.Items[0].MissingTooltip.ShouldNotBeNull();
    }

    /// <summary>The window is borderless, hidden from the taskbar, and never takes focus on show.</summary>
    /// <remarks>
    /// Runs on the headless platform: the window is constructed but never shown on a real desktop.
    /// Desktop-layer attach (P1-05) is not exercised here.
    /// </remarks>
    [Fact]
    public void DrapeWindow_HasCorrectChrome()
    {
        using var _ = HeadlessTestApp.Start();
        var (viewModel, _) = Create();
        var window = new DrapeWindow { DataContext = viewModel };

        window.ShowInTaskbar.ShouldBeFalse();
        window.ShowActivated.ShouldBeFalse();
        window.Title.ShouldBe("Reports");
    }

    /// <summary>Minimal Avalonia application for headless tests.</summary>
    private sealed class HeadlessTestApp : Application
    {
        /// <summary>Sets up the headless platform without starting a lifetime.</summary>
        /// <returns>A no-op disposable; the headless platform needs no per-test teardown.</returns>
        public static IDisposable Start()
        {
            AppBuilder.Configure<HeadlessTestApp>()
                .UseHeadless(new AvaloniaHeadlessPlatformOptions())
                .SetupWithoutStarting();
            return new Noop();
        }

        private sealed class Noop : IDisposable
        {
            public void Dispose()
            {
            }
        }
    }
}
