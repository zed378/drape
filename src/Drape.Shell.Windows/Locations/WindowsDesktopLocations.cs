using Drape.Core;

namespace Drape.Shell.Windows.Locations;

/// <summary>
/// Well-known folders resolved through the Known Folder API, never assembled from
/// <c>%USERPROFILE%\Desktop</c> (which breaks under OneDrive Known Folder Move).
/// </summary>
public sealed class WindowsDesktopLocations : IDesktopLocations
{
    // FOLDERID_Desktop and FOLDERID_PublicDesktop. These GUIDs are stable OS constants
    // documented in KnownFolders.h; CsWin32 does not project them, so they are written out.
    private static readonly Guid _folderIdDesktop = new("B4BFCC3A-DB2C-424C-B029-7FE99A87C641");
    private static readonly Guid _folderIdPublicDesktop = new("C4AA340D-F20F-4863-AFEF-F87EF2E6BA25");

    private readonly Lazy<string> _userDesktop = new(ResolveUserDesktop);
    private readonly Lazy<string> _publicDesktop = new(ResolvePublicDesktop);
    private readonly Lazy<string> _appData = new(ResolveAppData);

    /// <inheritdoc />
    public string UserDesktop => _userDesktop.Value;

    /// <inheritdoc />
    public string PublicDesktop => _publicDesktop.Value;

    /// <inheritdoc />
    public string AppDataDirectory => _appData.Value;

    private static string ResolveUserDesktop()
        => GetKnownFolder(_folderIdDesktop)
        ?? Environment.GetFolderPath(Environment.SpecialFolder.Desktop);

    private static string ResolvePublicDesktop()
        => GetKnownFolder(_folderIdPublicDesktop)
        ?? Environment.GetFolderPath(Environment.SpecialFolder.CommonDesktopDirectory);

    private static string ResolveAppData()
    {
        var local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        return Path.Combine(local, "Drape");
    }

    private static string? GetKnownFolder(Guid folderId)
    {
        var hr = PInvoke.SHGetKnownFolderPath(folderId, 0, null, out var path);
        if (hr.Failed)
        {
            return null;
        }

        using (path)
        {
            unsafe
            {
                return path.DangerousValue.Value is null ? null : path.DangerousValue.ToString();
            }
        }
    }
}
