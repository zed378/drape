using Drape.Core.Model;
using DrapeEntity = Drape.Core.Model.Drape;

namespace Drape.Infrastructure.Configuration;

/// <summary>
/// Schema migrations, each a pure function from one version to the next
/// (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>).
/// </summary>
/// <remarks>
/// <para>
/// Version 1 is the first shipped schema, so there is no v0 to migrate from yet. The table exists
/// with a working example shape so that adding v2 is a one-line change rather than a design
/// decision made under pressure.
/// </para>
/// <para>
/// Migrations operate on the DTO, not on the domain model: the domain model only ever represents
/// the current version, so a v1 field that v2 removes would have nowhere to live in it.
/// </para>
/// </remarks>
internal static class ConfigMigrations
{
    /// <summary>
    /// Migrates a parsed document up to <paramref name="targetVersion"/>.
    /// </summary>
    /// <param name="dto">The document as read.</param>
    /// <param name="targetVersion">The version this build writes.</param>
    /// <returns><c>true</c> when any migration ran.</returns>
    /// <exception cref="ArgumentOutOfRangeException">
    /// The document is newer than the target; callers must have rejected that already.
    /// </exception>
    public static bool Migrate(ConfigJson.ConfigDto dto, int targetVersion)
    {
        ArgumentNullException.ThrowIfNull(dto);

        if (dto.SchemaVersion > targetVersion)
        {
            throw new ArgumentOutOfRangeException(
                nameof(dto),
                dto.SchemaVersion,
                "A document from a newer version cannot be migrated downwards.");
        }

        var migrated = false;

        while (dto.SchemaVersion < targetVersion)
        {
            dto = dto.SchemaVersion switch
            {
                // Placeholder for the shape of a migration. Each case must also have a hand-written
                // JSON fixture per version in tests/Drape.Infrastructure.Tests/Fixtures.
                1 => MigrateV1ToV2(dto),
                _ => throw new InvalidOperationException(
                    $"No migration is defined from schema version {dto.SchemaVersion}."),
            };

            migrated = true;
        }

        return migrated;
    }

    /// <summary>
    /// Example migration, currently unused because v2 does not exist. It is written out so the
    /// pattern is visible: change only what the new version requires and bump the version.
    /// </summary>
    /// <param name="dto">The v1 document.</param>
    /// <returns>The same document, stamped v2.</returns>
    private static ConfigJson.ConfigDto MigrateV1ToV2(ConfigJson.ConfigDto dto)
    {
        dto.SchemaVersion = 2;
        return dto;
    }

    /// <summary>
    /// Repairs the cross-entity rules that a migration or hand edit can break, and reports what was
    /// changed (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>, "Validation").
    /// </summary>
    /// <param name="config">The configuration as loaded.</param>
    /// <param name="fallbackPageId">Page to use when a Drape points at a missing one.</param>
    /// <returns>The repaired configuration and whether anything changed.</returns>
    public static (DrapeConfig Config, bool Changed) Reconcile(DrapeConfig config, Guid fallbackPageId)
    {
        ArgumentNullException.ThrowIfNull(config);

        var pageIds = config.Pages.Select(page => page.Id).ToHashSet();
        if (!pageIds.Contains(fallbackPageId))
        {
            fallbackPageId = config.Pages[0].Id;
        }

        var changed = false;
        var repaired = new List<DrapeEntity>(config.Drapes.Count);

        foreach (var drape in config.Drapes)
        {
            if (!pageIds.Contains(drape.PageId))
            {
                repaired.Add(drape with { PageId = fallbackPageId });
                changed = true;
                continue;
            }

            repaired.Add(drape);
        }

        var rebuilt = changed
            ? new DrapeConfig(config.SchemaVersion, config.Pages, repaired, config.Settings)
            : config;

        return (rebuilt, changed);
    }
}
