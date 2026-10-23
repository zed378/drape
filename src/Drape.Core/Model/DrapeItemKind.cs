namespace Drape.Core.Model;

/// <summary>What an item points at.</summary>
public enum DrapeItemKind
{
    /// <summary>A file.</summary>
    File,

    /// <summary>A folder.</summary>
    Folder,

    /// <summary>A shortcut or link.</summary>
    Shortcut,

    /// <summary>A shell item with no file-system path (This PC, Recycle Bin). Deferred beyond the MVP.</summary>
    Virtual,
}
