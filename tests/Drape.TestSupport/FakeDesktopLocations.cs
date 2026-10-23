using Drape.Core;

namespace Drape.TestSupport;

/// <summary>Desktop and data locations that point at a fictional tree.</summary>
public sealed class FakeDesktopLocations : IDesktopLocations
{
    /// <inheritdoc />
    public string UserDesktop { get; init; } = @"C:\Users\test\Desktop";

    /// <inheritdoc />
    public string PublicDesktop { get; init; } = @"C:\Users\Public\Desktop";

    /// <inheritdoc />
    public string AppDataDirectory { get; init; } = @"C:\Users\test\AppData\Local\Drape";
}
