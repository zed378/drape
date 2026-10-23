namespace Drape.Core.Model;

/// <summary>What kind of Drape this is.</summary>
public enum DrapeKind
{
    /// <summary>An ordinary Drape holding items the user put there.</summary>
    Normal,

    /// <summary>The inbox that collects newly added items.</summary>
    Inbox,

    /// <summary>A view over a folder on disk; its items are not stored individually.</summary>
    Portal,
}
