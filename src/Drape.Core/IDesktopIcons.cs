namespace Drape.Core;

/// <summary>
/// Hides and restores the native desktop icons.
/// </summary>
/// <remarks>
/// <para>
/// <b>There is deliberately no toggle.</b> A toggle that does not read the current state first will
/// flip the user's desktop whenever the app's belief about that state is wrong — the abuse case
/// named in task P0-04. Callers read <see cref="AreHiddenAsync"/>, decide, then set.
/// </para>
/// <para>
/// Every hide must be matched by a restore on every exit path: normal exit, unhandled exception,
/// the Guard process, the next start, and <c>--restore-desktop</c>. This is a desktop-safety rule
/// and is not waivable.
/// </para>
/// </remarks>
public interface IDesktopIcons
{
    /// <summary>Reads whether the icons are currently hidden. Never changes the state.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> when the icons are hidden.</returns>
    Task<bool> AreHiddenAsync(CancellationToken ct = default);

    /// <summary>Hides the icons if they are not already hidden.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> when the icons are hidden afterwards.</returns>
    Task<bool> HideAsync(CancellationToken ct = default);

    /// <summary>Restores the icons if they are hidden.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> when the icons are visible afterwards.</returns>
    Task<bool> RestoreAsync(CancellationToken ct = default);
}
