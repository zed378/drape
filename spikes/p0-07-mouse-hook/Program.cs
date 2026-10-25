using System.Collections.Concurrent;
using System.Diagnostics;
using Windows.Win32;
using Windows.Win32.Foundation;
using Windows.Win32.UI.WindowsAndMessaging;

namespace P007MouseHook;

/// <summary>
/// P0-07 spike harness. Installs WH_MOUSE_LL on a dedicated thread, measures end-to-end callback
/// latency, and classifies coarse events. Press Esc in this console to stop and print stats.
/// No coordinates are stored or logged beyond the live classification.
/// </summary>
internal static class Program
{
    private static readonly ConcurrentQueue<string> Events = new();
    private static readonly List<double> LatenciesUs = new();
    private static readonly object Gate = new();

    private static HOOKPROC? _keptAlive;
    private static Windows.Win32.UnhookWindowsHookExSafeHandle? _hook;

    public static unsafe int Main()
    {
        Console.WriteLine($"[p0-07] os={Environment.OSVersion}");
        Console.WriteLine($"[p0-07] double-click time={PInvoke.GetDoubleClickTime()} ms");

        var thread = new Thread(HookThread) { IsBackground = true, Name = "p0-07-hook" };
        thread.SetApartmentState(ApartmentState.STA);
        thread.Start();
        Thread.Sleep(500);

        if (_hook is null || _hook.IsInvalid)
        {
            Console.WriteLine("[p0-07] FAIL: hook not installed.");
            return 1;
        }

        Console.WriteLine("[p0-07] hook installed. Double-click empty desktop / icons; right-click; right-drag. Press Esc to stop.");
        while (Console.ReadKey(intercept: true).Key != ConsoleKey.Escape)
        {
        }

        _hook.Dispose();
        PrintStats();
        return 0;
    }

    private static unsafe void HookThread()
    {
        _keptAlive = HookCallback;
        _hook = PInvoke.SetWindowsHookEx(WINDOWS_HOOK_ID.WH_MOUSE_LL, _keptAlive, null, 0);
        if (_hook.IsInvalid)
        {
            Console.WriteLine("[p0-07] SetWindowsHookEx failed.");
            return;
        }

        MSG msg;
        while (PInvoke.GetMessage(&msg, HWND.Null, 0, 0) != 0)
        {
        }
    }

    /// <summary>
    /// Minimal callback: timestamp, classify, enqueue, forward. No allocation to speak of, no I/O,
    /// no long lock. NEVER log coordinates here.
    /// </summary>
    private static unsafe LRESULT HookCallback(int code, WPARAM wParam, LPARAM lParam)
    {
        var start = Stopwatch.GetTimestamp();
        try
        {
            if (code >= 0)
            {
                var message = (uint)wParam.Value;
                if (message is 0x0201 or 0x0202 or 0x0204 or 0x0205 or 0x0203 or 0x0206) // L/R down/up/dblclk
                {
                    var data = (MSLLHOOKSTRUCT*)(void*)lParam.Value;
                    var kind = message switch
                    {
                        0x0201 => "L-down",
                        0x0202 => "L-up",
                        0x0203 => "L-dblclk",
                        0x0204 => "R-down",
                        0x0205 => "R-up",
                        _ => "R-dblclk",
                    };
                    var onDesktop = IsOverDesktopIcons(data->pt.X, data->pt.Y) ? "desktop" : "other";
                    Events.Enqueue($"{kind} {onDesktop}");
                }
            }

            return PInvoke.CallNextHookEx(null, code, wParam, lParam);
        }
        finally
        {
            var elapsedUs = (Stopwatch.GetTimestamp() - start) * 1_000_000.0 / Stopwatch.Frequency;
            lock (Gate)
            {
                if (LatenciesUs.Count < 10_000)
                {
                    LatenciesUs.Add(elapsedUs);
                }
            }
        }
    }

    private static unsafe bool IsOverDesktopIcons(int x, int y)
    {
        var point = new System.Drawing.Point(x, y);
        var hwnd = PInvoke.WindowFromPoint(point);
        if (hwnd.IsNull)
        {
            return false;
        }

        Span<char> buffer = stackalloc char[256];
        fixed (char* p = buffer)
        {
            var len = PInvoke.GetClassName(hwnd, p, 256);
            var name = len == 0 ? string.Empty : new string(p, 0, len);
            return name is "SysListView32" or "SHELLDLL_DefView";
        }
    }

    private static void PrintStats()
    {
        Console.WriteLine($"[p0-07] events seen: {Events.Count}");
        foreach (var group in Events.GroupBy(e => e).OrderBy(g => g.Key))
        {
            Console.WriteLine($"[p0-07]   {group.Key}: {group.Count()}");
        }

        lock (Gate)
        {
            if (LatenciesUs.Count == 0)
            {
                Console.WriteLine("[p0-07] no latency samples.");
                return;
            }

            var sorted = LatenciesUs.OrderBy(v => v).ToList();
            var avg = sorted.Average();
            var max = sorted[^1];
            var p99 = sorted[(int)(sorted.Count * 0.99)];
            Console.WriteLine($"[p0-07] latency us: avg={avg:F1} p99={p99:F1} max={max:F1} n={sorted.Count}");
            Console.WriteLine(avg < 1000
                ? "[p0-07] PASS: average under 1 ms."
                : "[p0-07] FAIL: average over 1 ms.");
        }
    }
}
