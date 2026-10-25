using Drape.Core.Layout;
using Drape.Core.Model;
using Drape.Core.Persistence;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.Core.Services;

/// <summary>
/// Creates, renames and deletes Drapes on top of a loaded configuration (P1-07).
/// </summary>
/// <remarks>
/// <para>
/// Deleting a Drape removes only the reference. It never touches a file on disk — the items become
/// ungrouped, and the test for that asserts against the file system, not the in-memory list.
/// </para>
/// <para>
/// The service holds the current document and queues every change through
/// <see cref="ConfigPersistence"/>. It is not thread-safe; callers serialise their calls.
/// </para>
/// </remarks>
public sealed class DrapeService
{
    private readonly ConfigPersistence _persistence;
    private readonly IDisplayMonitor _monitors;

    private DrapeConfig _config;

    /// <summary>Creates a service over an already-loaded configuration.</summary>
    /// <param name="config">The current document.</param>
    /// <param name="persistence">The debounced saver.</param>
    /// <param name="monitors">The display service, for placing new Drapes.</param>
    public DrapeService(DrapeConfig config, ConfigPersistence persistence, IDisplayMonitor monitors)
    {
        ArgumentNullException.ThrowIfNull(config);
        ArgumentNullException.ThrowIfNull(persistence);
        ArgumentNullException.ThrowIfNull(monitors);

        _config = config;
        _persistence = persistence;
        _monitors = monitors;
    }

    /// <summary>The current document.</summary>
    public DrapeConfig Config => _config;

    /// <summary>Raised after the document changes. The UI subscribes to refresh its views.</summary>
    public event EventHandler<DrapeConfig>? Changed;

    /// <summary>
    /// Creates a Drape with a unique default name on the primary monitor.
    /// </summary>
    /// <param name="pageId">The page to put it on; defaults to the first page.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The new Drape.</returns>
    /// <exception cref="InvalidOperationException">No page with that id exists.</exception>
    public async Task<DrapeEntity> CreateAsync(Guid? pageId = null, CancellationToken ct = default)
    {
        var targetPage = pageId ?? _config.Pages[0].Id;
        if (!_config.Pages.Any(page => page.Id == targetPage))
        {
            throw new InvalidOperationException($"No page with id {targetPage} exists.");
        }

        var monitors = await _monitors.GetMonitorsAsync(ct).ConfigureAwait(false);
        var primary = monitors.FirstOrDefault(m => m.IsPrimary, monitors[0]);
        var area = primary.WorkingArea;

        // A sensible default: centred, 320×240, clamped to the work area.
        var bounds = new DrapeBounds(
            primary.Id, area.X + ((area.Width - 320) / 2), area.Y + ((area.Height - 240) / 2),
            320, 240, primary.DpiX, 0.4, 0.4, 0.16, 0.22);
        bounds = LayoutService.Clamp(bounds, area);

        var drape = new DrapeEntity(Guid.NewGuid(), targetPage, UniqueName("New Drape"), bounds);
        _config = _config.AddDrape(drape);
        _persistence.Queue(_config);
        Changed?.Invoke(this, _config);
        return drape;
    }

    /// <summary>Renames a Drape.</summary>
    /// <param name="drapeId">The Drape to rename.</param>
    /// <param name="title">The new title; must not be blank.</param>
    /// <returns>The renamed Drape.</returns>
    /// <exception cref="InvalidOperationException">No Drape with that id exists.</exception>
    public DrapeEntity Rename(Guid drapeId, string title)
    {
        var existing = Find(drapeId);
        var renamed = existing.Rename(title);
        _config = _config.ReplaceDrape(renamed);
        _persistence.Queue(_config);
        Changed?.Invoke(this, _config);
        return renamed;
    }

    /// <summary>
    /// Deletes a Drape. Its items become ungrouped; no file is touched.
    /// </summary>
    /// <param name="drapeId">The Drape to delete.</param>
    /// <returns>The items that were in the Drape, now ungrouped.</returns>
    /// <exception cref="InvalidOperationException">No Drape with that id exists.</exception>
    public IReadOnlyList<DrapeItem> Delete(Guid drapeId)
    {
        var existing = Find(drapeId);
        _config = _config.RemoveDrape(drapeId);
        _persistence.Queue(_config);
        Changed?.Invoke(this, _config);
        return existing.Items;
    }

    private DrapeEntity Find(Guid drapeId)
    {
        var drape = _config.Drapes.FirstOrDefault(d => d.Id == drapeId);
        if (drape is null)
        {
            throw new InvalidOperationException($"No Drape with id {drapeId} exists.");
        }

        return drape;
    }

    private string UniqueName(string stem)
    {
        var taken = _config.Drapes.Select(d => d.Title).ToHashSet(StringComparer.OrdinalIgnoreCase);
        if (!taken.Contains(stem))
        {
            return stem;
        }

        for (var i = 2; ; i++)
        {
            var candidate = $"{stem} {i}";
            if (!taken.Contains(candidate))
            {
                return candidate;
            }
        }
    }
}
