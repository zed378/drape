using Drape.Core;
using Drape.Core.Persistence;
using Drape.Infrastructure.Configuration;
using Drape.Shell.Windows;
using Drape.Shell.Windows.Autostart;
using Drape.Shell.Windows.Locations;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Serilog;

namespace Drape.App.Desktop;

/// <summary>
/// Composition root for the Windows desktop app (ADR-024: one thin head per OS).
/// </summary>
/// <remarks>
/// This is the only place allowed to know which platform it runs on (CS-21). Everything else
/// talks through Core interfaces injected here.
/// </remarks>
public static class Program
{
    /// <summary>Named mutex: a second instance exits and signals the first.</summary>
    internal const string SingleInstanceMutexName = @"Local\Drape.SingleInstance";

    /// <summary>CLI flag that restores desktop icons and exits (P2-03 implements the behaviour).</summary>
    internal const string RestoreDesktopFlag = "--restore-desktop";

    /// <summary>Main application entry point.</summary>
    /// <param name="args">Command-line arguments.</param>
    /// <returns>Process exit code.</returns>
    public static async Task<int> Main(string[] args)
    {
        if (args.Contains(RestoreDesktopFlag, StringComparer.OrdinalIgnoreCase))
        {
            return await RunRestoreDesktopAsync();
        }

        using var mutex = new Mutex(initiallyOwned: true, SingleInstanceMutexName, out var isFirst);
        if (!isFirst)
        {
            Console.WriteLine("Drape is already running.");
            return 0;
        }

        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            try
            {
                Log.Fatal(e.ExceptionObject as Exception, "Unhandled exception; exiting");
                Log.CloseAndFlush();
            }
            catch
            {
                // Logging must never throw from an exception handler.
            }
        };

        var locations = new WindowsDesktopLocations();
        ConfigureLogging(locations.AppDataDirectory);

        try
        {
            Log.Information("Drape starting");
            var host = CreateHost(args, locations);
            await host.RunAsync();
            Log.Information("Drape stopped normally");
            return 0;
        }
        catch (Exception ex)
        {
            Log.Fatal(ex, "Drape terminated unexpectedly");
            return 1;
        }
        finally
        {
            Log.CloseAndFlush();
        }
    }

    /// <summary>Builds the generic host with every service the app needs.</summary>
    /// <param name="args">Command-line arguments.</param>
    /// <param name="locations">Platform locations (created before logging so the log path is known).</param>
    /// <returns>The configured host.</returns>
    internal static IHost CreateHost(string[] args, IDesktopLocations locations)
    {
        ArgumentNullException.ThrowIfNull(locations);

        return Host.CreateDefaultBuilder(args)
            .UseSerilog()
            .ConfigureServices(services => ConfigureServices(services, locations))
            .Build();
    }

    private static void ConfigureServices(IServiceCollection services, IDesktopLocations locations)
    {
        services.AddSingleton(locations);
        services.AddSingleton<IPlatformCapabilities, WindowsPlatformCapabilities>();
        services.AddSingleton<IDesktopLocations>(_ => locations);
        services.AddSingleton<IAutostart, WindowsAutostart>();
        services.AddSingleton<IClock, SystemClock>();
        services.AddSingleton<IFileSystem, PhysicalFileSystem>();
        services.AddSingleton<IConfigStore>(provider => new FileConfigStore(
            provider.GetRequiredService<IFileSystem>(),
            provider.GetRequiredService<IClock>(),
            locations.AppDataDirectory));
        services.AddSingleton<ConfigPersistence>();
    }

    private static void ConfigureLogging(string appDataDirectory)
    {
        var logDirectory = Path.Combine(appDataDirectory, "logs");
        Directory.CreateDirectory(logDirectory);

        Log.Logger = new LoggerConfiguration()
            .MinimumLevel.Information()
            .Enrich.FromLogContext()
            .WriteTo.Console()
            .WriteTo.File(
                Path.Combine(logDirectory, "Drape-.log"),
                rollingInterval: RollingInterval.Day,
                retainedFileCountLimit: 14)
            .CreateLogger();
    }

    private static Task<int> RunRestoreDesktopAsync()
    {
        // P2-03 implements the real restore. Until then the flag is parsed and dispatched,
        // and the user is told what happened instead of failing silently.
        Console.WriteLine("Drape --restore-desktop: icon restore is not implemented yet (P2-03).");
        return Task.FromResult(2);
    }

    /// <summary>Clock over the system time, for production use.</summary>
    private sealed class SystemClock : IClock
    {
        /// <inheritdoc />
        public DateTimeOffset UtcNow => DateTimeOffset.UtcNow;
    }

    /// <summary>File system over the real disk, for production use.</summary>
    private sealed class PhysicalFileSystem : IFileSystem
    {
        /// <inheritdoc />
        public bool IsCaseInsensitive => OperatingSystem.IsWindows() || OperatingSystem.IsMacOS();

        /// <inheritdoc />
        public bool FileExists(string path) => File.Exists(path);

        /// <inheritdoc />
        public bool DirectoryExists(string path) => Directory.Exists(path);

        /// <inheritdoc />
        public string ReadAllText(string path) => File.ReadAllText(path);

        /// <inheritdoc />
        public void WriteAllText(string path, string contents)
        {
            var directory = Path.GetDirectoryName(path);
            if (directory is not null)
            {
                Directory.CreateDirectory(directory);
            }

            File.WriteAllText(path, contents);
        }

        /// <inheritdoc />
        public void Replace(string source, string destination, string backup)
            => File.Replace(source, destination, backup);

        /// <inheritdoc />
        public void Move(string source, string destination) => File.Move(source, destination, overwrite: true);

        /// <inheritdoc />
        public void Delete(string path)
        {
            if (File.Exists(path))
            {
                File.Delete(path);
            }
        }

        /// <inheritdoc />
        public void CreateDirectory(string path) => Directory.CreateDirectory(path);

        /// <inheritdoc />
        public IReadOnlyList<string> EnumerateFiles(string path) => Directory.GetFiles(path);

        /// <inheritdoc />
        public string NormalizePath(string path)
        {
            var expanded = Environment.ExpandEnvironmentVariables(path);
            var full = Path.GetFullPath(expanded);
            var trimmed = full.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
            return IsCaseInsensitive ? trimmed.ToLowerInvariant() : trimmed;
        }
    }
}
