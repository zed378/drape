using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Drape.Core.Model;
using Drape.Core.Services;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.UI.ViewModels;

/// <summary>
/// View-model for one Drape window (P1-04). Binds the domain to the view; never touches a native
/// API — window placement goes through <c>IDesktopHost</c>, injected at the composition root.
/// </summary>
public sealed partial class DrapeViewModel : ObservableObject
{
    private readonly DrapeService _service;
    private DrapeEntity _drape;

    /// <summary>Creates a view-model over a Drape.</summary>
    /// <param name="drape">The Drape to show.</param>
    /// <param name="service">The service for mutations.</param>
    public DrapeViewModel(DrapeEntity drape, DrapeService service)
    {
        ArgumentNullException.ThrowIfNull(drape);
        ArgumentNullException.ThrowIfNull(service);

        _drape = drape;
        _service = service;
        Items = new ObservableCollection<DrapeItemViewModel>(
            drape.Items.Select(item => new DrapeItemViewModel(item)));
    }

    /// <summary>The Drape id.</summary>
    public Guid Id => _drape.Id;

    /// <summary>Window title, bound to the view.</summary>
    [ObservableProperty]
    private string _title = string.Empty;

    /// <summary>Whether the Drape is rolled up to its title bar.</summary>
    [ObservableProperty]
    private bool _isRolledUp;

    /// <summary>Whether the Drape is locked against move/resize.</summary>
    [ObservableProperty]
    private bool _isLocked;

    /// <summary>Items shown in the grid.</summary>
    public ObservableCollection<DrapeItemViewModel> Items { get; }

    /// <summary>Refreshes from the current domain state.</summary>
    /// <param name="drape">The updated Drape.</param>
    public void Refresh(DrapeEntity drape)
    {
        ArgumentNullException.ThrowIfNull(drape);
        _drape = drape;
        Title = drape.Title;
        IsRolledUp = drape.State.RolledUp;
        IsLocked = drape.State.Locked;

        Items.Clear();
        foreach (var item in drape.Items)
        {
            Items.Add(new DrapeItemViewModel(item));
        }
    }

    /// <summary>Rolls the Drape up or opens it.</summary>
    [RelayCommand]
    private void ToggleRollup() => IsRolledUp = !IsRolledUp;

    /// <summary>Renames the Drape through the service.</summary>
    /// <param name="title">The new title.</param>
    public void Rename(string title)
    {
        var renamed = _service.Rename(_drape.Id, title);
        Refresh(renamed);
    }

    /// <summary>Initialises bound properties from the Drape. Called once after construction.</summary>
    public void Initialise() => Refresh(_drape);
}

/// <summary>View-model for one item in the grid.</summary>
/// <param name="Item">The domain item.</param>
public sealed partial class DrapeItemViewModel(DrapeItem Item) : ObservableObject
{
    /// <summary>The domain item.</summary>
    public DrapeItem Item { get; } = Item;

    /// <summary>Display name: the override, or the file name.</summary>
    public string DisplayName => Item.DisplayName ?? Path.GetFileName(Item.Path);

    /// <summary>Whether the file is missing; the view dims it and shows a badge.</summary>
    public bool IsMissing => Item.IsMissing;

    /// <summary>Opacity for the view: 0.5 when missing per the UI spec, 1.0 otherwise.</summary>
    public double DimOpacity => Item.IsMissing ? 0.5 : 1.0;

    /// <summary>Tooltip text when missing; null otherwise (no tooltip).</summary>
    public string? MissingTooltip => Item.IsMissing ? "File not found. It may have been moved or deleted." : null;
}
