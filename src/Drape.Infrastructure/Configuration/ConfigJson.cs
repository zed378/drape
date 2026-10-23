using System.Text.Json;
using System.Text.Json.Serialization;
using Drape.Core.Model;
using DrapeEntity = Drape.Core.Model.Drape;
using DropModeValue = Drape.Core.Model.DropMode;
using LabelModeValue = Drape.Core.Model.LabelMode;
using SortModeValue = Drape.Core.Model.SortMode;

namespace Drape.Infrastructure.Configuration;

/// <summary>
/// The on-disk shape of <c>config.json</c> and the translation to and from the domain model.
/// </summary>
/// <remarks>
/// <para>
/// This is deliberately a separate set of DTOs rather than serialising the records directly. The
/// records have validation in their constructors and read-only collections; a serialiser that
/// writes them directly would either bypass the validation or need the model shaped around JSON,
/// and the schema would then be impossible to migrate without changing the domain.
/// </para>
/// <para>
/// Unknown properties are captured in <see cref="ConfigDto.Unknown"/> via
/// <see cref="JsonExtensionDataAttribute"/> and written back on save, so a downgrade does not drop
/// data a newer build wrote (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>).
/// </para>
/// </remarks>
internal static class ConfigJson
{
    /// <summary>Options used for both reading and writing.</summary>
    internal static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.CamelCase) },
    };

    /// <summary>Serialises a configuration.</summary>
    /// <param name="config">The configuration to write.</param>
    /// <param name="unknownRoot">Root-level properties read from disk that this build does not know.</param>
    /// <returns>The JSON text.</returns>
    public static string Serialize(DrapeConfig config, IReadOnlyDictionary<string, JsonElement>? unknownRoot = null)
    {
        ArgumentNullException.ThrowIfNull(config);

        var dto = new ConfigDto
        {
            SchemaVersion = config.SchemaVersion,
            Settings = SettingsDto.From(config.Settings),
            Pages = [.. config.Pages.Select(page => new PageDto { Id = page.Id, Name = page.Name, Order = page.Order })],
            Drapes = [.. config.Drapes.Select(DrapeDto.From)],
            Unknown = unknownRoot is null ? null : new Dictionary<string, JsonElement>(unknownRoot),
        };

        return JsonSerializer.Serialize(dto, Options);
    }

    /// <summary>Reads the schema version without parsing the rest, so a newer file is not mangled.</summary>
    /// <param name="json">The JSON text.</param>
    /// <returns>The version, or <c>null</c> when it cannot be read.</returns>
    public static int? ReadSchemaVersion(string json)
    {
        try
        {
            using var document = JsonDocument.Parse(json);
            if (document.RootElement.ValueKind != JsonValueKind.Object)
            {
                return null;
            }

            foreach (var property in document.RootElement.EnumerateObject())
            {
                if (string.Equals(property.Name, "schemaVersion", StringComparison.OrdinalIgnoreCase)
                    && property.Value.TryGetInt32(out var version))
                {
                    return version;
                }
            }

            return null;
        }
        catch (JsonException)
        {
            return null;
        }
    }

    /// <summary>Parses a configuration document.</summary>
    /// <param name="json">The JSON text.</param>
    /// <returns>The parsed document and its unknown root properties.</returns>
    /// <exception cref="JsonException">The document is not valid or violates a domain rule.</exception>
    public static (DrapeConfig Config, IReadOnlyDictionary<string, JsonElement> Unknown) Deserialize(string json)
    {
        var dto = JsonSerializer.Deserialize<ConfigDto>(json, Options)
            ?? throw new JsonException("The config file was empty.");

        if (dto.SchemaVersion <= 0)
        {
            throw new JsonException("The config file has no usable schemaVersion.");
        }

        var pages = (dto.Pages ?? []).Select(page => new Page(page.Id, page.Name, page.Order)).ToList();
        if (pages.Count == 0)
        {
            throw new JsonException("The config file has no pages.");
        }

        var drapes = (dto.Drapes ?? []).Select(drape => drape.ToModel()).ToList();
        var config = new DrapeConfig(dto.SchemaVersion, pages, drapes, dto.Settings?.ToModel() ?? DrapeSettings.Default);

        var unknown = dto.Unknown is null
            ? new Dictionary<string, JsonElement>()
            : new Dictionary<string, JsonElement>(dto.Unknown);

        return (config, unknown);
    }

    /// <summary>The root document.</summary>
    internal sealed class ConfigDto
    {
        /// <summary>Schema version.</summary>
        public int SchemaVersion { get; set; }

        /// <summary>Settings.</summary>
        public SettingsDto? Settings { get; set; }

        /// <summary>Pages.</summary>
        public List<PageDto>? Pages { get; set; }

        /// <summary>Drapes.</summary>
        public List<DrapeDto>? Drapes { get; set; }

        /// <summary>Properties this build does not know; preserved on save.</summary>
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? Unknown { get; set; }
    }

    /// <summary>A page on disk.</summary>
    internal sealed class PageDto
    {
        /// <summary>Id.</summary>
        public Guid Id { get; set; }

        /// <summary>Name.</summary>
        public string Name { get; set; } = string.Empty;

        /// <summary>Order.</summary>
        public int Order { get; set; }
    }

    /// <summary>A Drape on disk.</summary>
    internal sealed class DrapeDto
    {
        /// <summary>Id.</summary>
        public Guid Id { get; set; }

        /// <summary>Owning page id.</summary>
        public Guid PageId { get; set; }

        /// <summary>Title.</summary>
        public string Title { get; set; } = string.Empty;

        /// <summary>Kind.</summary>
        public string Kind { get; set; } = "normal";

        /// <summary>Bounds.</summary>
        public BoundsDto? Bounds { get; set; }

        /// <summary>Style.</summary>
        public StyleDto? Style { get; set; }

        /// <summary>State.</summary>
        public StateDto? State { get; set; }

        /// <summary>Whether the Drape is excluded from quick-hide.</summary>
        public bool QuickHideExcluded { get; set; }

        /// <summary>Items.</summary>
        public List<ItemDto>? Items { get; set; }

        /// <summary>Builds the domain record.</summary>
        /// <returns>The Drape.</returns>
        /// <exception cref="JsonException">Required fields are missing.</exception>
        public DrapeEntity ToModel()
        {
            if (Bounds is null)
            {
                throw new JsonException($"Drape {Id} has no bounds.");
            }

            var drape = new DrapeEntity(
                Id,
                PageId,
                Title,
                Bounds.ToModel(),
                ParseKind(Kind),
                Style: Style?.ToModel() ?? DrapeStyle.Default,
                State: State?.ToModel() ?? DrapeState.Default,
                QuickHideExcluded: QuickHideExcluded);

            return drape with
            {
                Items = [.. (Items ?? []).Select(item => item.ToModel())],
            };
        }

        /// <summary>Builds the DTO from a domain record.</summary>
        /// <param name="drape">The Drape.</param>
        /// <returns>The DTO.</returns>
        public static DrapeDto From(DrapeEntity drape) => new()
        {
            Id = drape.Id,
            PageId = drape.PageId,
            Title = drape.Title,
            Kind = drape.Kind.ToString().ToLowerInvariant(),
            Bounds = BoundsDto.From(drape.Bounds),
            Style = StyleDto.From(drape.Style),
            State = StateDto.From(drape.State),
            QuickHideExcluded = drape.QuickHideExcluded,
            Items = [.. drape.Items.Select(ItemDto.From)],
        };

        private static DrapeKind ParseKind(string? value) => value?.ToLowerInvariant() switch
        {
            "inbox" => DrapeKind.Inbox,
            "portal" => DrapeKind.Portal,
            _ => DrapeKind.Normal,
        };
    }

    /// <summary>Bounds on disk.</summary>
    internal sealed class BoundsDto
    {
        /// <summary>Monitor id.</summary>
        public string MonitorId { get; set; } = string.Empty;

        /// <summary>Left edge.</summary>
        public double X { get; set; }

        /// <summary>Top edge.</summary>
        public double Y { get; set; }

        /// <summary>Width.</summary>
        public double Width { get; set; }

        /// <summary>Height.</summary>
        public double Height { get; set; }

        /// <summary>DPI.</summary>
        public double Dpi { get; set; } = 96;

        /// <summary>Horizontal fraction.</summary>
        public double Fx { get; set; }

        /// <summary>Vertical fraction.</summary>
        public double Fy { get; set; }

        /// <summary>Width fraction.</summary>
        public double Fw { get; set; }

        /// <summary>Height fraction.</summary>
        public double Fh { get; set; }

        /// <summary>Builds the domain record.</summary>
        /// <returns>The bounds.</returns>
        public DrapeBounds ToModel() => new(MonitorId, X, Y, Width, Height, Dpi, Fx, Fy, Fw, Fh);

        /// <summary>Builds the DTO.</summary>
        /// <param name="bounds">The bounds.</param>
        /// <returns>The DTO.</returns>
        public static BoundsDto From(DrapeBounds bounds) => new()
        {
            MonitorId = bounds.MonitorId,
            X = bounds.X,
            Y = bounds.Y,
            Width = bounds.Width,
            Height = bounds.Height,
            Dpi = bounds.Dpi,
            Fx = bounds.Fx,
            Fy = bounds.Fy,
            Fw = bounds.Fw,
            Fh = bounds.Fh,
        };
    }

    /// <summary>Style on disk.</summary>
    internal sealed class StyleDto
    {
        /// <summary>Tint.</summary>
        public string? Tint { get; set; }

        /// <summary>Opacity.</summary>
        public double Opacity { get; set; } = 0.45;

        /// <summary>Blur kind.</summary>
        public string? Blur { get; set; }

        /// <summary>Corner radius.</summary>
        public double CornerRadius { get; set; } = 8;

        /// <summary>Title font.</summary>
        public string? TitleFont { get; set; }

        /// <summary>Title size.</summary>
        public double TitleSize { get; set; } = 13;

        /// <summary>Icon size.</summary>
        public double IconSize { get; set; } = 48;

        /// <summary>Icon spacing.</summary>
        public double IconSpacing { get; set; } = 12;

        /// <summary>Label mode.</summary>
        public string? LabelMode { get; set; }

        /// <summary>Theme override.</summary>
        public string? Theme { get; set; }

        /// <summary>Builds the domain record.</summary>
        /// <returns>The style.</returns>
        public DrapeStyle ToModel() => new()
        {
            Tint = Tint,
            Opacity = Opacity,
            Blur = Blur,
            CornerRadius = CornerRadius,
            TitleFont = TitleFont,
            TitleSize = TitleSize,
            IconSize = IconSize,
            IconSpacing = IconSpacing,
            LabelMode = ParseLabelMode(this.LabelMode),
            Theme = Theme,
        };

        /// <summary>Builds the DTO.</summary>
        /// <param name="style">The style.</param>
        /// <returns>The DTO.</returns>
        public static StyleDto From(DrapeStyle style) => new()
        {
            Tint = style.Tint,
            Opacity = style.Opacity,
            Blur = style.Blur,
            CornerRadius = style.CornerRadius,
            TitleFont = style.TitleFont,
            TitleSize = style.TitleSize,
            IconSize = style.IconSize,
            IconSpacing = style.IconSpacing,
            LabelMode = style.LabelMode.ToString().ToLowerInvariant(),
            Theme = style.Theme,
        };

        private static LabelModeValue ParseLabelMode(string? value) => value?.ToLowerInvariant() switch
        {
            "none" => LabelModeValue.None,
            "oneline" => LabelModeValue.OneLine,
            _ => LabelModeValue.TwoLines,
        };
    }

    /// <summary>State on disk.</summary>
    internal sealed class StateDto
    {
        /// <summary>Rolled up.</summary>
        public bool RolledUp { get; set; }

        /// <summary>Locked.</summary>
        public bool Locked { get; set; }

        /// <summary>Sort mode.</summary>
        public string? SortMode { get; set; }

        /// <summary>Z-order hint.</summary>
        public int? ZOrderHint { get; set; }

        /// <summary>Builds the domain record.</summary>
        /// <returns>The state.</returns>
        public DrapeState ToModel() => new()
        {
            RolledUp = RolledUp,
            Locked = Locked,
            SortMode = ParseSortMode(this.SortMode),
            ZOrderHint = ZOrderHint,
        };

        /// <summary>Builds the DTO.</summary>
        /// <param name="state">The state.</param>
        /// <returns>The DTO.</returns>
        public static StateDto From(DrapeState state) => new()
        {
            RolledUp = state.RolledUp,
            Locked = state.Locked,
            SortMode = state.SortMode.ToString().ToLowerInvariant(),
            ZOrderHint = state.ZOrderHint,
        };

        private static SortModeValue ParseSortMode(string? value) => value?.ToLowerInvariant() switch
        {
            "name" => SortModeValue.Name,
            "dateadded" => SortModeValue.DateAdded,
            _ => SortModeValue.Manual,
        };
    }

    /// <summary>An item on disk.</summary>
    internal sealed class ItemDto
    {
        /// <summary>Id.</summary>
        public Guid Id { get; set; }

        /// <summary>Kind.</summary>
        public string Kind { get; set; } = "file";

        /// <summary>Path.</summary>
        public string Path { get; set; } = string.Empty;

        /// <summary>Order.</summary>
        public int Order { get; set; }

        /// <summary>Added timestamp.</summary>
        public DateTimeOffset AddedAt { get; set; }

        /// <summary>File id.</summary>
        public string? FileId { get; set; }

        /// <summary>Display name override.</summary>
        public string? DisplayName { get; set; }

        /// <summary>Whether pinned.</summary>
        public bool PinnedByUser { get; set; }

        /// <summary>Missing-since timestamp.</summary>
        public DateTimeOffset? MissingSince { get; set; }

        /// <summary>Builds the domain record.</summary>
        /// <returns>The item.</returns>
        public DrapeItem ToModel() => new(
            Id,
            ParseKind(Kind),
            Path,
            Order,
            AddedAt,
            FileId,
            DisplayName,
            PinnedByUser,
            MissingSince);

        /// <summary>Builds the DTO.</summary>
        /// <param name="item">The item.</param>
        /// <returns>The DTO.</returns>
        public static ItemDto From(DrapeItem item) => new()
        {
            Id = item.Id,
            Kind = item.Kind.ToString().ToLowerInvariant(),
            Path = item.Path,
            Order = item.Order,
            AddedAt = item.AddedAt,
            FileId = item.FileId,
            DisplayName = item.DisplayName,
            PinnedByUser = item.PinnedByUser,
            MissingSince = item.MissingSince,
        };

        private static DrapeItemKind ParseKind(string? value) => value?.ToLowerInvariant() switch
        {
            "folder" => DrapeItemKind.Folder,
            "shortcut" => DrapeItemKind.Shortcut,
            "virtual" => DrapeItemKind.Virtual,
            _ => DrapeItemKind.File,
        };
    }

    /// <summary>Settings on disk.</summary>
    internal sealed class SettingsDto
    {
        /// <summary>Language.</summary>
        public string Language { get; set; } = "en-US";

        /// <summary>Theme.</summary>
        public string Theme { get; set; } = "system";

        /// <summary>Hide icons when active.</summary>
        public bool HideIconsWhenActive { get; set; } = true;

        /// <summary>Quick-hide enabled.</summary>
        public bool QuickHideEnabled { get; set; } = true;

        /// <summary>Autostart.</summary>
        public bool Autostart { get; set; }

        /// <summary>Inbox enabled.</summary>
        public bool InboxEnabled { get; set; } = true;

        /// <summary>Drop mode.</summary>
        public string DropMode { get; set; } = "reference";

        /// <summary>Log level.</summary>
        public string LogLevel { get; set; } = "Information";

        /// <summary>Update check mode.</summary>
        public string UpdateCheck { get; set; } = "off";

        /// <summary>Builds the domain record.</summary>
        /// <returns>The settings.</returns>
        public DrapeSettings ToModel() => new()
        {
            Language = Language,
            Theme = Theme,
            HideIconsWhenActive = HideIconsWhenActive,
            QuickHideEnabled = QuickHideEnabled,
            Autostart = Autostart,
            InboxEnabled = InboxEnabled,
            // Only one drop mode exists in v1; anything else on disk falls back to it rather than
            // failing the whole load over a cosmetic setting.
            DropMode = DropModeValue.Reference,
            LogLevel = LogLevel,
            UpdateCheck = UpdateCheck.ToLowerInvariant() switch
            {
                "manual" => UpdateCheckMode.Manual,
                "periodic" => UpdateCheckMode.Periodic,
                _ => UpdateCheckMode.Off,
            },
        };

        /// <summary>Builds the DTO.</summary>
        /// <param name="settings">The settings.</param>
        /// <returns>The DTO.</returns>
        public static SettingsDto From(DrapeSettings settings) => new()
        {
            Language = settings.Language,
            Theme = settings.Theme,
            HideIconsWhenActive = settings.HideIconsWhenActive,
            QuickHideEnabled = settings.QuickHideEnabled,
            Autostart = settings.Autostart,
            InboxEnabled = settings.InboxEnabled,
            DropMode = settings.DropMode.ToString().ToLowerInvariant(),
            LogLevel = settings.LogLevel,
            UpdateCheck = settings.UpdateCheck.ToString().ToLowerInvariant(),
        };
    }
}
