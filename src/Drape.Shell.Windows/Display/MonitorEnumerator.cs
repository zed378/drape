using Drape.Core;

namespace Drape.Shell.Windows.Display;

/// <summary>
/// Reads the connected monitors and their effective DPI through <c>EnumDisplayMonitors</c>,
/// <c>GetMonitorInfo</c> and <c>GetDpiForMonitor</c>
/// (see <c>docs/PLATFORM/00-WINDOWS-INTEGRATION.md</c> §8).
/// </summary>
/// <remarks>
/// Values are returned in physical pixels. Conversion to Avalonia logical units happens at the
/// <c>IDisplayMonitor</c> boundary, never here (CS-21).
/// </remarks>
internal static class MonitorEnumerator
{
    private const double DefaultDpi = 96d;
    private const string UnknownDisplayId = "display-unknown";

    /// <summary>Returns every monitor currently attached, in the order Windows reports them.</summary>
    public static IReadOnlyList<MonitorInfo> GetMonitors()
    {
        var monitors = new List<MonitorInfo>(2);
        unsafe
        {
            // The null clip rectangle means "every monitor"; the pointer overload is chosen
            // explicitly because the nullable-struct overload makes the call ambiguous.
            PInvoke.EnumDisplayMonitors(default, (RECT*)null, Callback, default);
        }

        return monitors;

        unsafe BOOL Callback(HMONITOR hMonitor, HDC hdc, RECT* clip, LPARAM data)
        {
            if (TryReadMonitor(hMonitor) is { } monitor)
            {
                monitors.Add(monitor);
            }

            // Returning true keeps the enumeration going; a monitor that cannot be read is
            // skipped rather than aborting the whole list.
            return true;
        }
    }

    /// <summary>Returns the primary monitor.</summary>
    /// <exception cref="InvalidOperationException">No monitor could be read.</exception>
    public static MonitorInfo GetPrimaryMonitor()
    {
        var monitors = GetMonitors();
        foreach (var monitor in monitors)
        {
            if (monitor.IsPrimary)
            {
                return monitor;
            }
        }

        if (monitors.Count > 0)
        {
            return monitors[0];
        }

        throw new InvalidOperationException("No display monitor could be enumerated for this session.");
    }

    private static unsafe MonitorInfo? TryReadMonitor(HMONITOR hMonitor)
    {
        var info = default(MONITORINFOEXW);
        info.monitorInfo.cbSize = (uint)sizeof(MONITORINFOEXW);

        // GetMonitorInfo dispatches on cbSize, so passing the embedded MONITORINFO with the
        // MONITORINFOEXW size fills in szDevice as well.
        if (!PInvoke.GetMonitorInfo(hMonitor, ref info.monitorInfo))
        {
            return null;
        }

        var bounds = info.monitorInfo.rcMonitor;
        var workArea = info.monitorInfo.rcWork;
        var deviceName = info.szDevice.ToString();

        var dpiX = DefaultDpi;
        var dpiY = DefaultDpi;
        if (PInvoke.GetDpiForMonitor(hMonitor, MONITOR_DPI_TYPE.MDT_EFFECTIVE_DPI, out var rawX, out var rawY).Succeeded)
        {
            dpiX = rawX > 0 ? rawX : DefaultDpi;
            dpiY = rawY > 0 ? rawY : DefaultDpi;
        }

        return new MonitorInfo(
            Id: deviceName.Length > 0 ? deviceName : UnknownDisplayId,
            DeviceName: deviceName,
            Bounds: ToRectangle(bounds),
            WorkingArea: ToRectangle(workArea),
            DpiX: dpiX,
            DpiY: dpiY,
            ScaleFactor: dpiX / DefaultDpi,
            IsPrimary: (info.monitorInfo.dwFlags & PInvoke.MONITORINFOF_PRIMARY) != 0);
    }

    private static Rectangle ToRectangle(RECT rect) => new(rect.left, rect.top, rect.Width, rect.Height);
}
