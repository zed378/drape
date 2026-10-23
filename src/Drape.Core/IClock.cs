namespace Drape.Core;

/// <summary>A clock, so time-dependent behaviour is testable (CS-45).</summary>
/// <remarks>Implementations must be safe to call from any thread.</remarks>
public interface IClock
{
    /// <summary>The current time, with offset.</summary>
    DateTimeOffset UtcNow { get; }
}
