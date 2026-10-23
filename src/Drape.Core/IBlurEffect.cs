namespace Drape.Core;

/// <summary>The blur kinds Drape can request, in order of preference.</summary>
public enum BlurKind
{
    /// <summary>No blur; a solid or translucent tint only. Always available.</summary>
    None,

    /// <summary>A simple blur of whatever is behind the panel.</summary>
    Blur,

    /// <summary>Acrylic: blur plus a tint and noise.</summary>
    Acrylic,

    /// <summary>Mica: a desktop-wallpaper-derived backdrop. Windows 11 only.</summary>
    Mica,
}

/// <summary>
/// Applies a blur kind to a window.
/// </summary>
/// <remarks>
/// Which kinds are available is a capability question answered by
/// <see cref="IPlatformCapabilities"/>; the UI asks, then degrades. A fallback must be a genuinely
/// different code path — the same blur with a different brush is not a fallback, because it is
/// never exercised when blur is unavailable.
/// </remarks>
public interface IBlurEffect
{
    /// <summary>The kinds this implementation can apply, most capable first.</summary>
    IReadOnlyList<BlurKind> SupportedKinds { get; }

    /// <summary>
    /// Applies the best available kind at or below <paramref name="requested"/>.
    /// </summary>
    /// <param name="windowHandle">Native window handle.</param>
    /// <param name="requested">The kind the user asked for.</param>
    /// <returns>The kind actually applied, which may be lower than requested.</returns>
    BlurKind Apply(nint windowHandle, BlurKind requested);
}
