namespace Drape.Core;

/// <summary>
/// The file-system operations Core is allowed to perform (CS-17).
/// </summary>
/// <remarks>
/// <para>
/// Core never calls <see cref="System.IO"/> directly. Two reasons: it must stay portable, and the
/// atomic-write behaviour in <see cref="Replace"/> has different platform primitives that belong at
/// this seam rather than in the store.
/// </para>
/// <para>All methods may block; callers are responsible for running them off the UI thread.</para>
/// </remarks>
public interface IFileSystem
{
    /// <summary>Whether the path exists as a file.</summary>
    /// <param name="path">An absolute path.</param>
    /// <returns><c>true</c> when a file exists there.</returns>
    bool FileExists(string path);

    /// <summary>Whether the path exists as a directory.</summary>
    /// <param name="path">An absolute path.</param>
    /// <returns><c>true</c> when a directory exists there.</returns>
    bool DirectoryExists(string path);

    /// <summary>Reads a whole file as text.</summary>
    /// <param name="path">An absolute path.</param>
    /// <returns>The file contents.</returns>
    string ReadAllText(string path);

    /// <summary>
    /// Writes text to a new or existing file, creating parent directories as needed, and flushes it
    /// to disk.
    /// </summary>
    /// <param name="path">An absolute path.</param>
    /// <param name="contents">The text to write.</param>
    void WriteAllText(string path, string contents);

    /// <summary>
    /// Atomically replaces <paramref name="destination"/> with <paramref name="source"/>, keeping the
    /// old file at <paramref name="backup"/>.
    /// </summary>
    /// <param name="source">The fully written and flushed temporary file.</param>
    /// <param name="destination">The live file to replace.</param>
    /// <param name="backup">Where the previous contents are kept.</param>
    void Replace(string source, string destination, string backup);

    /// <summary>Moves a file, overwriting the destination if it exists.</summary>
    /// <param name="source">The file to move.</param>
    /// <param name="destination">The new location.</param>
    void Move(string source, string destination);

    /// <summary>Deletes a file if it exists. Does nothing when it does not.</summary>
    /// <param name="path">The file to delete.</param>
    void Delete(string path);

    /// <summary>Creates a directory and any missing parents. Does nothing when it exists.</summary>
    /// <param name="path">The directory to create.</param>
    void CreateDirectory(string path);

    /// <summary>Lists the files in a directory, without recursing.</summary>
    /// <param name="path">An existing directory.</param>
    /// <returns>The full paths of the files it contains.</returns>
    IReadOnlyList<string> EnumerateFiles(string path);

    /// <summary>
    /// Whether path comparison is case-insensitive on this platform. Windows and macOS are
    /// case-insensitive by default; Linux is not.
    /// </summary>
    bool IsCaseInsensitive { get; }

    /// <summary>
    /// Normalises a path for identity comparison: expands environment variables, uses the platform
    /// separator and removes a trailing separator.
    /// </summary>
    /// <param name="path">A path as given by the user or the shell.</param>
    /// <returns>The normalised path.</returns>
    string NormalizePath(string path);
}
