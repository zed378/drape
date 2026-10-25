using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace Drape.UI.Views;

/// <summary>
/// One Avalonia window per Drape (ADR-007). Code-behind holds view plumbing only: no business
/// logic, no native calls — window placement goes through <c>IDesktopHost</c> (CS-30).
/// </summary>
public partial class DrapeWindow : Window
{
    /// <summary>Creates the window and loads its XAML.</summary>
    public DrapeWindow()
    {
        InitializeComponent();
    }

    private void InitializeComponent() => AvaloniaXamlLoader.Load(this);
}
