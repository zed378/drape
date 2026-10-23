using Drape.Core;
using Drape.Core.Model;

namespace Drape.TestSupport;

/// <summary>
/// A desktop host that records placement calls and can be told to fail, so the degraded path is
/// exercised rather than assumed.
/// </summary>
public sealed class FakeDesktopHost : IDesktopHost
{
    /// <summary>When set, <see cref="AttachAsync"/> reports <see cref="DesktopHostStatus.Degraded"/>.</summary>
    public bool AttachFails { get; init; }

    /// <inheritdoc />
    public DesktopHostStatus Status { get; private set; } = DesktopHostStatus.Detached;

    /// <summary>The handle passed to <see cref="AttachAsync"/>.</summary>
    public nint AttachedHandle { get; private set; }

    /// <summary>The last bounds passed to <see cref="PlaceAsync"/>.</summary>
    public DrapeBounds? LastPlaced { get; private set; }

    /// <summary>How many times <see cref="EnsureZOrderAsync"/> was called.</summary>
    public int ZOrderCalls { get; private set; }

    /// <summary>How many times <see cref="ReattachAsync"/> was called.</summary>
    public int ReattachCalls { get; private set; }

    /// <inheritdoc />
    public Task<DesktopHostStatus> AttachAsync(nint windowHandle, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        AttachedHandle = windowHandle;
        Status = AttachFails ? DesktopHostStatus.Degraded : DesktopHostStatus.Attached;
        return Task.FromResult(Status);
    }

    /// <inheritdoc />
    public Task<DesktopHostStatus> ReattachAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        ReattachCalls++;
        Status = AttachFails ? DesktopHostStatus.Degraded : DesktopHostStatus.Attached;
        return Task.FromResult(Status);
    }

    /// <inheritdoc />
    public Task PlaceAsync(DrapeBounds bounds, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        LastPlaced = bounds;
        return Task.CompletedTask;
    }

    /// <inheritdoc />
    public Task EnsureZOrderAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        ZOrderCalls++;
        return Task.CompletedTask;
    }

    /// <inheritdoc />
    public Task DetachAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        Status = DesktopHostStatus.Detached;
        return Task.CompletedTask;
    }
}
