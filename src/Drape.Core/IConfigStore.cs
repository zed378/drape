namespace Drape.Core;

/// <summary>
/// How a config load ended. Callers must be able to tell "loaded", "recovered from a backup" and
/// "started empty" apart, because only the first is silent (CS-14: no swallowed errors).
/// </summary>
public enum ConfigLoadOutcome
{
    /// <summary>The live file was read successfully.</summary>
    Loaded,

    /// <summary>The live file was corrupt; a backup was used and the corrupt file quarantined.</summary>
    RecoveredFromBackup,

    /// <summary>The live file was corrupt and no valid backup existed; an empty config was used.</summary>
    StartedEmpty,

    /// <summary>No config exists yet; this is a first run.</summary>
    FirstRun,

    /// <summary>
    /// The file was written by a newer schema version. It is opened read-only and must never be
    /// overwritten.
    /// </summary>
    ReadOnlyNewerSchema,
}

/// <summary>The result of loading a configuration.</summary>
/// <param name="Config">The configuration that is now in use.</param>
/// <param name="Outcome">How it was obtained.</param>
/// <param name="QuarantinedPath">Where the corrupt file was moved, when there was one.</param>
/// <param name="Message">
/// A short, non-sensitive explanation for the user. Must not contain a file path (CS-37).
/// </param>
public readonly record struct ConfigLoadResult(
    Model.DrapeConfig Config,
    ConfigLoadOutcome Outcome,
    string? QuarantinedPath = null,
    string? Message = null)
{
    /// <summary>Whether saving is allowed. False for a config written by a newer version.</summary>
    public bool CanSave => Outcome != ConfigLoadOutcome.ReadOnlyNewerSchema;
}

/// <summary>
/// Reads and writes the configuration document atomically
/// (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>, ADR-005).
/// </summary>
/// <remarks>
/// <para>
/// Implementations must never leave a half-written live file: write a temporary file, flush it,
/// then replace. A save that fails must leave the previous file readable.
/// </para>
/// <para>
/// <see cref="SaveAsync"/> is not thread-safe; callers serialise their own writes.
/// </para>
/// </remarks>
public interface IConfigStore
{
    /// <summary>Loads the configuration, recovering from corruption as described in the data model.</summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The configuration in use and how it was obtained.</returns>
    Task<ConfigLoadResult> LoadAsync(CancellationToken ct = default);

    /// <summary>Saves the configuration atomically and prunes old backups.</summary>
    /// <param name="config">The configuration to persist.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> when the file was replaced; <c>false</c> when the write was refused.</returns>
    /// <exception cref="InvalidOperationException">The store is read-only.</exception>
    Task<bool> SaveAsync(Model.DrapeConfig config, CancellationToken ct = default);
}
