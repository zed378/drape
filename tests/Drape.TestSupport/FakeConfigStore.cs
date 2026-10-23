using Drape.Core;
using Drape.Core.Model;

namespace Drape.TestSupport;

/// <summary>
/// A config store that keeps the document in memory and can be told to fail or to report a
/// particular load outcome.
/// </summary>
public sealed class FakeConfigStore : IConfigStore
{
    /// <summary>The configuration returned by <see cref="LoadAsync"/>; <c>null</c> means first run.</summary>
    public DrapeConfig? Stored { get; set; }

    /// <summary>The outcome to report from <see cref="LoadAsync"/>.</summary>
    public ConfigLoadOutcome Outcome { get; set; } = ConfigLoadOutcome.Loaded;

    /// <summary>When set, <see cref="SaveAsync"/> throws.</summary>
    public bool FailSave { get; set; }

    /// <summary>How many times <see cref="SaveAsync"/> was called.</summary>
    public int SaveCount { get; private set; }

    /// <summary>The last document passed to <see cref="SaveAsync"/>.</summary>
    public DrapeConfig? LastSaved { get; private set; }

    /// <inheritdoc />
    public Task<ConfigLoadResult> LoadAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();

        var config = Stored ?? DrapeConfig.CreateEmpty(Guid.Parse("6f1c2b8e-0000-4000-8000-000000000001"));
        return Task.FromResult(new ConfigLoadResult(config, Outcome));
    }

    /// <inheritdoc />
    public Task<bool> SaveAsync(DrapeConfig config, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(config);
        ct.ThrowIfCancellationRequested();

        SaveCount++;
        LastSaved = config;

        if (FailSave)
        {
            throw new IOException("Simulated save failure.");
        }

        Stored = config;
        return Task.FromResult(true);
    }
}
