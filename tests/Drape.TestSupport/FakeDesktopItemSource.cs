using Drape.Core;

namespace Drape.TestSupport;

/// <summary>A desktop item source over a fixed list of entries.</summary>
public sealed class FakeDesktopItemSource : IDesktopItemSource
{
    private readonly List<DesktopEntry> _entries;

    /// <summary>Creates a source with the given entries.</summary>
    /// <param name="entries">The entries to return; may be empty.</param>
    public FakeDesktopItemSource(IEnumerable<DesktopEntry>? entries = null)
        => _entries = entries is null ? [] : [.. entries];

    /// <inheritdoc />
    public Task<IReadOnlyList<DesktopEntry>> EnumerateAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        return Task.FromResult<IReadOnlyList<DesktopEntry>>(_entries);
    }
}
