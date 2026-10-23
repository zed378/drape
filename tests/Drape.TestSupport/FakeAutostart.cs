using Drape.Core;

namespace Drape.TestSupport;

/// <summary>An autostart registration that only remembers its state.</summary>
public sealed class FakeAutostart : IAutostart
{
    /// <summary>Whether autostart starts enabled.</summary>
    public bool Enabled { get; private set; }

    /// <inheritdoc />
    public Task<bool> IsEnabledAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        return Task.FromResult(Enabled);
    }

    /// <inheritdoc />
    public Task SetEnabledAsync(bool enabled, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        Enabled = enabled;
        return Task.CompletedTask;
    }
}
