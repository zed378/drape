using System.Text.Json;
using Drape.Core;
using Drape.Core.Model;

namespace Drape.Infrastructure.Configuration;

/// <summary>
/// <see cref="IConfigStore"/> over <see cref="IFileSystem"/>, with atomic replacement, rolling
/// backups and quarantine (<c>docs/ARCHITECTURE/02-DATA-MODEL.md</c>, ADR-005).
/// </summary>
/// <remarks>
/// <para>
/// The write sequence is: serialise to <c>config.json.tmp</c>, flush it, then <c>File.Replace</c>
/// the live file with the temporary one, keeping the previous contents as a backup. The live file is
/// therefore never half-written: a crash leaves either the old file or the new one.
/// </para>
/// <para>
/// The store never deletes the user's data on a parse failure. The corrupt file is moved to
/// <c>quarantine\</c> and kept for diagnosis.
/// </para>
/// </remarks>
public sealed class FileConfigStore : IConfigStore
{
    /// <summary>How many backups are kept.</summary>
    public const int MaxBackups = 10;

    private const string ConfigFileName = "config.json";
    private const string BackupFolderName = "backup";
    private const string QuarantineFolderName = "quarantine";
    private const string BackupPrefix = "config-";
    private const string BackupExtension = ".json";

    private readonly IFileSystem _fileSystem;
    private readonly IClock _clock;
    private readonly string _directory;

    /// <summary>
    /// Root-level properties read from disk that this build does not know. They are written back on
    /// the next save so that a downgrade does not silently drop a newer build's data.
    /// </summary>
    private IReadOnlyDictionary<string, JsonElement> _unknownRoot = new Dictionary<string, JsonElement>();

    /// <summary>Creates a store.</summary>
    /// <param name="fileSystem">The file system to use.</param>
    /// <param name="clock">The clock, used to name backups.</param>
    /// <param name="directory">The application data directory.</param>
    /// <exception cref="ArgumentException">The directory is blank.</exception>
    public FileConfigStore(IFileSystem fileSystem, IClock clock, string directory)
    {
        ArgumentNullException.ThrowIfNull(fileSystem);
        ArgumentNullException.ThrowIfNull(clock);
        ArgumentException.ThrowIfNullOrWhiteSpace(directory);

        _fileSystem = fileSystem;
        _clock = clock;
        _directory = directory;
    }

    /// <summary>The live config path.</summary>
    public string ConfigPath => Combine(_directory, ConfigFileName);

    /// <summary>The directory holding rolling backups.</summary>
    public string BackupDirectory => Combine(_directory, BackupFolderName);

    /// <summary>The directory holding corrupt files.</summary>
    public string QuarantineDirectory => Combine(_directory, QuarantineFolderName);

    private bool IsReadOnly { get; set; }

    /// <inheritdoc />
    public Task<ConfigLoadResult> LoadAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        return Task.FromResult(Load());
    }

    /// <inheritdoc />
    public async Task<bool> SaveAsync(DrapeConfig config, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(config);
        ct.ThrowIfCancellationRequested();

        if (IsReadOnly)
        {
            throw new InvalidOperationException(
                "The configuration was written by a newer version of Drape and is open read-only.");
        }

        // Never downgrade the version on disk: a config loaded as v1 stays v1 until a migration
        // explicitly moves it.
        var toWrite = config.SchemaVersion == DrapeConfig.CurrentSchemaVersion
            ? config
            : config with { SchemaVersion = DrapeConfig.CurrentSchemaVersion };

        await Task.Run(() => Write(toWrite), ct).ConfigureAwait(false);
        return true;
    }

    private ConfigLoadResult Load()
    {
        var path = ConfigPath;

        if (!_fileSystem.FileExists(path))
        {
            return new ConfigLoadResult(CreateFirstRunConfig(), ConfigLoadOutcome.FirstRun);
        }

        string json;
        try
        {
            json = _fileSystem.ReadAllText(path);
        }
        catch (IOException)
        {
            // The file exists but cannot be read: treat it as corrupt rather than overwriting it.
            return Recover("The configuration file could not be read.");
        }

        var version = ConfigJson.ReadSchemaVersion(json);
        if (version is > DrapeConfig.CurrentSchemaVersion)
        {
            // Parse it if possible so the user can still see their layout, but refuse to save.
            IsReadOnly = true;
            var newer = TryParse(json) ?? CreateFirstRunConfig();
            return new ConfigLoadResult(
                newer,
                ConfigLoadOutcome.ReadOnlyNewerSchema,
                Message: "This configuration was written by a newer version of Drape and is open read-only.");
        }

        var config = TryParse(json);
        if (config is null)
        {
            return Recover("The configuration file could not be read.");
        }

        return new ConfigLoadResult(config, ConfigLoadOutcome.Loaded);
    }

    private ConfigLoadResult Recover(string message)
    {
        var quarantined = Quarantine();

        var backup = FindNewestValidBackup();
        if (backup is { } recovered)
        {
            return new ConfigLoadResult(
                recovered,
                ConfigLoadOutcome.RecoveredFromBackup,
                quarantined,
                $"{message} The most recent backup was restored.");
        }

        return new ConfigLoadResult(
            CreateFirstRunConfig(),
            ConfigLoadOutcome.StartedEmpty,
            quarantined,
            $"{message} No usable backup was found, so Drape started with an empty layout. No desktop files were changed.");
    }

    private DrapeConfig? TryParse(string json)
    {
        try
        {
            var (config, unknown) = ConfigJson.Deserialize(json);
            _unknownRoot = unknown;
            return config;
        }
        catch (Exception ex) when (ex is JsonException or ArgumentException or ArgumentOutOfRangeException)
        {
            // ArgumentException covers the domain validators: a document that parses but violates a
            // rule is as unusable as one that does not parse.
            return null;
        }
    }

    private string? Quarantine()
    {
        var path = ConfigPath;
        if (!_fileSystem.FileExists(path))
        {
            return null;
        }

        var stamp = _clock.UtcNow.ToString("yyyyMMdd-HHmmss", System.Globalization.CultureInfo.InvariantCulture);
        var target = Combine(QuarantineDirectory, $"config-{stamp}{BackupExtension}");

        _fileSystem.CreateDirectory(QuarantineDirectory);
        _fileSystem.Move(path, target);
        return target;
    }

    private DrapeConfig? FindNewestValidBackup()
    {
        if (!_fileSystem.DirectoryExists(BackupDirectory))
        {
            return null;
        }

        var backups = _fileSystem.EnumerateFiles(BackupDirectory)
            .Where(path => path.EndsWith(BackupExtension, StringComparison.OrdinalIgnoreCase))
            // The name carries a sortable timestamp, so ordering by name is ordering by time.
            .OrderByDescending(path => path, StringComparer.OrdinalIgnoreCase);

        foreach (var backup in backups)
        {
            try
            {
                var parsed = TryParse(_fileSystem.ReadAllText(backup));
                if (parsed is not null)
                {
                    return parsed;
                }
            }
            catch (IOException)
            {
                // A backup that cannot be read is skipped; the next one may be fine.
            }
        }

        return null;
    }

    private void Write(DrapeConfig config)
    {
        var path = ConfigPath;
        var temporary = path + ".tmp";

        _fileSystem.CreateDirectory(_directory);

        var json = ConfigJson.Serialize(config, _unknownRoot);
        _fileSystem.WriteAllText(temporary, json);

        if (!_fileSystem.FileExists(path))
        {
            // First write: there is nothing to replace, so an atomic move is the equivalent.
            _fileSystem.Move(temporary, path);
            return;
        }

        _fileSystem.CreateDirectory(BackupDirectory);
        var backup = Combine(BackupDirectory, $"{BackupPrefix}{_clock.UtcNow:yyyyMMdd-HHmmss}{BackupExtension}");
        _fileSystem.Replace(temporary, path, backup);

        PruneBackups();
    }

    private void PruneBackups()
    {
        var backups = _fileSystem.EnumerateFiles(BackupDirectory)
            .Where(path => path.EndsWith(BackupExtension, StringComparison.OrdinalIgnoreCase))
            .OrderByDescending(path => path, StringComparer.OrdinalIgnoreCase)
            .ToList();

        foreach (var stale in backups.Skip(MaxBackups))
        {
            _fileSystem.Delete(stale);
        }
    }

    private DrapeConfig CreateFirstRunConfig()
        => DrapeConfig.CreateEmpty(Guid.NewGuid());

    private static string Combine(string directory, string name)
        => directory.TrimEnd('\\', '/') + "\\" + name;
}
