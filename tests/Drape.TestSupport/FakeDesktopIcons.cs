using Drape.Core;

namespace Drape.TestSupport;

/// <summary>
/// An icon manager that models the real one honestly: it tracks whether icons are hidden and it
/// records every call, so a test can prove that a hide was followed by a restore.
/// </summary>
public sealed class FakeDesktopIcons : IDesktopIcons
{
    /// <summary>Whether the icons start hidden.</summary>
    public bool StartHidden { get; init; }

    /// <summary>When set, <see cref="HideAsync"/> throws.</summary>
    public bool FailHide { get; init; }

    /// <summary>When set, <see cref="RestoreAsync"/> throws — the stranded-icons case.</summary>
    public bool FailRestore { get; init; }

    /// <summary>The order of calls made, for example <c>["hide", "restore"]</c>.</summary>
    public List<string> Calls { get; } = [];

    /// <summary>Whether the icons are hidden right now.</summary>
    public bool Hidden { get; private set; }

    /// <inheritdoc />
    public FakeDesktopIcons()
    {
        Hidden = StartHidden;
    }

    /// <inheritdoc />
    public Task<bool> AreHiddenAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        Calls.Add("read");
        return Task.FromResult(Hidden);
    }

    /// <inheritdoc />
    public Task<bool> HideAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        Calls.Add("hide");

        if (FailHide)
        {
            throw new InvalidOperationException("Simulated hide failure.");
        }

        Hidden = true;
        return Task.FromResult(true);
    }

    /// <inheritdoc />
    public Task<bool> RestoreAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        Calls.Add("restore");

        if (FailRestore)
        {
            throw new InvalidOperationException("Simulated restore failure.");
        }

        Hidden = false;
        return Task.FromResult(true);
    }
}
