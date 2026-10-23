namespace Drape.Core;

/// <summary>
/// The well-known folders Drape reads, resolved by the platform.
/// </summary>
/// <remarks>
/// Resolved through the Known Folder API on Windows, never assembled from
/// <c>%USERPROFILE%\Desktop</c>: that path is wrong when OneDrive redirects the folder
/// (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>, "Item Identity").
/// </remarks>
public interface IDesktopLocations
{
    /// <summary>The current user's desktop folder.</summary>
    string UserDesktop { get; }

    /// <summary>The public desktop folder shared by all users; may be empty when it does not exist.</summary>
    string PublicDesktop { get; }

    /// <summary>Where configuration, logs and state are written for this user.</summary>
    string AppDataDirectory { get; }
}
