using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Media;
using Avalonia.Threading;

namespace P006BlurBench;

/// <summary>
/// P0-06 spike harness. Opens 20 test windows (5 per variant: transparent, blur-behind,
/// acrylic, solid fallback) and reports process CPU + working set. The owner runs this in the
/// VM, idles 60 s, drags one window, and records Task Manager / Performance Monitor numbers.
/// Usage: <c>p0-06-blur-bench.exe [transparent|blur|acrylic|solid|all]</c>.
/// </summary>
internal static class Program
{
    public static void Main(string[] args)
    {
        var variant = args.FirstOrDefault()?.ToLowerInvariant() ?? "all";
        Console.WriteLine($"[p0-06] variant={variant} os={Environment.OSVersion}");
        Console.WriteLine($"[p0-06] processors={Environment.ProcessorCount} mem={GC.GetGCMemoryInfo().TotalAvailableMemoryBytes / 1024 / 1024} MB");

        BuildAvaloniaApp(variant).StartWithClassicDesktopLifetime(args);
    }

    private static AppBuilder BuildAvaloniaApp(string variant)
        => AppBuilder.Configure<SpikeApp>()
            .UsePlatformDetect()
            .LogToTrace()
            .AfterSetup(_ => SpikeState.Variant = variant);

    private static class SpikeState
    {
        public static string Variant = "all";
    }

    private sealed class SpikeApp : Application
    {
        public override void OnFrameworkInitializationCompleted()
        {
            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                var variants = SpikeState.Variant == "all"
                    ? new[] { "transparent", "blur", "acrylic", "solid" }
                    : new[] { SpikeState.Variant };

                var x = 40;
                foreach (var variant in variants)
                {
                    for (var i = 0; i < 5; i++)
                    {
                        var window = new TestWindow(variant, i);
                        window.Position = new PixelPoint(x, 60 + (i * 40));
                        window.Show();
                        x += 60;
                    }
                }

                Console.WriteLine("[p0-06] 20 windows open. Idle 60s, note CPU/GPU in Task Manager, then drag one window and note again.");
                Console.WriteLine("[p0-06] Also test: high-contrast mode on/off; rounded corners on Win10 vs Win11.");
            }

            base.OnFrameworkInitializationCompleted();
        }
    }

    private sealed class TestWindow : Window
    {
        public TestWindow(string variant, int index)
        {
            Title = $"P0-06 {variant} #{index}";
            Width = 320;
            Height = 240;
            ShowInTaskbar = true;
            Background = variant switch
            {
                "solid" => new SolidColorBrush(Color.FromArgb(255, 0x3A, 0x6E, 0xA5)),
                _ => new SolidColorBrush(Color.FromArgb(115, 0x3A, 0x6E, 0xA5)),
            };
            TransparencyLevelHint = variant switch
            {
                "blur" => new[] { WindowTransparencyLevel.Blur },
                "acrylic" => new[] { WindowTransparencyLevel.AcrylicBlur },
                "transparent" => new[] { WindowTransparencyLevel.Transparent },
                _ => new[] { WindowTransparencyLevel.None },
            };
            Content = new TextBlock
            {
                Text = $"{variant} #{index}",
                Margin = new Avalonia.Thickness(12),
            };
        }
    }
}
