namespace Drape.Core.Model;

/// <summary>
/// A panel on the desktop holding a set of item references.
/// </summary>
/// <remarks>
/// <para>
/// The type is named <c>Drape</c> and so is the root namespace (CS-10, ADR-020). An explicit
/// constructor cannot be declared: <c>public Drape(...)</c> does not compile, because in that
/// position <c>Drape</c> resolves to the namespace rather than to the type. A primary constructor
/// avoids naming the type, so this record uses one.
/// </para>
/// <para>
/// The item collection is exposed as a read-only view over a private copy. A record with a public
/// mutable collection property is immutable in name only; tests would pass while callers mutated
/// shared state.
/// </para>
/// </remarks>
#pragma warning disable CA1724 // The type name collides with the root namespace by design; see CS-10.
public sealed record Drape(
    Guid Id,
    Guid PageId,
    string Title,
    DrapeBounds Bounds,
    DrapeKind Kind = DrapeKind.Normal,
    DrapeStyle? Style = null,
    DrapeState? State = null,
    bool QuickHideExcluded = false)
#pragma warning restore CA1724
{
    private readonly List<DrapeItem> _items = [];

    /// <summary>The items, in the order they are stored. Read-only.</summary>
    /// <exception cref="ArgumentException">Two items share an id or a path.</exception>
    public IReadOnlyList<DrapeItem> Items
    {
        get => _items;
        init
        {
            ArgumentNullException.ThrowIfNull(value);
            EnsureUniqueItems(value);
            _items.Clear();
            _items.AddRange(value);
        }
    }

    /// <summary>User-visible title; never blank.</summary>
    /// <exception cref="ArgumentException">The title is blank.</exception>
    public string Title { get; init; } = ValidateTitle(Title);

    /// <summary>Unique identifier; never empty.</summary>
    /// <exception cref="ArgumentException">The id is empty.</exception>
    public Guid Id { get; init; } = ValidateId(Id, "Drape");

    /// <summary>The page this Drape belongs to; never empty.</summary>
    /// <exception cref="ArgumentException">The page id is empty.</exception>
    public Guid PageId { get; init; } = ValidateId(PageId, "page");

    /// <summary>Visual style; never <c>null</c>.</summary>
    public DrapeStyle Style { get; init; } = Style ?? DrapeStyle.Default;

    /// <summary>Runtime state; never <c>null</c>.</summary>
    public DrapeState State { get; init; } = State ?? DrapeState.Default;

    /// <summary>Returns a copy with a different title.</summary>
    /// <param name="title">The new title; must not be blank.</param>
    /// <returns>The renamed Drape.</returns>
    /// <exception cref="ArgumentException">The title is blank.</exception>
    /// <remarks>
    /// Validated explicitly because a <c>with</c> expression assigns through the <c>init</c>
    /// accessor without running the property initializer — the constructor-time validation does
    /// not apply to copies.
    /// </remarks>
    public Drape Rename(string title)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(title);
        return this with { Title = title };
    }

    /// <summary>Returns a copy with new bounds.</summary>
    /// <param name="bounds">The new bounds.</param>
    /// <returns>The moved Drape.</returns>
    public Drape WithBounds(DrapeBounds bounds) => this with { Bounds = bounds };

    /// <summary>
    /// Returns a copy with an item added at the end.
    /// </summary>
    /// <param name="item">The item to add.</param>
    /// <returns>The Drape with the item appended.</returns>
    /// <exception cref="InvalidOperationException">An item with the same id or path already exists.</exception>
    public Drape AddItem(DrapeItem item)
    {
        ArgumentNullException.ThrowIfNull(item);

        if (_items.Any(existing => existing.Id == item.Id))
        {
            throw new InvalidOperationException($"An item with id {item.Id} is already in this Drape.");
        }

        if (_items.Any(existing => string.Equals(existing.Path, item.Path, StringComparison.OrdinalIgnoreCase)))
        {
            throw new InvalidOperationException($"The path '{item.Path}' is already in this Drape.");
        }

        var added = item.Order == _items.Count ? item : item.WithOrder(_items.Count);
        return this with { Items = [.. _items, added] };
    }

    /// <summary>
    /// Returns a copy with an item removed. This never touches the file on disk.
    /// </summary>
    /// <param name="itemId">The item to remove.</param>
    /// <returns>The Drape without the item, with the remaining items renumbered.</returns>
    public Drape RemoveItem(Guid itemId)
    {
        var remaining = _items.Where(item => item.Id != itemId).ToList();
        for (var i = 0; i < remaining.Count; i++)
        {
            remaining[i] = remaining[i].WithOrder(i);
        }

        return this with { Items = remaining };
    }

    private static string ValidateTitle(string title)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(title);
        return title;
    }

    private static Guid ValidateId(Guid id, string what)
    {
        if (id == Guid.Empty)
        {
            throw new ArgumentException($"A {what} needs a non-empty id.", what);
        }

        return id;
    }

    private static void EnsureUniqueItems(IReadOnlyList<DrapeItem> items)
    {
        var ids = new HashSet<Guid>();
        var paths = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var item in items)
        {
            if (!ids.Add(item.Id))
            {
                throw new ArgumentException($"Duplicate item id {item.Id}.", nameof(items));
            }

            if (!paths.Add(item.Path))
            {
                throw new ArgumentException($"Duplicate item path '{item.Path}'.", nameof(items));
            }
        }
    }
}
