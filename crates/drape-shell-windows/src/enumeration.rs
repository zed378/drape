//! R0-06 passive filesystem metadata probe; not a complete DesktopItemSource.
#![allow(unsafe_code)]

use crate::shell_items::WindowsDesktopLocations;
use drape_core::domain::ItemKind;
use drape_core::ports::{
    Cancellation, DesktopEntry, DesktopItemSource, DesktopLocations, DesktopSnapshot, OperationId,
    PortError,
};
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_OFFLINE,
    FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS, FILE_ATTRIBUTE_RECALL_ON_OPEN,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_SYSTEM, FindClose, FindFirstFileW, FindNextFileW,
    GetFileAttributesW, WIN32_FIND_DATAW,
};
use windows::core::{HRESULT, PCWSTR};

/// Work budgets selected by the caller, not production defaults. Scans include
/// filtered entries; UTF-16 units count retained literal paths without terminators.
#[derive(Debug, Clone, Copy)]
pub struct EnumerationLimits {
    pub entries: usize,
    pub scanned_entries: usize,
    pub path_utf16_units: usize,
}

/// The caller supplies Explorer's verified visibility settings. This probe does
/// not read the registry or assume that hidden/system items should be displayed.
#[derive(Debug, Clone, Copy, Default)]
pub struct EnumerationVisibility {
    pub show_hidden: bool,
    pub show_system: bool,
}

/// Owned literal metadata only. No file identity, case-comparison key, resolved
/// shortcut target, icon, content or shell extension is requested by enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellMetadataEntry {
    pub path: OsString,
    pub kind: ItemKind,
    pub attributes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellMetadataSnapshot {
    pub operation: OperationId,
    pub entries: Vec<ShellMetadataEntry>,
}

/// Attach comparison keys obtained by a separately reviewed Windows identity
/// provider. Path keys are deliberately supplied by the caller: deriving one
/// by lower-casing or canonicalising the literal path would be incorrect for
/// case-sensitive directories, UNC roots and unavailable cloud items.
///
/// This conversion is allocation-bounded by the already completed metadata
/// scan. It performs no I/O and never opens, moves or resolves an item.
pub fn attach_comparison_keys(
    snapshot: ShellMetadataSnapshot,
    keys: Vec<drape_core::domain::PathKey>,
) -> Result<DesktopSnapshot, PortError> {
    if snapshot.entries.len() != keys.len() {
        return Err(PortError::Unavailable);
    }
    let entries = snapshot
        .entries
        .into_iter()
        .zip(keys)
        .map(|(entry, key)| DesktopEntry {
            path: entry.path,
            key,
            kind: entry.kind,
            file_id: None,
        })
        .collect();
    Ok(DesktopSnapshot {
        operation: snapshot.operation,
        entries,
    })
}

/// Explicit boundary for the unfinished native identity provider. Returning
/// `Unsupported` prevents callers from accidentally treating a literal path
/// as a Windows comparison key.
#[derive(Debug, Default)]
pub struct WindowsDesktopItemSource;

impl DesktopItemSource for WindowsDesktopItemSource {
    fn snapshot(
        &self,
        _operation: OperationId,
        _entry_limit: usize,
        _cancellation: &Cancellation,
    ) -> Result<DesktopSnapshot, PortError> {
        Err(PortError::Unsupported)
    }
}

/// Enumerates immediate filesystem children of both Known Folder desktop roots.
/// Call on a bounded worker, never the Slint event loop. Cancellation is observed
/// before/between native calls, not a guarantee that a blocked OS call is aborted.
/// An error discards the partial snapshot; limits never silently truncate it.
///
/// FindFirst/Next request directory metadata only; no child is opened or followed.
/// Reparse/offline/recall roots fail closed. Child placeholders remain references
/// and must not be passed to an unverified content/icon loader. Provider-specific
/// no-hydration behavior still requires the R0-06 native matrix.
///
/// This is deliberately not DesktopItemSource: Windows comparison keys (including
/// per-directory case sensitivity and extended/UNC paths) are not established.
pub fn probe_desktop_metadata(
    operation: OperationId,
    limits: EnumerationLimits,
    visibility: EnumerationVisibility,
    cancellation: &Cancellation,
) -> Result<ShellMetadataSnapshot, PortError> {
    cancellation.check()?;
    let roots = WindowsDesktopLocations.desktop_locations()?;
    cancellation.check()?;
    let mut accumulator = MetadataAccumulator::new(limits, visibility);
    for root in roots {
        cancellation.check()?;
        let wide_root = terminated_path(&root)?;
        // SAFETY: the root is a live, bounded, terminated UTF-16 string.
        let attributes = unsafe { GetFileAttributesW(PCWSTR(wide_root.as_ptr())) };
        let blocked = FILE_ATTRIBUTE_REPARSE_POINT.0
            | FILE_ATTRIBUTE_OFFLINE.0
            | FILE_ATTRIBUTE_RECALL_ON_OPEN.0
            | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS.0;
        if attributes == u32::MAX
            || attributes & FILE_ATTRIBUTE_DIRECTORY.0 == 0
            || attributes & blocked != 0
        {
            return Err(PortError::Unavailable);
        }
        let pattern = PathBuf::from(&root).join("*");
        let wide_pattern = terminated_path(pattern.as_os_str())?;
        let mut data = WIN32_FIND_DATAW::default();
        // SAFETY: the pattern stays live and terminated; data is writable. The
        // successful search handle is immediately owned by its FindClose guard.
        let search = match unsafe { FindFirstFileW(PCWSTR(wide_pattern.as_ptr()), &mut data) } {
            Ok(handle) => SearchHandle(handle),
            Err(error) if error.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) => {
                continue;
            }
            Err(_) => return Err(PortError::Unavailable),
        };
        loop {
            cancellation.check()?;
            accumulator.observe(&root, &data.cFileName, data.dwFileAttributes)?;
            cancellation.check()?;
            // SAFETY: search is live and exclusively owned; data is writable.
            match unsafe { FindNextFileW(search.0, &mut data) } {
                Ok(()) => {}
                Err(error) if error.code() == HRESULT::from_win32(ERROR_NO_MORE_FILES.0) => break,
                Err(_) => return Err(PortError::Unavailable),
            }
        }
    }
    cancellation.check()?;
    Ok(ShellMetadataSnapshot {
        operation,
        entries: accumulator.entries,
    })
}

struct SearchHandle(HANDLE);

impl Drop for SearchHandle {
    fn drop(&mut self) {
        // SAFETY: FindFirstFileW transferred one search handle, released once.
        // FindClose, not CloseHandle, is the matching release operation.
        let _ = unsafe { FindClose(self.0) };
    }
}

fn terminated_path(path: &OsStr) -> Result<Vec<u16>, PortError> {
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

struct MetadataAccumulator {
    limits: EnumerationLimits,
    visibility: EnumerationVisibility,
    scanned: usize,
    path_units: usize,
    entries: Vec<ShellMetadataEntry>,
}

impl MetadataAccumulator {
    fn new(limits: EnumerationLimits, visibility: EnumerationVisibility) -> Self {
        Self {
            limits,
            visibility,
            scanned: 0,
            path_units: 0,
            entries: Vec::new(),
        }
    }

    fn observe(&mut self, root: &OsStr, name: &[u16], attributes: u32) -> Result<(), PortError> {
        // Every observed native entry consumes work, even if malformed or
        // excluded below. Exhaustion fails rather than yielding a partial scan.
        self.scanned = self
            .scanned
            .checked_add(1)
            .ok_or(PortError::LimitExceeded)?;
        if self.scanned > self.limits.scanned_entries {
            return Err(PortError::LimitExceeded);
        }
        let end = name
            .iter()
            .position(|unit| *unit == 0)
            .ok_or(PortError::Unavailable)?;
        let name = &name[..end];
        if name == [b'.' as u16] || name == [b'.' as u16, b'.' as u16] {
            return Ok(());
        }
        // A native name is one immediate component, never a traversal request.
        if name.is_empty()
            || name
                .iter()
                .any(|unit| [b'\\' as u16, b'/' as u16, b':' as u16].contains(unit))
        {
            return Err(PortError::Unavailable);
        }
        if ascii_eq(name, b"desktop.ini")
            || (!self.visibility.show_hidden && attributes & FILE_ATTRIBUTE_HIDDEN.0 != 0)
            || (!self.visibility.show_system && attributes & FILE_ATTRIBUTE_SYSTEM.0 != 0)
        {
            return Ok(());
        }
        if self.entries.len() >= self.limits.entries {
            return Err(PortError::LimitExceeded);
        }
        // Bound the complete literal path before allocating its owned copy.
        let root_units = root.encode_wide().count();
        let separator_units = usize::from(
            !root
                .encode_wide()
                .last()
                .is_some_and(|unit| unit == b'\\' as u16 || unit == b'/' as u16),
        );
        let units = root_units
            .checked_add(separator_units)
            .and_then(|value| value.checked_add(name.len()))
            .ok_or(PortError::LimitExceeded)?;
        let total = self
            .path_units
            .checked_add(units)
            .ok_or(PortError::LimitExceeded)?;
        if units >= 32_768 || total > self.limits.path_utf16_units {
            return Err(PortError::LimitExceeded);
        }
        let path = PathBuf::from(root)
            .join(OsString::from_wide(name))
            .into_os_string();
        let kind = if attributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0 {
            ItemKind::Folder
        } else if name.len() >= 4
            && (ascii_eq(&name[name.len() - 4..], b".lnk")
                || ascii_eq(&name[name.len() - 4..], b".url"))
        {
            ItemKind::Shortcut
        } else {
            ItemKind::File
        };
        self.entries.push(ShellMetadataEntry {
            path,
            kind,
            attributes,
        });
        self.path_units = total;
        Ok(())
    }
}

fn ascii_eq(wide: &[u16], ascii: &[u8]) -> bool {
    wide.len() == ascii.len()
        && wide.iter().zip(ascii).all(|(unit, expected)| {
            u8::try_from(*unit).is_ok_and(|value| value.eq_ignore_ascii_case(expected))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accumulator(
        entries: usize,
        scanned_entries: usize,
        path_utf16_units: usize,
    ) -> MetadataAccumulator {
        MetadataAccumulator::new(
            EnumerationLimits {
                entries,
                scanned_entries,
                path_utf16_units,
            },
            EnumerationVisibility::default(),
        )
    }

    fn name(value: &str) -> Vec<u16> {
        value.encode_utf16().chain([0]).collect()
    }

    #[test]
    fn comparison_keys_are_attached_without_path_normalization() {
        let operation = OperationId(42);
        let snapshot = ShellMetadataSnapshot {
            operation,
            entries: vec![ShellMetadataEntry {
                path: OsString::from(r"\\server\Share\MiXeD.txt"),
                kind: ItemKind::File,
                attributes: 0,
            }],
        };
        let key = drape_core::domain::PathKey::new("provider-key".into()).unwrap();
        let result = attach_comparison_keys(snapshot, vec![key.clone()]).unwrap();
        assert_eq!(result.operation, operation);
        assert_eq!(
            result.entries[0].path,
            OsString::from(r"\\server\Share\MiXeD.txt")
        );
        assert_eq!(result.entries[0].key, key);
    }

    #[test]
    fn comparison_key_cardinality_mismatch_fails_closed() {
        let snapshot = ShellMetadataSnapshot {
            operation: OperationId(1),
            entries: vec![],
        };
        let key = drape_core::domain::PathKey::new("unused".into()).unwrap();
        assert_eq!(
            attach_comparison_keys(snapshot, vec![key]),
            Err(PortError::Unavailable)
        );
    }

    #[test]
    fn item_source_does_not_claim_unverified_windows_keys() {
        let cancellation = Cancellation::default();
        assert_eq!(
            WindowsDesktopItemSource.snapshot(OperationId(1), 10, &cancellation),
            Err(PortError::Unsupported)
        );
    }

    #[test]
    fn metadata_scan_budget_counts_dot_entries_before_filtering() {
        let mut scan = accumulator(1, 0, 1000);

        assert_eq!(
            scan.observe(OsStr::new("C:\\Desktop"), &name("."), 0),
            Err(PortError::LimitExceeded)
        );
        assert_eq!(scan.scanned, 1);
        assert!(scan.entries.is_empty());
    }

    #[test]
    fn metadata_scan_budget_counts_malformed_entries_before_validation() {
        let root = OsStr::new("C:\\Desktop");
        let malformed = [b'x' as u16];
        let mut scan = accumulator(1, 1, 1000);

        assert_eq!(
            scan.observe(root, &malformed, 0),
            Err(PortError::Unavailable)
        );
        assert_eq!(scan.scanned, 1);
        assert!(scan.entries.is_empty());
        assert_eq!(scan.path_units, 0);

        let mut exhausted = accumulator(1, 0, 1000);
        assert_eq!(
            exhausted.observe(root, &malformed, 0),
            Err(PortError::LimitExceeded)
        );
        assert_eq!(exhausted.scanned, 1);
    }

    #[test]
    fn metadata_scan_budget_is_shared_across_roots_and_excluded_entries() {
        let mut scan = accumulator(0, 4, 0);
        let user = OsStr::new("C:\\Desktop");
        let public = OsStr::new("C:\\PublicDesktop");

        scan.observe(user, &name("."), 0).unwrap();
        scan.observe(user, &name(".."), 0).unwrap();
        scan.observe(user, &name("hidden"), FILE_ATTRIBUTE_HIDDEN.0)
            .unwrap();
        scan.observe(public, &name("system"), FILE_ATTRIBUTE_SYSTEM.0)
            .unwrap();
        assert_eq!(scan.scanned, 4);
        assert!(scan.entries.is_empty());
        assert_eq!(scan.path_units, 0);
        assert_eq!(
            scan.observe(public, &name("desktop.ini"), 0),
            Err(PortError::LimitExceeded)
        );
    }

    #[test]
    fn metadata_scan_counts_filtered_entries_and_never_silently_truncates() {
        let mut scan = accumulator(1, 3, 1000);
        let root = OsStr::new("C:\\Desktop");
        scan.observe(root, &name("."), 0).unwrap();
        scan.observe(root, &name("DESKTOP.INI"), 0).unwrap();
        scan.observe(root, &name("one.txt"), 0).unwrap();
        assert_eq!(
            scan.observe(root, &name("hidden.txt"), FILE_ATTRIBUTE_HIDDEN.0),
            Err(PortError::LimitExceeded)
        );
        assert_eq!(scan.entries.len(), 1);
        let mut limited = accumulator(0, 1, 1000);
        assert_eq!(
            limited.observe(root, &name("one.txt"), 0),
            Err(PortError::LimitExceeded)
        );
    }

    #[test]
    fn metadata_visibility_filters_independently_and_always_ignores_desktop_ini() {
        let root = OsStr::new("C:\\Desktop");
        let mut scan = accumulator(4, 10, 1000);
        scan.visibility.show_hidden = true;
        scan.observe(root, &name("hidden"), FILE_ATTRIBUTE_HIDDEN.0)
            .unwrap();
        scan.observe(root, &name("system"), FILE_ATTRIBUTE_SYSTEM.0)
            .unwrap();
        scan.observe(root, &name("desktop.ini"), 0).unwrap();
        assert_eq!(scan.entries.len(), 1);
        scan.visibility.show_system = true;
        scan.observe(root, &name("system"), FILE_ATTRIBUTE_SYSTEM.0)
            .unwrap();
        assert_eq!(scan.entries.len(), 2);
    }

    #[test]
    fn metadata_preserves_literal_utf16_and_classifies_without_opening_children() {
        let root = OsStr::new("\\\\server\\share\\Desktop");
        let mut scan = accumulator(4, 4, 1000);
        let odd_name = [
            0xd800,
            b'.' as u16,
            b't' as u16,
            b'x' as u16,
            b't' as u16,
            0,
        ];
        scan.observe(
            root,
            &odd_name,
            FILE_ATTRIBUTE_OFFLINE.0 | FILE_ATTRIBUTE_REPARSE_POINT.0,
        )
        .unwrap();
        scan.observe(root, &name("LINK.LNK"), 0).unwrap();
        scan.observe(root, &name("site.URL"), 0).unwrap();
        scan.observe(root, &name("folder.lnk"), FILE_ATTRIBUTE_DIRECTORY.0)
            .unwrap();
        let expected = PathBuf::from(root).join(OsString::from_wide(&odd_name[..5]));
        assert_eq!(scan.entries[0].path, expected.into_os_string());
        assert_eq!(
            scan.entries
                .iter()
                .map(|entry| entry.kind)
                .collect::<Vec<_>>(),
            vec![
                ItemKind::File,
                ItemKind::Shortcut,
                ItemKind::Shortcut,
                ItemKind::Folder
            ]
        );
        assert_eq!(
            scan.entries[0].attributes,
            FILE_ATTRIBUTE_OFFLINE.0 | FILE_ATTRIBUTE_REPARSE_POINT.0
        );
    }

    #[test]
    fn metadata_path_budget_rejects_before_retaining_a_partial_entry() {
        let root = OsStr::new("C:\\D");
        let mut scan = accumulator(2, 2, 8);
        scan.observe(root, &name("one"), 0).unwrap();
        assert_eq!(
            scan.observe(root, &name("two"), 0),
            Err(PortError::LimitExceeded)
        );
        assert_eq!(scan.entries.len(), 1);
        assert_eq!(scan.path_units, 8);
    }

    #[test]
    fn metadata_rejects_malformed_components_and_honors_precancel_before_native_work() {
        for invalid in [
            vec![b'x' as u16],
            name(""),
            name("a\\b"),
            name("C:evil"),
            name("/abs"),
        ] {
            let mut scan = accumulator(1, 1, 1000);
            assert_eq!(
                scan.observe(OsStr::new("C:\\D"), &invalid, 0),
                Err(PortError::Unavailable)
            );
            assert!(scan.entries.is_empty());
        }
        let cancellation = Cancellation::default();
        cancellation.cancel();
        assert_eq!(
            probe_desktop_metadata(
                OperationId(7),
                EnumerationLimits {
                    entries: 1,
                    scanned_entries: 1,
                    path_utf16_units: 10
                },
                EnumerationVisibility::default(),
                &cancellation
            ),
            Err(PortError::Cancelled)
        );
    }
}
