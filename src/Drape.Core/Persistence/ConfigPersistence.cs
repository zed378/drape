using Drape.Core.Model;

namespace Drape.Core.Persistence;

/// <summary>
/// Debounced saving with a synchronous flush on shutdown (P1-11).
/// </summary>
/// <remarks>
/// <para>
/// A debounce without a flush is how layouts get lost: the user moves a Drape, closes the app
/// within the debounce window, and the change never reaches disk. <see cref="FlushAsync"/> must be
/// called on the shutdown path, before the process exits.
/// </para>
/// <para>
/// The type is deliberately not a timer loop. It exposes the decision — "is a save due?" — so a test
/// can drive it with a fake clock instead of sleeping (CS-45).
/// </para>
/// </remarks>
public sealed class ConfigPersistence
{
    /// <summary>The debounce interval from the data model.</summary>
    public static readonly TimeSpan DebounceInterval = TimeSpan.FromMilliseconds(500);

    private readonly IConfigStore _store;
    private readonly IClock _clock;
    private readonly object _gate = new();

    private DrapeConfig? _pending;
    private DateTimeOffset? _dueAt;
    private DrapeConfig? _lastSaved;

    /// <summary>Creates a persistence service.</summary>
    /// <param name="store">The store to write to.</param>
    /// <param name="clock">The clock that drives the debounce.</param>
    public ConfigPersistence(IConfigStore store, IClock clock)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(clock);

        _store = store;
        _clock = clock;
    }

    /// <summary>Whether a save is pending.</summary>
    public bool HasPendingChanges
    {
        get
        {
            lock (_gate)
            {
                return _pending is not null;
            }
        }
    }

    /// <summary>Queues a document to be saved after the debounce interval.</summary>
    /// <param name="config">The document to save.</param>
    /// <remarks>Repeated calls within the interval reset it: only the last document is written.</remarks>
    public void Queue(DrapeConfig config)
    {
        ArgumentNullException.ThrowIfNull(config);

        lock (_gate)
        {
            _pending = config;
            _dueAt = _clock.UtcNow + DebounceInterval;
        }
    }

    /// <summary>
    /// Saves the pending document if the debounce interval has elapsed.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> when a save happened.</returns>
    public async Task<bool> SaveIfDueAsync(CancellationToken ct = default)
    {
        DrapeConfig? toSave;
        lock (_gate)
        {
            if (_pending is null || _dueAt is null || _clock.UtcNow < _dueAt)
            {
                return false;
            }

            toSave = _pending;
            _pending = null;
            _dueAt = null;
        }

        await _store.SaveAsync(toSave, ct).ConfigureAwait(false);
        _lastSaved = toSave;
        return true;
    }

    /// <summary>
    /// Saves the pending document immediately, ignoring the debounce.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> when a save happened.</returns>
    /// <remarks>This is the shutdown path. It must be awaited before the process exits.</remarks>
    public async Task<bool> FlushAsync(CancellationToken ct = default)
    {
        DrapeConfig? toSave;
        lock (_gate)
        {
            toSave = _pending;
            _pending = null;
            _dueAt = null;
        }

        if (toSave is null)
        {
            return false;
        }

        await _store.SaveAsync(toSave, ct).ConfigureAwait(false);
        _lastSaved = toSave;
        return true;
    }

    /// <summary>The document most recently written, for tests and diagnostics.</summary>
    public DrapeConfig? LastSaved => _lastSaved;
}
