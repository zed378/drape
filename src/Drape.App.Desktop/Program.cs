using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace Drape.App.Desktop;

/// <summary>
/// Main application entry point.
/// </summary>
public static class Program
{
    /// <summary>
    /// Main application entry point.
    /// </summary>
    /// <param name="args">Command-line arguments.</param>
    public static async Task Main(string[] args)
    {
        try
        {
            // Create host with default configuration
            var host = Host.CreateDefaultBuilder(args)
                .ConfigureServices(ConfigureServices)
                .Build();

            // Start the application
            await host.RunAsync();
        }
        catch (Exception ex)
        {
            // Log unhandled exception
            Console.Error.WriteLine($"Application terminated unexpectedly: {ex}");
            Environment.Exit(1);
        }
    }

    private static void ConfigureServices(IServiceCollection services)
    {
        // TODO: Register platform capabilities and services (Phase 0 setup only)
    }
}
