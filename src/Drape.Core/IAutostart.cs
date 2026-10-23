namespace Drape.Core;

/// <summary>Registers Drape to start with the user's session.</summary>
/// <remarks>
/// On Windows this is the Run key, not the Startup folder: a shortcut in the Startup folder cannot
/// be removed reliably when the user moves the executable, and the registry value can be read back
/// to report the real state.
/// </remarks>
public interface IAutostart
{
    /// <summary>Whether autostart is currently registered.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> when Drape starts with the session.</returns>
    Task<bool> IsEnabledAsync(CancellationToken ct = default);

    /// <summary>Registers or unregisters autostart.</summary>
    /// <param name="enabled">Whether Drape should start with the session.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>A task that completes when the change is applied.</returns>
    Task SetEnabledAsync(bool enabled, CancellationToken ct = default);
}
