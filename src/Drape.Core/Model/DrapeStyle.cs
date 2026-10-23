namespace Drape.Core.Model;

/// <summary>How labels under an item are shown.</summary>
public enum LabelMode
{
    /// <summary>No label.</summary>
    None,

    /// <summary>One line, ellipsised.</summary>
    OneLine,

    /// <summary>Two lines, ellipsised.</summary>
    TwoLines,
}

/// <summary>How the items in a Drape are ordered.</summary>
public enum SortMode
{
    /// <summary>The order the user arranged by hand.</summary>
    Manual,

    /// <summary>Alphabetical by display name.</summary>
    Name,

    /// <summary>Newest first by added date.</summary>
    DateAdded,
}

/// <summary>The visual style of a Drape (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>).</summary>
/// <remarks>
/// Colours are stored as strings so Core carries no UI-framework colour type (CS-22). Parsing them
/// is the UI's job; an unparseable value must fall back rather than throw.
/// </remarks>
public sealed record DrapeStyle
{
    /// <summary>The smallest icon edge, in DIP.</summary>
    public const double MinIconSize = 16;

    /// <summary>The largest icon edge, in DIP.</summary>
    public const double MaxIconSize = 256;

    /// <summary>A style with the documented defaults.</summary>
    public static DrapeStyle Default { get; } = new();

    /// <summary>Tint colour, <c>#RRGGBB</c> or <c>#AARRGGBB</c>; <c>null</c> means "no tint".</summary>
    public string? Tint { get; init; } = "#3A6EA5";

    /// <summary>Panel opacity, 0..1.</summary>
    public double Opacity { get; init; } = 0.45;

    /// <summary>Blur kind requested by the user; whether it can be honoured is a capability question.</summary>
    public string? Blur { get; init; } = "acrylic";

    /// <summary>Corner radius in DIP.</summary>
    public double CornerRadius { get; init; } = 8;

    /// <summary>Title font family name; <c>null</c> means the theme default.</summary>
    public string? TitleFont { get; init; }

    /// <summary>Title font size in DIP.</summary>
    public double TitleSize { get; init; } = 13;

    /// <summary>Icon edge length in DIP.</summary>
    public double IconSize { get; init; } = 48;

    /// <summary>Gap between items in DIP.</summary>
    public double IconSpacing { get; init; } = 12;

    /// <summary>How labels are drawn.</summary>
    public LabelMode LabelMode { get; init; } = LabelMode.TwoLines;

    /// <summary>Overrides the application theme for this Drape; <c>null</c> follows the application.</summary>
    public string? Theme { get; init; }

    /// <summary>Validates the style, rejecting values the renderer could not use.</summary>
    /// <returns><c>true</c> when the style is usable.</returns>
    /// <exception cref="ArgumentOutOfRangeException">A numeric value is outside its range.</exception>
    public bool Validate()
    {
        if (Opacity is < 0 or > 1)
        {
            throw new ArgumentOutOfRangeException(nameof(Opacity), Opacity, "Opacity must be between 0 and 1.");
        }

        if (CornerRadius < 0)
        {
            throw new ArgumentOutOfRangeException(nameof(CornerRadius), CornerRadius, "Corner radius cannot be negative.");
        }

        if (IconSize is < MinIconSize or > MaxIconSize)
        {
            throw new ArgumentOutOfRangeException(nameof(IconSize), IconSize, $"Icon size must be between {MinIconSize} and {MaxIconSize} DIP.");
        }

        if (IconSpacing < 0)
        {
            throw new ArgumentOutOfRangeException(nameof(IconSpacing), IconSpacing, "Icon spacing cannot be negative.");
        }

        if (TitleSize <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(TitleSize), TitleSize, "Title size must be positive.");
        }

        return true;
    }
}
