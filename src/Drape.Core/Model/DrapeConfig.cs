using DrapeEntity = global::Drape.Core.Model.Drape;

namespace Drape.Core.Model;

/// <summary>
/// The whole persisted configuration: settings, pages, Drapes and rules.
/// </summary>
/// <remarks>
/// Construction validates the cross-entity rules from <c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>:
/// ids are unique, every Drape's page exists, and an item belongs to at most one Drape (ADR-011).
/// Keeping these here means the UI cannot construct an invalid document and the store cannot
/// persist one.
/// </remarks>
public sealed record DrapeConfig
{
    /// <summary>The schema version this build writes.</summary>
    public const int CurrentSchemaVersion = 1;

    private readonly List<Page> _pages;
    private readonly List<DrapeEntity> _drapes;

    /// <summary>Creates a configuration, validating the cross-entity rules.</summary>
    /// <param name="schemaVersion">The schema version read from disk.</param>
    /// <param name="pages">Pages; at least one is required.</param>
    /// <param name="drapes">Drapes.</param>
    /// <param name="settings">Application settings; <c>null</c> means defaults.</param>
    /// <exception cref="ArgumentOutOfRangeException"><paramref name="schemaVersion"/> is not positive.</exception>
    /// <exception cref="ArgumentException">A page id is duplicated, or there are no pages.</exception>
    /// <exception cref="ArgumentException">A Drape references a page that does not exist.</exception>
    /// <exception cref="ArgumentException">The same item path appears in more than one Drape.</exception>
    public DrapeConfig(
        int schemaVersion,
        IEnumerable<Page> pages,
        IEnumerable<DrapeEntity>? drapes = null,
        DrapeSettings? settings = null)
    {
        if (schemaVersion <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(schemaVersion), schemaVersion, "The schema version must be positive.");
        }

        _pages = pages is null ? [] : [.. pages];
        _drapes = drapes is null ? [] : [.. drapes];

        if (_pages.Count == 0)
        {
            throw new ArgumentException("A configuration needs at least one page.", nameof(pages));
        }

        EnsureUniqueIds(_pages.Select(page => page.Id), "page");
        EnsureUniqueIds(_drapes.Select(drape => drape.Id), "Drape");
        EnsurePagesExist();
        EnsureEachItemBelongsToOneDrape();

        SchemaVersion = schemaVersion;
        Settings = settings ?? DrapeSettings.Default;
    }

    /// <summary>The schema version of this document.</summary>
    public int SchemaVersion { get; init; }

    /// <summary>Application settings.</summary>
    public DrapeSettings Settings { get; }

    /// <summary>The pages, read-only.</summary>
    public IReadOnlyList<Page> Pages => _pages;

    /// <summary>The Drapes, read-only.</summary>
    public IReadOnlyList<DrapeEntity> Drapes => _drapes;

    /// <summary>Creates an empty configuration with one page, as a first run does.</summary>
    /// <param name="pageId">Id for the initial page.</param>
    /// <param name="pageName">Name for the initial page.</param>
    /// <returns>A configuration with one empty page.</returns>
    public static DrapeConfig CreateEmpty(Guid pageId, string pageName = "Main")
        => new(CurrentSchemaVersion, [new Page(pageId, pageName, 0)]);

    /// <summary>Returns a copy with a Drape added.</summary>
    /// <param name="drape">The Drape to add.</param>
    /// <returns>The configuration containing the Drape.</returns>
    /// <exception cref="InvalidOperationException">The Drape id exists or its page does not.</exception>
    /// <exception cref="InvalidOperationException">One of its items is already in another Drape.</exception>
    public DrapeConfig AddDrape(DrapeEntity drape)
    {
        ArgumentNullException.ThrowIfNull(drape);
        return new DrapeConfig(SchemaVersion, _pages, [.. _drapes, drape], Settings);
    }

    /// <summary>Returns a copy with a Drape replaced, matched by id.</summary>
    /// <param name="drape">The Drape to store.</param>
    /// <returns>The updated configuration.</returns>
    /// <exception cref="InvalidOperationException">No Drape with that id exists.</exception>
    public DrapeConfig ReplaceDrape(DrapeEntity drape)
    {
        ArgumentNullException.ThrowIfNull(drape);

        var index = _drapes.FindIndex(existing => existing.Id == drape.Id);
        if (index < 0)
        {
            throw new InvalidOperationException($"No Drape with id {drape.Id} exists.");
        }

        var updated = new List<DrapeEntity>(_drapes) { [index] = drape };
        return new DrapeConfig(SchemaVersion, _pages, updated, Settings);
    }

    /// <summary>
    /// Returns a copy with a Drape removed. Its items become ungrouped; no file is touched.
    /// </summary>
    /// <param name="drapeId">The Drape to remove.</param>
    /// <returns>The configuration without the Drape.</returns>
    public DrapeConfig RemoveDrape(Guid drapeId)
        => new(SchemaVersion, _pages, _drapes.Where(drape => drape.Id != drapeId), Settings);

    private void EnsurePagesExist()
    {
        var pageIds = _pages.Select(page => page.Id).ToHashSet();
        foreach (var drape in _drapes)
        {
            if (!pageIds.Contains(drape.PageId))
            {
                throw new ArgumentException(
                    $"Drape {drape.Id} references page {drape.PageId}, which does not exist.",
                    nameof(_drapes));
            }
        }
    }

    /// <summary>ADR-011: an item belongs to exactly one Drape in v1.</summary>
    private void EnsureEachItemBelongsToOneDrape()
    {
        var owners = new Dictionary<string, Guid>(StringComparer.OrdinalIgnoreCase);
        foreach (var drape in _drapes)
        {
            foreach (var item in drape.Items)
            {
                if (owners.TryGetValue(item.Path, out var owner) && owner != drape.Id)
                {
                    throw new ArgumentException(
                        $"The path '{item.Path}' is in both Drape {owner} and Drape {drape.Id}; an item belongs to one Drape (ADR-011).",
                        nameof(_drapes));
                }

                owners[item.Path] = drape.Id;
            }
        }
    }

    private static void EnsureUniqueIds(IEnumerable<Guid> ids, string kind)
    {
        var seen = new HashSet<Guid>();
        foreach (var id in ids)
        {
            if (!seen.Add(id))
            {
                throw new ArgumentException($"Duplicate {kind} id {id}.", kind);
            }
        }
    }
}
