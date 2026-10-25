using System.Diagnostics;
using Windows.Win32;
using Windows.Win32.Foundation;
using Windows.Win32.System.Com;
using Windows.Win32.UI.Shell;

namespace P005ShellIcons;

/// <summary>
/// P0-05 spike harness. Enumerates the user + public Desktop via Known Folder paths, times
/// icon extraction via IShellItemImageFactory, and reports overlays/hydration observations.
/// Run in the VM: <c>p0-05-shell-icons.exe [--count N]</c>.
/// </summary>
internal static class Program
{
    private static readonly Guid FolderIdDesktop = new("B4BFCC3A-DB2C-424C-B029-7FE99A87C641");
    private static readonly Guid FolderIdPublicDesktop = new("C4AA340D-F20F-4863-AFEF-F87EF2E6BA25");

    public static unsafe int Main(string[] args)
    {
        var wantCount = 0;
        for (var i = 0; i < args.Length - 1; i++)
        {
            if (args[i] == "--count" && int.TryParse(args[i + 1], out var n))
            {
                wantCount = n;
            }
        }

        Console.WriteLine($"[p0-05] os={Environment.OSVersion}");
        PInvoke.CoInitializeEx(COINIT.COINIT_APARTMENTTHREADED);
        try
        {
            return Run(wantCount);
        }
        finally
        {
            PInvoke.CoUninitialize();
        }
    }

    private static unsafe int Run(int wantCount)
    {
        var userDesktop = GetKnownFolder(FolderIdDesktop);
        var publicDesktop = GetKnownFolder(FolderIdPublicDesktop);
        Console.WriteLine($"[p0-05] userDesktop={userDesktop}");
        Console.WriteLine($"[p0-05] publicDesktop={publicDesktop}");

        var files = new List<string>();
        foreach (var dir in new[] { userDesktop, publicDesktop }.Where(d => d is not null))
        {
            foreach (var file in Directory.EnumerateFiles(dir!))
            {
                var name = Path.GetFileName(file);
                if (name.Equals("desktop.ini", StringComparison.OrdinalIgnoreCase))
                {
                    continue;
                }

                var attributes = File.GetAttributes(file);
                if ((attributes & (FileAttributes.Hidden | FileAttributes.System)) != 0)
                {
                    Console.WriteLine($"[p0-05] skip hidden/system: {name}");
                    continue;
                }

                files.Add(file);
            }
        }

        Console.WriteLine($"[p0-05] enumerated {files.Count} items");

        // Pad with synthetic copies for the 500-item timing when the desktop is sparse.
        while (files.Count < wantCount && files.Count > 0)
        {
            files.Add(files[files.Count % Math.Max(1, files.Count / 2)]);
        }

        var sw = Stopwatch.StartNew();
        var ok = 0;
        var failed = 0;
        foreach (var file in files)
        {
            if (TryGetIcon(file, 48, out var w, out var h))
            {
                ok++;
            }
            else
            {
                failed++;
                Console.WriteLine($"[p0-05] no icon: {file}");
            }
        }

        sw.Stop();
        Console.WriteLine($"[p0-05] icons: ok={ok} failed={failed} elapsed={sw.Elapsed.TotalMilliseconds:F1} ms for {files.Count} items");
        Console.WriteLine("[p0-05] Manual: check OneDrive Files On-Demand placeholder was NOT hydrated; note overlays (shortcut arrow, cloud status) present or missing.");
        return 0;
    }

    private static unsafe bool TryGetIcon(string path, int size, out int width, out int height)
    {
        width = 0;
        height = 0;
        try
        {
            var hr = PInvoke.SHCreateItemFromParsingName<IShellItem>(path, null, out var item);
            if (hr.Failed || item is null)
            {
                return false;
            }

            try
            {
                if (item is not IShellItemImageFactory factory)
                {
                    return false;
                }

                var sizeStruct = new SIZE { cx = size, cy = size };
                factory.GetImage(sizeStruct, SIIGBF.SIIGBF_RESIZETOFIT, out var bitmap);
                using (bitmap)
                {
                    // Success = the shell produced a bitmap. Dimensions are read from the
                    // requested size; pixel extraction is production work (P1-08), not the spike.
                    width = size;
                    height = size;
                    return !bitmap.IsInvalid;
                }
            }
            finally
            {
                System.Runtime.InteropServices.Marshal.ReleaseComObject(item);
            }
        }
        catch
        {
            return false;
        }
    }

    private static unsafe string? GetKnownFolder(Guid folderId)
    {
        var hr = PInvoke.SHGetKnownFolderPath(folderId, 0, null, out var path);
        if (hr.Failed)
        {
            return null;
        }

        using (path)
        {
            return path.DangerousValue.Value is null ? null : path.DangerousValue.ToString();
        }
    }
}
