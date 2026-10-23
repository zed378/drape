using Drape.Core;

namespace Drape.TestSupport;

/// <summary>
/// A blur effect with a configurable supported set, so the degradation path can be tested without
/// a compositor.
/// </summary>
public sealed class FakeBlurEffect : IBlurEffect
{
    /// <summary>The kinds to report as supported, most capable first.</summary>
    public IReadOnlyList<BlurKind> SupportedKinds { get; init; } = [BlurKind.Mica, BlurKind.Acrylic, BlurKind.Blur, BlurKind.None];

    /// <summary>What each call asked for and what was applied.</summary>
    public List<(BlurKind Requested, BlurKind Applied)> Calls { get; } = [];

    /// <inheritdoc />
    public BlurKind Apply(nint windowHandle, BlurKind requested)
    {
        var applied = ResolveSupported(requested, SupportedKinds);
        Calls.Add((requested, applied));
        return applied;
    }

    /// <summary>
    /// Picks the best supported kind at or below the request, falling back to <see cref="BlurKind.None"/>.
    /// </summary>
    /// <param name="requested">The kind the caller wants.</param>
    /// <param name="supported">The kinds available.</param>
    /// <returns>The kind to apply.</returns>
    public static BlurKind ResolveSupported(BlurKind requested, IReadOnlyList<BlurKind> supported)
    {
        for (var kind = requested; kind >= BlurKind.None; kind--)
        {
            if (supported.Contains(kind))
            {
                return kind;
            }
        }

        return BlurKind.None;
    }
}
