using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace Drape.Guard;

/// <summary>
/// Main application entry point for Drape.Guard.
/// </summary>
public static class Program
{
    public static async Task Main(string[] args)
    {
        try
        {
            var host = Host.CreateDefaultBuilder(args)
                .ConfigureServices(ConfigureServices)
                .Build();

            await host.RunAsync();
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Drape.Guard terminated unexpectedly: {ex}");
            Environment.Exit(1);
        }
    }

    private static void ConfigureServices(IServiceCollection services)
    {
        // TODO: Register Drape.Guard services (Phase 0 setup only)
    }
}