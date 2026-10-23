using Drape.Core.Model;

namespace Drape.Core;

/// <summary>What the host can do after an attach attempt.</summary>
public enum DesktopHostStatus
{
    /// <summary>The window is in the desktop layer.</summary>
    Attached,

    /// <summary>
    /// The window could not be placed in the desktop layer and is behaving as an ordinary window.
    /// The UI must tell the user; this is a degraded mode, not a failure to hide.
    /// </summary>
    Degraded,

    /// <summary>The host has not been attached yet.</summary>
    Detached,
}

/// <summary>
/// Places Drape windows in the desktop layer and keeps them there.
/// </summary>
/// <remarks>
/// <para>
/// The strategy behind this interface is decided by spike P0-03 and recorded in ADR-007. Until that
/// ADR is <c>Accepted</c>, <see cref="IPlatformCapabilities.SupportsDesktopLevelWindow"/> reports
/// <c>false</c> and the UI must not offer the feature.
/// </para>
/// <para>
/// Every method must be safe to call when the host is degraded: it reports the status rather than
/// throwing. The desktop window hierarchy must be discovered, never hard-coded to a fixed path.
/// </para>
/// </remarks>
public interface IDesktopHost
{
    /// <summary>The current status.</summary>
    DesktopHostStatus Status { get; }

    /// <summary>Attaches a native window handle to the desktop layer.</summary>
    /// <param name="windowHandle">The window's native handle.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The status afterwards.</returns>
    Task<DesktopHostStatus> AttachAsync(nint windowHandle, CancellationToken ct = default);

    /// <summary>
    /// Re-attaches after Explorer restarted and rebuilt its windows. The window handle stays valid;
    /// the parent does not.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The status afterwards.</returns>
    Task<DesktopHostStatus> ReattachAsync(CancellationToken ct = default);

    /// <summary>Moves the hosted window.</summary>
    /// <param name="bounds">New bounds, in DIP relative to the monitor work area.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>A task that completes when the move is applied.</returns>
    Task PlaceAsync(DrapeBounds bounds, CancellationToken ct = default);

    /// <summary>Re-asserts the z-order after another window changed it.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>A task that completes when the z-order is re-asserted.</returns>
    Task EnsureZOrderAsync(CancellationToken ct = default);

    /// <summary>Removes the window from the desktop layer without closing it.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>A task that completes when the window is detached.</returns>
    Task DetachAsync(CancellationToken ct = default);
}
