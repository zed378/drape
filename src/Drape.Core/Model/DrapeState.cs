namespace Drape.Core.Model;

/// <summary>The transient state of a Drape: rolled up, locked, how its items are sorted.</summary>
public sealed record DrapeState
{
    /// <summary>A state with the documented defaults.</summary>
    public static DrapeState Default { get; } = new();

    /// <summary>Whether the Drape is collapsed to its title bar.</summary>
    public bool RolledUp { get; init; }

    /// <summary>Whether the Drape can be moved and resized by the user.</summary>
    public bool Locked { get; init; }

    /// <summary>How the items are ordered.</summary>
    public SortMode SortMode { get; init; } = SortMode.Manual;

    /// <summary>Hint for the desktop host's z-order; <c>null</c> means "derive it".</summary>
    public int? ZOrderHint { get; init; }
}
