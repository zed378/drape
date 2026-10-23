using Drape.Core;

namespace Drape.Shell.Windows.Dialogs;

/// <summary>
/// Folder and file pickers built on <c>IFileOpenDialog</c> through CsWin32.
/// </summary>
/// <remarks>
/// <para>
/// The common item dialog must run on a single-threaded apartment (STA) thread with a message
/// pump. The caller (usually Avalonia's UI thread) is not guaranteed to satisfy that, so every
/// dialog is opened on a short-lived STA thread and the result is marshalled back.
/// </para>
/// <para>
/// Nothing here touches Core state: the methods return paths and the caller decides what to do
/// with them (CS-23, CS-24).
/// </para>
/// </remarks>
internal static class ShellFileDialog
{
    /// <summary>HRESULT_FROM_WIN32(ERROR_CANCELLED): the user closed the dialog.</summary>
    private const int CancelledHResult = unchecked((int)0x800704C7);

    private static readonly Guid _fileOpenDialogClassId = new("DC1C5A9C-E88A-4DDE-A5A1-60F82A20AEF7");

    /// <summary>Opens a single-folder picker and returns the chosen path, or <c>null</c> when cancelled.</summary>
    public static Task<string?> PickFolderAsync(string? initialPath, CancellationToken ct)
        => RunOnStaThreadAsync(
            hwnd => PickFolder(initialPath, hwnd),
            ct);

    /// <summary>Opens a multi-select file picker and returns the chosen paths, or <c>null</c> when cancelled.</summary>
    public static Task<IReadOnlyList<string>?> PickFilesAsync(string? initialPath, string? filter, CancellationToken ct)
        => RunOnStaThreadAsync<IReadOnlyList<string>?>(
            hwnd => PickFiles(initialPath, filter, hwnd),
            ct);

    private static string? PickFolder(string? initialPath, HWND owner)
    {
        var dialog = CreateDialog(owner);
        try
        {
            var options = GetOptions(dialog);
            options |= FILEOPENDIALOGOPTIONS.FOS_PICKFOLDERS
                | FILEOPENDIALOGOPTIONS.FOS_FORCEFILESYSTEM
                | FILEOPENDIALOGOPTIONS.FOS_PATHMUSTEXIST;
            dialog.SetOptions(options);

            SetInitialFolder(dialog, initialPath);

            if (!TryShow(dialog))
            {
                return null;
            }

            dialog.GetResult(out var item);
            try
            {
                return GetFileSystemPath(item);
            }
            finally
            {
                ReleaseComObject(item);
            }
        }
        finally
        {
            ReleaseComObject(dialog);
        }
    }

    private static IReadOnlyList<string>? PickFiles(string? initialPath, string? filter, HWND owner)
    {
        var dialog = CreateDialog(owner);
        try
        {
            var options = GetOptions(dialog);
            options |= FILEOPENDIALOGOPTIONS.FOS_ALLOWMULTISELECT
                | FILEOPENDIALOGOPTIONS.FOS_FORCEFILESYSTEM
                | FILEOPENDIALOGOPTIONS.FOS_FILEMUSTEXIST
                | FILEOPENDIALOGOPTIONS.FOS_PATHMUSTEXIST;
            dialog.SetOptions(options);

            SetInitialFolder(dialog, initialPath);

            if (filter is not null)
            {
                ApplyFilter(dialog, filter);
            }

            if (!TryShow(dialog))
            {
                return null;
            }

            dialog.GetResults(out var results);
            try
            {
                results.GetCount(out var count);
                var paths = new List<string>((int)count);
                for (var i = 0u; i < count; i++)
                {
                    results.GetItemAt(i, out var item);
                    try
                    {
                        if (GetFileSystemPath(item) is { } path)
                        {
                            paths.Add(path);
                        }
                    }
                    finally
                    {
                        ReleaseComObject(item);
                    }
                }

                return paths;
            }
            finally
            {
                ReleaseComObject(results);
            }
        }
        finally
        {
            ReleaseComObject(dialog);
        }
    }

    private static IFileOpenDialog CreateDialog(HWND owner)
    {
        var hr = PInvoke.CoCreateInstance<IFileOpenDialog>(
            _fileOpenDialogClassId,
            null!,
            CLSCTX.CLSCTX_INPROC_SERVER,
            out var dialog);

        hr.ThrowOnFailure();
        return dialog;
    }

    private static unsafe FILEOPENDIALOGOPTIONS GetOptions(IFileOpenDialog dialog)
    {
        FILEOPENDIALOGOPTIONS options;
        dialog.GetOptions(&options);
        return options;
    }

    private static unsafe void ApplyFilter(IFileOpenDialog dialog, string filter)
    {
        // The caller passes a shell filter spec such as "*.txt" or "*.png;*.jpg".
        // The strings must stay pinned for the duration of the SetFileTypes call.
        fixed (char* name = "Supported files")
        fixed (char* spec = filter)
        {
            var filterSpec = new COMDLG_FILTERSPEC
            {
                pszName = name,
                pszSpec = spec,
            };

            dialog.SetFileTypes(1, &filterSpec);
        }
    }

    private static void SetInitialFolder(IFileOpenDialog dialog, string? initialPath)
    {
        if (string.IsNullOrWhiteSpace(initialPath))
        {
            return;
        }

        var hr = PInvoke.SHCreateItemFromParsingName<IShellItem>(initialPath, null!, out var item);

        if (hr.Failed)
        {
            // A missing or inaccessible folder is not a reason to refuse to show the dialog.
            return;
        }

        try
        {
            dialog.SetFolder(item);
        }
        finally
        {
            ReleaseComObject(item);
        }
    }

    /// <summary>Shows the dialog; returns <c>false</c> when the user cancelled.</summary>
    private static bool TryShow(IFileOpenDialog dialog)
    {
        try
        {
            dialog.Show(default);
            return true;
        }
        catch (COMException ex) when (ex.HResult == CancelledHResult)
        {
            return false;
        }
    }

    private static unsafe string? GetFileSystemPath(IShellItem item)
    {
        if (item is null)
        {
            return null;
        }

        item.GetDisplayName(SIGDN.SIGDN_FILESYSPATH, out var name);
        if (name.Value is null)
        {
            return null;
        }

        try
        {
            return name.ToString();
        }
        finally
        {
            // The string is allocated with CoTaskMemAlloc by the shell; the caller owns it.
            PInvoke.CoTaskMemFree(name.Value);
        }
    }

    private static void ReleaseComObject(object? instance)
    {
        if (instance is not null && Marshal.IsComObject(instance))
        {
            _ = Marshal.ReleaseComObject(instance);
        }
    }

    private static Task<T> RunOnStaThreadAsync<T>(Func<HWND, T> work, CancellationToken ct)
    {
        var completion = new TaskCompletionSource<T>(TaskCreationOptions.RunContinuationsAsynchronously);

        var thread = new Thread(() =>
        {
            if (ct.IsCancellationRequested)
            {
                completion.TrySetCanceled(ct);
                return;
            }

            // RPC_E_CHANGED_MODE is acceptable: it means the thread was already initialised
            // with a different apartment, which cannot happen on a fresh thread but is not fatal.
            var hr = PInvoke.CoInitializeEx(COINIT.COINIT_APARTMENTTHREADED);
            var mustUninitialize = hr.Succeeded;
            try
            {
                completion.TrySetResult(work(default));
            }
            catch (Exception ex)
            {
                completion.TrySetException(ex);
            }
            finally
            {
                if (mustUninitialize)
                {
                    PInvoke.CoUninitialize();
                }
            }
        })
        {
            IsBackground = true,
            Name = "Drape.Shell.Windows.FileDialog",
        };

        thread.SetApartmentState(ApartmentState.STA);
        thread.Start();

        return completion.Task;
    }
}
