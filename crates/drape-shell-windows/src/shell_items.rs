//! Read-only R0-06 native probe. No host or icon-visibility capability is enabled.
#![allow(unsafe_code)]

use drape_core::ports::{DesktopLocations, PortError};
use std::ffi::{OsStr, OsString};
use std::marker::PhantomData;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::rc::Rc;
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, GetDIBits, GetObjectW, HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS, FILE_ATTRIBUTE_RECALL_ON_OPEN,
    FILE_ATTRIBUTE_REPARSE_POINT, GetFileAttributesW,
};
use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize,
};
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_PublicDesktop, IShellItemImageFactory, KF_FLAG_DEFAULT,
    SHCreateItemFromParsingName, SHGetKnownFolderPath, SIIGBF_ICONONLY, SIIGBF_MEMORYONLY,
    SIIGBF_RESIZETOFIT,
};
use windows::core::{PCWSTR, PWSTR};

/// Synchronous known-folder lookup. Call on a worker, not the Slint event loop.
#[derive(Debug, Default)]
pub struct WindowsDesktopLocations;

impl DesktopLocations for WindowsDesktopLocations {
    fn desktop_locations(&self) -> Result<Vec<OsString>, PortError> {
        let _apartment = Apartment::new()?;
        [FOLDERID_Desktop, FOLDERID_PublicDesktop]
            .iter()
            .map(|id| {
                // SAFETY: the GUID is live and no token is borrowed. Success transfers
                // a CoTaskMem allocation, immediately wrapped for all return paths.
                let path = KnownFolderPath(
                    unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }
                        .map_err(|_| PortError::Unavailable)?,
                );
                if path.0.is_null() {
                    return Err(PortError::Unavailable);
                }
                // SAFETY: SHGetKnownFolderPath returns a NUL-terminated wide string;
                // the owning wrapper stays alive while it is copied losslessly.
                let value = OsString::from_wide(unsafe { path.0.as_wide() });
                if value.is_empty() {
                    Err(PortError::Unavailable)
                } else {
                    Ok(value)
                }
            })
            .collect()
    }
}

/// Raw probe result. Rows are bottom-up B,G,R,reserved bytes; alpha is unspecified.
/// It cannot be passed to Core `IconBitmap` until alpha semantics are established.
#[derive(Debug)]
pub struct ShellIconDib {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bottom_up_bgrx: Vec<u8>,
}

/// Obtains only an icon available in memory and copies its DIB bytes before native
/// owners are released. With `SIIGBF_MEMORYONLY`, the shell may return a per-class
/// icon when the item's own icon is not cached; callers cannot distinguish it, so
/// they must not cache this result as the item's own icon.
///
/// Synchronous and STA-affine for the duration of the call; run on a bounded icon
/// worker. No thumbnail, cache, background executor, UI conversion or overlay claim.
/// Cloud/reparse/recall/offline attributes fail closed. This attribute guard is not
/// proof against every provider's hydration behavior; R0-06 needs VM evidence.
pub fn probe_shell_icon(
    path: &OsStr,
    pixel_size: u32,
    byte_limit: usize,
) -> Result<ShellIconDib, PortError> {
    let size = i32::try_from(pixel_size).map_err(|_| PortError::LimitExceeded)?;
    let requested_stride = pixel_size.checked_mul(4).ok_or(PortError::LimitExceeded)?;
    drape_core::domain::IconBitmap::required_bytes(
        pixel_size,
        pixel_size,
        requested_stride,
        byte_limit,
    )
    .map_err(|_| PortError::LimitExceeded)?;
    let wide = terminated_path(path)?;
    // SAFETY: wide remains NUL-terminated and live across the attribute query.
    let attributes = unsafe { GetFileAttributesW(PCWSTR(wide.as_ptr())) };
    let blocked = FILE_ATTRIBUTE_OFFLINE.0
        | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS.0
        | FILE_ATTRIBUTE_RECALL_ON_OPEN.0
        | FILE_ATTRIBUTE_REPARSE_POINT.0;
    if attributes == u32::MAX || attributes & blocked != 0 {
        return Err(PortError::Unavailable);
    }
    let _apartment = Apartment::new()?;
    // SAFETY: path is live and terminated; COM is initialized on this thread.
    // The windows interface owns its reference and is dropped before Apartment.
    let factory: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None) }
            .map_err(|_| PortError::Unavailable)?;
    // SAFETY: the factory is live on its COM apartment, positive bounded size.
    // GetImage transfers the bitmap to our RAII wrapper.
    let bitmap = OwnedBitmap(
        unsafe {
            factory.GetImage(
                SIZE { cx: size, cy: size },
                SIIGBF_RESIZETOFIT | SIIGBF_ICONONLY | SIIGBF_MEMORYONLY,
            )
        }
        .map_err(|_| PortError::Unavailable)?,
    );
    if bitmap.0.is_invalid() {
        return Err(PortError::Unavailable);
    }
    copy_dib(&bitmap, pixel_size, byte_limit)
}

fn terminated_path(path: &OsStr) -> Result<Vec<u16>, PortError> {
    // Bound the Win32 input before collection, including the terminator.
    let mut wide: Vec<u16> = path.encode_wide().take(32_768).collect();
    if wide.len() >= 32_768 {
        return Err(PortError::LimitExceeded);
    }
    if wide.is_empty() || wide.contains(&0) {
        return Err(PortError::Unavailable);
    }
    wide.push(0);
    Ok(wide)
}

fn copy_dib(
    bitmap: &OwnedBitmap,
    pixel_size: u32,
    byte_limit: usize,
) -> Result<ShellIconDib, PortError> {
    let mut metadata = BITMAP::default();
    // SAFETY: the bitmap is owned/live and metadata points to a writable BITMAP.
    let copied = unsafe {
        GetObjectW(
            HGDIOBJ(bitmap.0.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some((&mut metadata as *mut BITMAP).cast()),
        )
    };
    if copied != std::mem::size_of::<BITMAP>() as i32 {
        return Err(PortError::Unavailable);
    }
    let width = u32::try_from(metadata.bmWidth).map_err(|_| PortError::Unavailable)?;
    let height = u32::try_from(metadata.bmHeight).map_err(|_| PortError::Unavailable)?;
    if width > pixel_size || height > pixel_size {
        return Err(PortError::LimitExceeded);
    }
    let stride = width.checked_mul(4).ok_or(PortError::LimitExceeded)?;
    let len = drape_core::domain::IconBitmap::required_bytes(width, height, stride, byte_limit)
        .map_err(|_| PortError::LimitExceeded)?;
    let mut bytes = vec![0; len];
    // SAFETY: creates an unselected memory DC owned by OwnedDc.
    let dc = OwnedDc(unsafe { CreateCompatibleDC(None) });
    if dc.0.is_invalid() {
        return Err(PortError::Unavailable);
    }
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: metadata.bmWidth,
            biHeight: metadata.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    // SAFETY: the bitmap has never been selected into this or any DC here; the
    // tightly packed 32-bit buffer was validated for all requested scan lines.
    let lines = unsafe {
        GetDIBits(
            dc.0,
            bitmap.0,
            0,
            height,
            Some(bytes.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        )
    };
    if lines != metadata.bmHeight {
        return Err(PortError::Unavailable);
    }
    Ok(ShellIconDib {
        width,
        height,
        stride,
        bottom_up_bgrx: bytes,
    })
}

struct Apartment(PhantomData<Rc<()>>);
impl Apartment {
    fn new() -> Result<Self, PortError> {
        // SAFETY: matching CoUninitialize occurs on this thread. A changed COM
        // mode fails rather than using an apartment with an unverified contract.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(|_| PortError::Unavailable)?;
        Ok(Self(PhantomData))
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: one successful initialization is paired on the same !Send owner.
        unsafe { CoUninitialize() };
    }
}
struct KnownFolderPath(PWSTR);
impl Drop for KnownFolderPath {
    fn drop(&mut self) {
        // SAFETY: SHGetKnownFolderPath transferred this CoTaskMem allocation.
        unsafe { CoTaskMemFree(Some(self.0.as_ptr().cast())) };
    }
}
struct OwnedBitmap(HBITMAP);
impl Drop for OwnedBitmap {
    fn drop(&mut self) {
        // SAFETY: GetImage transferred this unselected bitmap, released once.
        let _ = unsafe { DeleteObject(HGDIOBJ(self.0.0)) };
    }
}
struct OwnedDc(HDC);
impl Drop for OwnedDc {
    fn drop(&mut self) {
        // SAFETY: only CreateCompatibleDC creates this independently owned DC.
        let _ = unsafe { DeleteDC(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_wide_paths_reject_empty_and_embedded_nul_before_native_access() {
        assert_eq!(terminated_path(OsStr::new("")), Err(PortError::Unavailable));
        assert_eq!(
            terminated_path(OsStr::new("a\0b")),
            Err(PortError::Unavailable)
        );
        assert_eq!(
            terminated_path(OsStr::new("\\\\server\\share\\未")),
            Ok("\\\\server\\share\\未".encode_utf16().chain([0]).collect())
        );
    }
}
