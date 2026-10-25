using Windows.Win32;
using Windows.Win32.Foundation;

namespace P003DesktopHost;

/// <summary>
/// Discovers the desktop host window dynamically — never hard-coded (PLATFORM §1).
/// </summary>
internal static class DesktopDiscovery
{
    /// <summary>Message sent to Progman to spawn a WorkerW behind the icons.</summary>
    private const uint SpawnWorkerW = 0x052C;

    /// <summary>Finds the window to parent to: Progman/WorkerW hosting SHELLDLL_DefView.</summary>
    /// <returns>The host handle, or null when not found.</returns>
    public static unsafe HWND? FindDesktopHost()
    {
        // 1. Progman with a direct SHELLDLL_DefView child.
        var progman = PInvoke.FindWindow("Progman", null);
        if (!progman.IsNull)
        {
            if (FindChild(progman, "SHELLDLL_DefView") is { } direct)
            {
                Console.WriteLine($"[discover] SHELLDLL_DefView under Progman (host={progman}).");
                return progman;
            }

            // 2. No direct child: ask Progman to spawn a WorkerW, then look for it.
            PInvoke.SendMessage(progman, SpawnWorkerW, 0, 0);
            Thread.Sleep(500);
        }

        // 3. Enumerate top-level WorkerW windows for one with a SHELLDLL_DefView child.
        HWND? found = null;
        PInvoke.EnumWindows((hwnd, _) =>
        {
            if (GetClassName(hwnd) == "WorkerW" && FindChild(hwnd, "SHELLDLL_DefView") is not null)
            {
                found = hwnd;
                return false;
            }

            return true;
        }, 0);

        if (found is { } host)
        {
            Console.WriteLine($"[discover] SHELLDLL_DefView under WorkerW (host={host}).");
            return host;
        }

        Console.WriteLine("[discover] No desktop host found.");
        return null;
    }

    /// <summary>Finds a direct child window by class name.</summary>
    private static unsafe HWND? FindChild(HWND parent, string className)
    {
        HWND? found = null;
        PInvoke.EnumChildWindows(parent, (hwnd, _) =>
        {
            if (GetClassName(hwnd) == className)
            {
                found = hwnd;
                return false;
            }

            return true;
        }, 0);

        return found;
    }

    private static unsafe string GetClassName(HWND hwnd)
    {
        Span<char> buffer = stackalloc char[256];
        fixed (char* p = buffer)
        {
            var len = PInvoke.GetClassName(hwnd, p, 256);
            return len == 0 ? string.Empty : new string(p, 0, len);
        }
    }
}
