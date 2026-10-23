using Drape.Core.Model;

namespace Drape.Core;

/// <summary>Why a Drape's stored position could not be used as-is.</summary>
public enum MonitorResolution
{
    /// <summary>The stored monitor is present and the bounds were used unchanged.</summary>
    Exact,

    /// <summary>The stored monitor is gone; the position was scaled onto a fallback monitor.</summary>
    Fallback,

    /// <summary>The stored monitor is present but its work area shrank; the Drape was clamped.</summary>
    Clamped,
}

/// <summary>The result of resolving stored bounds against the monitors that exist now.</summary>
/// <param name="Bounds">Bounds that are valid for the current monitor set.</param>
/// <param name="Resolution">How they were obtained.</param>
public readonly record struct BoundsResolution(DrapeBounds Bounds, MonitorResolution Resolution);

/// <summary>
/// Resolves stored positions against the current monitor set
/// (<c>docs/PLATFORM/00-WINDOWS-INTEGRATION.md</c> §8).
/// </summary>
/// <remarks>
/// Always recompute from the stored values, never from the current on-screen position: repeated
/// scaling drifts, and users notice weeks later.
/// </remarks>
public interface IDisplayMonitor
{
    /// <summary>The monitors that exist now.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>Every attached monitor.</returns>
    Task<IReadOnlyList<MonitorInfo>> GetMonitorsAsync(CancellationToken ct = default);

    /// <summary>
    /// Resolves stored bounds for the current monitor set, scaling onto a fallback monitor when the
    /// stored one is gone.
    /// </summary>
    /// <param name="stored">The bounds read from the config.</param>
    /// <param name="monitors">The monitors that exist now; must not be empty.</param>
    /// <returns>Bounds that can be applied, and how they were derived.</returns>
    BoundsResolution Resolve(DrapeBounds stored, IReadOnlyList<MonitorInfo> monitors);
}
