using Windows.Win32;
using Windows.Win32.Foundation;
using Windows.Win32.UI.WindowsAndMessaging;

namespace P004DesktopIcons;

/// <summary>
/// P0-04 spike harness. Commands: <c>read</c>, <c>hide</c>, <c>show</c>, <c>--restore</c>.
/// Run in the VM only, with a checkpoint taken first. Never run on the host.
/// </summary>
internal static class Program
{
    /// <summary>Toggle command for "Show desktop icons" sent to SHELLDLL_DefView.</summary>
    private const uint ToggleIcons = 0x7402;

    /// <summary>WM_COMMAND message id.</summary>
    private const uint WmCommand = 0x0111;

    public static int Main(string[] args)
    {
        var command = args.FirstOrDefault()?.ToLowerInvariant() ?? "read";
        Console.WriteLine($"[p0-04] command={command} os={Environment.OSVersion}");

        return command switch
        {
            "read" => Read(),
            "hide" => SetHidden(true),
            "show" => SetHidden(false),
            "--restore" => SetHidden(false),
            _ => Usage(),
        };
    }

    private static int Usage()
    {
        Console.WriteLine("Usage: p0-04-desktop-icons [read|hide|show|--restore]");
        return 2;
    }

    /// <summary>Reads the current icon state without changing it.</summary>
    private static int Read()
    {
        var state = ReadState();
        Console.WriteLine($"[p0-04] icons hidden={state}");
        return 0;
    }

    /// <summary>Hides or shows icons, reading state first so a redundant call never flips.</summary>
    private static int SetHidden(bool hidden)
    {
        var current = ReadState();
        if (current is null)
        {
            Console.WriteLine("[p0-04] FAIL: could not read icon state; refusing to toggle.");
            return 1;
        }

        if (current == hidden)
        {
            Console.WriteLine($"[p0-04] already {(hidden ? "hidden" : "shown")}; no toggle sent.");
            return 0;
        }

        var defView = FindDefView();
        if (defView is null)
        {
            Console.WriteLine("[p0-04] FAIL: SHELLDLL_DefView not found.");
            return 1;
        }

        unsafe
        {
            PInvoke.SendMessage(defView.Value, WmCommand, new WPARAM(ToggleIcons), 0);
        }

        Thread.Sleep(500);
        var after = ReadState();
        Console.WriteLine($"[p0-04] after: hidden={after} (wanted {hidden})");
        return after == hidden ? 0 : 1;
    }

    /// <summary>Reads whether icons are hidden via SysListView32 visibility.</summary>
    /// <returns>True when hidden, false when shown, null when unreadable.</returns>
    private static bool? ReadState()
    {
        var listView = FindListView();
        if (listView is null)
        {
            return null;
        }

        return !(bool)PInvoke.IsWindowVisible(listView.Value);
    }

    private static HWND? FindDefView()
    {
        HWND? found = null;
        unsafe
        {
            PInvoke.EnumWindows((hwnd, _) =>
            {
                if (GetClassName(hwnd) is "Progman" or "WorkerW"
                    && FindChild(hwnd, "SHELLDLL_DefView") is { } defView)
                {
                    found = defView;
                    return false;
                }

                return true;
            }, 0);
        }

        return found;
    }

    private static HWND? FindListView()
    {
        var defView = FindDefView();
        return defView is null ? null : FindChild(defView.Value, "SysListView32");
    }

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
