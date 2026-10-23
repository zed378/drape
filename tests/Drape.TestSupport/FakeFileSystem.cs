using Drape.Core;

namespace Drape.TestSupport;

/// <summary>
/// An in-memory file system that records every call, including deletions.
/// </summary>
/// <remarks>
/// It exposes <see cref="Deletions"/> rather than omitting <see cref="Delete"/>: a fake with no
/// delete method makes a "no file was deleted" test impossible to fail, which is the abuse case
/// named in P1-07.
/// </remarks>
public sealed class FakeFileSystem : IFileSystem
{
    private readonly Dictionary<string, string> _files = new(StringComparer.OrdinalIgnoreCase);
    private readonly HashSet<string> _directories = new(StringComparer.OrdinalIgnoreCase);

    /// <inheritdoc />
    public bool IsCaseInsensitive { get; init; } = true;

    /// <summary>Every path passed to <see cref="Delete"/>, whether or not it existed.</summary>
    public List<string> Deletions { get; } = [];

    /// <summary>Every path passed to <see cref="Replace"/>.</summary>
    public List<(string Source, string Destination, string Backup)> Replaces { get; } = [];

    /// <summary>When set, the next <see cref="Replace"/> throws instead of replacing.</summary>
    public bool FailNextReplace { get; set; }

    /// <summary>When set, the next <see cref="WriteAllText"/> throws instead of writing.</summary>
    public bool FailNextWrite { get; set; }

    /// <summary>Adds a file directly, bypassing the API under test.</summary>
    /// <param name="path">The path to create.</param>
    /// <param name="contents">Its contents.</param>
    public void AddFile(string path, string contents) => _files[Normalize(path)] = contents;

    /// <summary>Adds a directory directly.</summary>
    /// <param name="path">The directory to create.</param>
    public void AddDirectory(string path) => _directories.Add(Normalize(path));

    /// <inheritdoc />
    public bool FileExists(string path) => _files.ContainsKey(Normalize(path));

    /// <inheritdoc />
    public bool DirectoryExists(string path) => _directories.Contains(Normalize(path));

    /// <inheritdoc />
    public string ReadAllText(string path) => _files.TryGetValue(Normalize(path), out var contents)
        ? contents
        : throw new FileNotFoundException($"No such file: {path}", path);

    /// <inheritdoc />
    public void WriteAllText(string path, string contents)
    {
        if (FailNextWrite)
        {
            FailNextWrite = false;
            throw new IOException("Simulated write failure.");
        }

        _files[Normalize(path)] = contents;
    }

    /// <inheritdoc />
    public void Replace(string source, string destination, string backup)
    {
        Replaces.Add((source, destination, backup));

        if (FailNextReplace)
        {
            FailNextReplace = false;
            throw new IOException("Simulated replace failure.");
        }

        var normalizedSource = Normalize(source);
        if (!_files.TryGetValue(normalizedSource, out var incoming))
        {
            throw new FileNotFoundException($"No such file: {source}", source);
        }

        var normalizedDestination = Normalize(destination);
        if (_files.TryGetValue(normalizedDestination, out var existing))
        {
            _files[Normalize(backup)] = existing;
        }

        _files[normalizedDestination] = incoming;
        _files.Remove(normalizedSource);
    }

    /// <inheritdoc />
    public void Move(string source, string destination)
    {
        var normalizedSource = Normalize(source);
        if (!_files.TryGetValue(normalizedSource, out var contents))
        {
            throw new FileNotFoundException($"No such file: {source}", source);
        }

        _files[Normalize(destination)] = contents;
        _files.Remove(normalizedSource);
    }

    /// <inheritdoc />
    public void Delete(string path)
    {
        var normalized = Normalize(path);
        Deletions.Add(normalized);
        _files.Remove(normalized);
    }

    /// <inheritdoc />
    public void CreateDirectory(string path) => _directories.Add(Normalize(path));

    /// <inheritdoc />
    public IReadOnlyList<string> EnumerateFiles(string path)
    {
        var prefix = Normalize(path).TrimEnd('/') + "/";
        return [.. _files.Keys.Where(file => file.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))];
    }

    /// <inheritdoc />
    public string NormalizePath(string path) => Normalize(path);

    private string Normalize(string path)
    {
        var value = path.Replace('\\', '/').TrimEnd('/');
        return IsCaseInsensitive ? value.ToLowerInvariant() : value;
    }
}
