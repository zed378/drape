using Windows.Win32;
using Windows.Win32.Foundation;
using Windows.Win32.UI.WindowsAndMessaging;

namespace P003DesktopHost;

/// <summary>Which host strategy the spike is exercising.</summary>
internal enum HostStrategy
{
    /// <summary>SetParent to the discovered desktop host.</summary>
    A,

    /// <summary>Top-level at HWND_BOTTOM with TOOLWINDOW style.</summary>
    B,

    /// <summary>Try A, verify, fall back to B.</summary>
    C,
}

/// <summary>Result of one attach attempt, printed for the results table.</summary>
internal readonly record struct AttachResult(HostStrategy Strategy, bool Attached, string Detail);

/// <summary>Strategies A/B/C from PLATFORM §2.</summary>
internal static class Strategies
{
    /// <summary>Strategy A: reparent to the desktop host.</summary>
    public static unsafe AttachResult TryA(HWND child)
    {
        var host = DesktopDiscovery.FindDesktopHost();
        if (host is null)
        {
            return new AttachResult(HostStrategy.A, false, "no desktop host found");
        }

        var previous = PInvoke.SetParent(child, host.Value);
        if (previous.IsNull)
        {
            return new AttachResult(HostStrategy.A, false, $"SetParent failed: {GetLastError()}");
        }

        // Verify: the parent is now the host and the window still exists.
        var parent = PInvoke.GetParent(child);
        var ok = parent == host.Value && !PInvoke.IsWindow(child).Equals(false);
        return new AttachResult(
            HostStrategy.A, ok,
            ok ? $"parent={parent}" : $"parent mismatch: expected {host.Value}, got {parent}");
    }

    /// <summary>Strategy B: bottom of the z-order, tool window, no activate.</summary>
    public static unsafe AttachResult TryB(HWND child)
    {
        var ok = PInvoke.SetWindowPos(
            child,
            new HWND(1), // HWND_BOTTOM
            0, 0, 0, 0,
            SET_WINDOW_POS_FLAGS.SWP_NOMOVE | SET_WINDOW_POS_FLAGS.SWP_NOSIZE
            | SET_WINDOW_POS_FLAGS.SWP_NOACTIVATE);
        return new AttachResult(
            HostStrategy.B, (bool)ok,
            ok ? "SetWindowPos(HWND_BOTTOM, NOACTIVATE) ok" : $"SetWindowPos failed: {GetLastError()}");
    }

    /// <summary>Strategy C: try A, verify, fall back to B.</summary>
    public static AttachResult TryC(HWND child)
    {
        var a = TryA(child);
        Console.WriteLine($"[C] A: attached={a.Attached} ({a.Detail})");
        if (a.Attached)
        {
            return a with { Strategy = HostStrategy.C };
        }

        var b = TryB(child);
        return new AttachResult(
            HostStrategy.C, b.Attached,
            $"A failed ({a.Detail}); B: {b.Detail}");
    }

    private static int GetLastError() => System.Runtime.InteropServices.Marshal.GetLastWin32Error();
}
