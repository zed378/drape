// CsWin32 generates its bindings under the Windows.Win32.* namespaces.
// Pull the ones this assembly needs into scope so callers can use short names.
// P/Invoke entry points live in the Windows.Win32.PInvoke root class.
global using Windows.Win32;
global using Windows.Win32.Foundation;
global using Windows.Win32.Graphics.Gdi;
global using Windows.Win32.System;
global using Windows.Win32.System.Com;
global using Windows.Win32.UI.HiDpi;
global using Windows.Win32.UI.Shell;
global using Windows.Win32.UI.Shell.Common;

// Managed COM / interop helpers (Marshal, GCHandle, CoTaskMemFree caller, COMException).
global using System.Runtime.InteropServices;
// Win32 error reporting.
global using System.ComponentModel;
