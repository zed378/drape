using Drape.Core;

namespace Drape.TestSupport;

/// <summary>
/// A clock the test moves by hand, so nothing has to sleep (CS-45).
/// </summary>
public sealed class FakeClock : IClock
{
    /// <summary>Creates a clock fixed at a known instant.</summary>
    /// <param name="start">The initial time; defaults to 2026-01-01T00:00:00Z.</param>
    public FakeClock(DateTimeOffset? start = null)
        => UtcNow = start ?? new DateTimeOffset(2026, 1, 1, 0, 0, 0, TimeSpan.Zero);

    /// <inheritdoc />
    public DateTimeOffset UtcNow { get; private set; }

    /// <summary>Moves the clock forward.</summary>
    /// <param name="delta">How far to move.</param>
    public void Advance(TimeSpan delta)
    {
        if (delta < TimeSpan.Zero)
        {
            throw new ArgumentOutOfRangeException(nameof(delta), delta, "A fake clock does not go backwards.");
        }

        UtcNow += delta;
    }

    /// <summary>Moves the clock forward by a number of milliseconds.</summary>
    /// <param name="milliseconds">How many milliseconds to advance.</param>
    public void AdvanceMilliseconds(double milliseconds) => Advance(TimeSpan.FromMilliseconds(milliseconds));
}
