using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Media;
using Avalonia.Threading;
using Windows.Win32.Foundation;

namespace P003DesktopHost;

/// <summary>
/// P0-03 spike harness. Opens a plain Avalonia test window, takes its HWND, and exercises one
/// host strategy. The owner runs this in the VM and fills the results table:
/// <c>dotnet run --project spikes/p0-03-desktop-host -- A|B|C</c>.
/// </summary>
internal static class Program
{
    public static void Main(string[] args)
    {
        var strategy = args.FirstOrDefault()?.ToUpperInvariant() switch
        {
            "A" => HostStrategy.A,
            "B" => HostStrategy.B,
            _ => HostStrategy.C,
        };

        Console.WriteLine($"[p0-03] strategy={strategy} os={Environment.OSVersion}");
        Console.WriteLine($"[p0-03] dpi awareness: check manifest; monitors: {Environment.MachineName}");

        BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
    }

    private static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<SpikeApp>()
            .UsePlatformDetect()
            .LogToTrace();

    private sealed class SpikeApp : Application
    {
        public override void OnFrameworkInitializationCompleted()
        {
            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                var window = new TestWindow();
                desktop.MainWindow = window;
                window.Opened += (_, _) => Dispatcher.UIThread.Post(() => RunStrategy(window));
            }

            base.OnFrameworkInitializationCompleted();
        }

        private static void RunStrategy(TestWindow window)
        {
            var handle = window.TryGetPlatformHandle();
            if (handle is null)
            {
                Console.WriteLine("[p0-03] FAIL: no platform handle");
                return;
            }

            var hwnd = new HWND(handle.Handle);
            Console.WriteLine($"[p0-03] test window HWND={hwnd}");

            var strategy = Environment.GetCommandLineArgs().LastOrDefault()?.ToUpperInvariant() switch
            {
                "A" => HostStrategy.A,
                "B" => HostStrategy.B,
                _ => HostStrategy.C,
            };

            var result = strategy switch
            {
                HostStrategy.A => Strategies.TryA(hwnd),
                HostStrategy.B => Strategies.TryB(hwnd),
                _ => Strategies.TryC(hwnd),
            };

            Console.WriteLine($"[p0-03] RESULT strategy={result.Strategy} attached={result.Attached} detail={result.Detail}");
            Console.WriteLine("[p0-03] Manual checks: Win+D, peek, focus, Alt+Tab, Explorer restart, DPI.");
        }
    }

    private sealed class TestWindow : Window
    {
        public TestWindow()
        {
            Title = "P0-03 spike";
            Width = 320;
            Height = 240;
            Background = new SolidColorBrush(Color.FromArgb(115, 0x3A, 0x6E, 0xA5));
            Content = new TextBlock
            {
                Text = "P0-03 test window — check wallpaper-above, Win+D, focus, Alt+Tab",
                TextWrapping = Avalonia.Media.TextWrapping.Wrap,
                Margin = new Avalonia.Thickness(12),
            };
        }
    }
}
