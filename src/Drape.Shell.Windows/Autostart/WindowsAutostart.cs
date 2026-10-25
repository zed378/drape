using Drape.Core;
using Microsoft.Win32;

namespace Drape.Shell.Windows.Autostart;

/// <summary>
/// Run-at-login via the HKCU Run key (not the Startup folder: a shortcut there cannot be removed
/// reliably when the executable moves, and the registry value reads back the real state).
/// </summary>
public sealed class WindowsAutostart : IAutostart
{
    private const string RunKeyPath = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "Drape";

    /// <inheritdoc />
    public Task<bool> IsEnabledAsync(CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: false);
        return Task.FromResult(key?.GetValue(ValueName) is not null);
    }

    /// <inheritdoc />
    public Task SetEnabledAsync(bool enabled, CancellationToken ct = default)
    {
        ct.ThrowIfCancellationRequested();
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: true)
            ?? throw new InvalidOperationException("The per-user Run key could not be opened.");

        if (enabled)
        {
            var exe = Environment.ProcessPath
                ?? throw new InvalidOperationException("The current executable path is unknown.");
            key.SetValue(ValueName, $"\"{exe}\"", RegistryValueKind.String);
        }
        else
        {
            key.DeleteValue(ValueName, throwOnMissingValue: false);
        }

        return Task.CompletedTask;
    }
}
