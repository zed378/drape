namespace Drape.Core.Model;

/// <summary>How often the app checks for updates (ADR-017).</summary>
public enum UpdateCheckMode
{
    /// <summary>Never; the default.</summary>
    Off,

    /// <summary>Only when the user asks.</summary>
    Manual,

    /// <summary>Periodically in the background.</summary>
    Periodic,
}

/// <summary>What happens to a file dropped onto a Drape.</summary>
public enum DropMode
{
    /// <summary>Only a reference is stored; the file stays where it is.</summary>
    Reference,
}

/// <summary>Application-wide settings (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>).</summary>
public sealed record DrapeSettings
{
    /// <summary>Settings with the documented defaults.</summary>
    public static DrapeSettings Default { get; } = new();

    /// <summary>UI language tag, for example <c>en-US</c> or <c>id-ID</c>.</summary>
    public string Language { get; init; } = "en-US";

    /// <summary>Theme name, or <c>system</c>.</summary>
    public string Theme { get; init; } = "system";

    /// <summary>Whether desktop icons are hidden while Drape is active.</summary>
    public bool HideIconsWhenActive { get; init; } = true;

    /// <summary>Whether the quick-hide shortcut is registered.</summary>
    public bool QuickHideEnabled { get; init; } = true;

    /// <summary>Whether Drape starts with Windows.</summary>
    public bool Autostart { get; init; }

    /// <summary>Whether the inbox Drape exists.</summary>
    public bool InboxEnabled { get; init; } = true;

    /// <summary>What a drop does.</summary>
    public DropMode DropMode { get; init; } = DropMode.Reference;

    /// <summary>Minimum log level name.</summary>
    public string LogLevel { get; init; } = "Information";

    /// <summary>Update-check behaviour; off by default, because ADR-010 forbids network by default.</summary>
    public UpdateCheckMode UpdateCheck { get; init; } = UpdateCheckMode.Off;
}
