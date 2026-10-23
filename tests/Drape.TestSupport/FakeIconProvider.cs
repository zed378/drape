using Drape.Core;

namespace Drape.TestSupport;

/// <summary>An icon provider that returns a solid 1x1 image, or <c>null</c> for unknown paths.</summary>
public sealed class FakeIconProvider : IIconProvider
{
    private readonly Dictionary<string, IconBitmap> _icons = new(StringComparer.OrdinalIgnoreCase);

    /// <summary>Paths requested, in order.</summary>
    public List<string> Requests { get; } = [];

    /// <summary>Registers an icon for a path.</summary>
    /// <param name="path">The path.</param>
    /// <param name="bitmap">The icon to return.</param>
    public void Add(string path, IconBitmap bitmap) => _icons[path] = bitmap;

    /// <inheritdoc />
    public Task<IconBitmap?> GetIconAsync(string path, int pixelSize, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        Requests.Add(path);
        return Task.FromResult(_icons.TryGetValue(path, out var bitmap) ? bitmap : null);
    }

    /// <summary>Builds a transparent 1x1 icon, which is enough for wiring tests.</summary>
    /// <returns>A valid 1x1 BGRA bitmap.</returns>
    public static IconBitmap SinglePixel() => new(1, 1, [0, 0, 0, 0]);
}
