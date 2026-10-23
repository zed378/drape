namespace Drape.Core;

/// <summary>
/// Capabilities that the platform (Windows/macOS/Linux) provides to the core application.
/// The implementation lives in the platform-specific shell project.
/// </summary>
public interface IPlatformCapabilities
{
    /// <summary>
    /// Whether the current OS session supports hiding desktop icons.
    /// </summary>
    bool CanHideDesktopIcons { get; }

    /// <summary>
    /// Whether the current OS supports desktop-level window hosting (window behind icons).
    /// </summary>
    bool SupportsDesktopLevelWindow { get; }

    /// <summary>
    /// Gets the list of connected monitors with their bounds and DPI.
    /// </summary>
    IReadOnlyList<MonitorInfo> GetMonitors();

    /// <summary>
    /// Gets the primary monitor info.
    /// </summary>
    MonitorInfo GetPrimaryMonitor();

    /// <summary>
    /// Opens a file/folder picker dialog.
    /// </summary>
    Task<string?> PickFolderAsync(string? initialPath = null, CancellationToken ct = default);

    /// <summary>
    /// Opens a file picker dialog.
    /// </summary>
    Task<IReadOnlyList<string>?> PickFilesAsync(string? initialPath = null, string? filter = null, CancellationToken ct = default);
}

/// <summary>
/// Information about a monitor.
/// </summary>
public readonly record struct MonitorInfo(
    string Id,
    string DeviceName,
    Rectangle Bounds,
    Rectangle WorkingArea,
    double DpiX,
    double DpiY,
    double ScaleFactor,
    bool IsPrimary
);

/// <summary>
/// Rectangle with double precision for cross-platform DPI handling.
/// </summary>
public readonly record struct Rectangle(
    double X,
    double Y,
    double Width,
    double Height
)
{
    /// <summary>Gets the left edge (same as <see cref="X"/>).</summary>
    public double Left => X;

    /// <summary>Gets the top edge (same as <see cref="Y"/>).</summary>
    public double Top => Y;

    /// <summary>Gets the right edge (X + Width).</summary>
    public double Right => X + Width;

    /// <summary>Gets the bottom edge (Y + Height).</summary>
    public double Bottom => Y + Height;

    /// <summary>Gets an empty rectangle at the origin.</summary>
    public static Rectangle Empty => new(0, 0, 0, 0);
}

/// <summary>
/// Capabilities related to desktop icon management.
/// </summary>
public interface IDesktopIconManager
{
    /// <summary>
    /// Hides all desktop icons.
    /// </summary>
    Task<bool> HideIconsAsync(CancellationToken ct = default);

    /// <summary>
    /// Restores all desktop icons.
    /// </summary>
    Task<bool> ShowIconsAsync(CancellationToken ct = default);

    /// <summary>
    /// Checks if icons are currently hidden.
    /// </summary>
    Task<bool> AreIconsHiddenAsync(CancellationToken ct = default);
}

/// <summary>
/// Capabilities related to desktop window hosting.
/// </summary>
public interface IDesktopWindowHost
{
    /// <summary>
    /// Creates a window at desktop level (behind icons, above wallpaper).
    /// </summary>
    Task<IDesktopWindow> CreateWindowAsync(DesktopWindowOptions options, CancellationToken ct = default);
}

/// <summary>
/// Options for creating a desktop-level window.
/// </summary>
public readonly record struct DesktopWindowOptions(
    Rectangle Bounds,
    string? WindowClass = null,
    bool ClickThrough = false,
    bool Topmost = false
);

/// <summary>
/// A window hosted at desktop level.
/// </summary>
public interface IDesktopWindow : IDisposable
{
    /// <summary>
    /// Shows the window.
    /// </summary>
    Task ShowAsync(CancellationToken ct = default);

    /// <summary>
    /// Hides the window.
    /// </summary>
    Task HideAsync(CancellationToken ct = default);

    /// <summary>
    /// Sets the window bounds.
    /// </summary>
    Task SetBoundsAsync(Rectangle bounds, CancellationToken ct = default);

    /// <summary>
    /// Gets the current window bounds.
    /// </summary>
    Rectangle Bounds { get; }

    /// <summary>
    /// Whether the window is visible.
    /// </summary>
    bool IsVisible { get; }

    /// <summary>
    /// Native window handle (HWND on Windows).
    /// </summary>
    nint Handle { get; }
}
