namespace Drape.Core;

/// <summary>An item found on the desktop, before it becomes a <see cref="Model.DrapeItem"/>.</summary>
/// <param name="Path">Normalised path.</param>
/// <param name="IsFolder">Whether it is a directory.</param>
/// <param name="IsShortcut">Whether it is a .lnk or similar link.</param>
/// <param name="IsHidden">Whether the shell reports it hidden or system.</param>
public readonly record struct DesktopEntry(string Path, bool IsFolder, bool IsShortcut, bool IsHidden);

/// <summary>
/// Enumerates the items that live on the user's desktop.
/// </summary>
/// <remarks>
/// Reads the Desktop and Public Desktop folders and applies the Explorer hidden/system rules.
/// <c>desktop.ini</c> is never returned. Implementations must not trigger cloud-file hydration.
/// </remarks>
public interface IDesktopItemSource
{
    /// <summary>Lists the desktop entries, unsorted.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>Every entry found in the user and public desktop folders.</returns>
    Task<IReadOnlyList<DesktopEntry>> EnumerateAsync(CancellationToken ct = default);
}
