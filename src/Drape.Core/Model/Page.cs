namespace Drape.Core.Model;

/// <summary>A page groups Drapes so the user can switch between layouts.</summary>
public sealed record Page
{
    /// <summary>Creates a page.</summary>
    /// <param name="id">Unique identifier.</param>
    /// <param name="name">User-visible name; never blank.</param>
    /// <param name="order">Position among the pages; zero-based and non-negative.</param>
    /// <exception cref="ArgumentException">The id is empty or the name is blank.</exception>
    /// <exception cref="ArgumentOutOfRangeException"><paramref name="order"/> is negative.</exception>
    public Page(Guid id, string name, int order)
    {
        if (id == Guid.Empty)
        {
            throw new ArgumentException("A page needs a non-empty id.", nameof(id));
        }

        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        if (order < 0)
        {
            throw new ArgumentOutOfRangeException(nameof(order), order, "Order cannot be negative.");
        }

        Id = id;
        Name = name;
        Order = order;
    }

    /// <summary>Unique identifier.</summary>
    public Guid Id { get; }

    /// <summary>User-visible name.</summary>
    public string Name { get; }

    /// <summary>Position among the pages.</summary>
    public int Order { get; }
}
