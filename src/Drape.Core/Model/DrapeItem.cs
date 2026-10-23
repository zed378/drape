namespace Drape.Core.Model;

/// <summary>
/// One item inside a Drape: a reference to a file, folder or shortcut on disk.
/// </summary>
/// <remarks>
/// <para>
/// An item is a <em>reference</em>. Removing it from a Drape never touches the file on disk
/// (ADR-006, and the delete confirmation in the UI spec says so explicitly).
/// </para>
/// <para>
/// Identity is the normalised path (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>). Normalisation is
/// supplied by <c>IFileSystem</c> because case sensitivity is a per-platform rule; this type only
/// stores what it is given and compares it ordinally.
/// </para>
/// </remarks>
public sealed record DrapeItem(
    Guid Id,
    DrapeItemKind Kind,
    string Path,
    int Order,
    DateTimeOffset AddedAt,
    string? FileId = null,
    string? DisplayName = null,
    bool PinnedByUser = false,
    DateTimeOffset? MissingSince = null)
{
    /// <summary>Unique identifier; never empty.</summary>
    /// <exception cref="ArgumentException">The id is empty.</exception>
    public Guid Id { get; init; } = Id == Guid.Empty
        ? throw new ArgumentException("An item needs a non-empty id.", nameof(Id))
        : Id;

    /// <summary>Normalised path; never blank.</summary>
    /// <exception cref="ArgumentException">The path is blank.</exception>
    public string Path { get; init; } = string.IsNullOrWhiteSpace(Path)
        ? throw new ArgumentException("An item needs a path.", nameof(Path))
        : Path;

    /// <summary>Position within its Drape; never negative.</summary>
    /// <exception cref="ArgumentOutOfRangeException">The order is negative.</exception>
    public int Order { get; init; } = Order < 0
        ? throw new ArgumentOutOfRangeException(nameof(Order), Order, "Order cannot be negative.")
        : Order;

    /// <summary>Whether the file is currently missing.</summary>
    public bool IsMissing => MissingSince is not null;

    /// <summary>Returns a copy with a new order.</summary>
    /// <param name="order">New position; must not be negative.</param>
    /// <returns>The reordered item.</returns>
    public DrapeItem WithOrder(int order) => this with { Order = order };

    /// <summary>Marks the item as missing, keeping the original timestamp if it was already missing.</summary>
    /// <param name="at">When it was noticed.</param>
    /// <returns>The item with <see cref="MissingSince"/> set.</returns>
    public DrapeItem MarkMissing(DateTimeOffset at) => MissingSince is null ? this with { MissingSince = at } : this;

    /// <summary>Clears the missing marker, for example when the file reappears.</summary>
    /// <returns>The item with <see cref="MissingSince"/> cleared.</returns>
    public DrapeItem MarkPresent() => MissingSince is null ? this : this with { MissingSince = null };
}
