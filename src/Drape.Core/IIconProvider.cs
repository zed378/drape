namespace Drape.Core;

/// <summary>
/// Decoded icon pixels, free of any UI framework (CS-22).
/// </summary>
/// <remarks>
/// The UI converts this to its own bitmap type on its own thread. Core never sees an Avalonia,
/// WPF or System.Drawing image.
/// </remarks>
public sealed record IconBitmap
{
    /// <summary>Creates an icon bitmap.</summary>
    /// <param name="width">Pixel width.</param>
    /// <param name="height">Pixel height.</param>
    /// <param name="bgraPixels">Premultiplied BGRA pixels, <c>width * height * 4</c> bytes.</param>
    /// <exception cref="ArgumentOutOfRangeException">A dimension is not positive.</exception>
    /// <exception cref="ArgumentException">The buffer length does not match the dimensions.</exception>
    public IconBitmap(int width, int height, byte[] bgraPixels)
    {
        ArgumentNullException.ThrowIfNull(bgraPixels);

        if (width <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(width), width, "Width must be positive.");
        }

        if (height <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(height), height, "Height must be positive.");
        }

        var expected = checked(width * height * 4);
        if (bgraPixels.Length != expected)
        {
            throw new ArgumentException(
                $"Expected {expected} bytes for {width}x{height} BGRA, got {bgraPixels.Length}.",
                nameof(bgraPixels));
        }

        Width = width;
        Height = height;
        BgraPixels = bgraPixels;
    }

    /// <summary>Pixel width.</summary>
    public int Width { get; }

    /// <summary>Pixel height.</summary>
    public int Height { get; }

    /// <summary>Premultiplied BGRA pixels, row-major, top-down.</summary>
    public byte[] BgraPixels { get; }
}

/// <summary>
/// Supplies icons for items.
/// </summary>
/// <remarks>
/// Implementations must be safe to call from any thread and must not trigger cloud-file
/// hydration (<c>docs/PLATFORM/00-WINDOWS-INTEGRATION.md</c> §4).
/// </remarks>
public interface IIconProvider
{
    /// <summary>Gets the icon for an item at a requested pixel size.</summary>
    /// <param name="path">Normalised path of the item.</param>
    /// <param name="pixelSize">Requested edge length in physical pixels.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The icon, or <c>null</c> when the shell cannot supply one.</returns>
    Task<IconBitmap?> GetIconAsync(string path, int pixelSize, CancellationToken ct = default);
}
