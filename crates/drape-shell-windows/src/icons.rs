//! Owned 32-bit pixel conversion helpers for the Windows shell adapter.
//!
//! The caller must establish channel order and alpha semantics before selecting a
//! `PixelFormat`. This module does not access native handles or claim shell API behavior.

use drape_core::domain::{DomainError, IconBitmap, PixelFormat};
use std::collections::VecDeque;
use std::ffi::OsString;
use std::fmt;

/// The caller supplies a platform item identity and changes `revision` whenever
/// its icon becomes stale. Physical size includes the requested DPI scale.
/// Literal identities are not normalized or merged by this cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconCacheKey {
    pub identity: OsString,
    pub pixel_size: u32,
    pub revision: u64,
}

/// Worker-owned LRU cache of validated pixels, with no native or UI resources.
///
/// Both retained pixel payload bytes and entry count are bounded. The byte limit
/// excludes allocator overhead and keys; each identity is separately bounded to
/// 64 KiB. This is not a process working-set limit. No executor or locking is
/// implicit; the icon worker owns and invalidates its cache.
#[derive(Debug)]
pub struct IconCache {
    entries: VecDeque<(IconCacheKey, IconBitmap)>,
    byte_limit: usize,
    entry_limit: usize,
    pixel_bytes: usize,
}

impl IconCache {
    pub fn new(byte_limit: usize, entry_limit: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            byte_limit,
            entry_limit,
            pixel_bytes: 0,
        }
    }

    pub fn pixel_bytes(&self) -> usize {
        self.pixel_bytes
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Borrows pixels and marks the matching entry most recently used.
    pub fn get(&mut self, key: &IconCacheKey) -> Option<&IconBitmap> {
        let position = self.entries.iter().position(|(stored, _)| stored == key)?;
        let entry = self.entries.remove(position)?;
        self.entries.push_back(entry);
        self.entries.back().map(|(_, bitmap)| bitmap)
    }

    /// Rejects invalid/oversized entries without changing existing cached data.
    /// Smaller shell icons are permitted; upscaling remains a UI policy.
    pub fn insert(&mut self, mut key: IconCacheKey, bitmap: IconBitmap) -> bool {
        let bytes = bitmap.pixels().len();
        if self.entry_limit == 0
            || bytes > self.byte_limit
            || key.identity.is_empty()
            || key.identity.as_encoded_bytes().len() > 65_536
            || key.pixel_size == 0
            || bitmap.width() > key.pixel_size
            || bitmap.height() > key.pixel_size
        {
            return false;
        }
        // Do not retain excess caller-reserved capacity in either allocation.
        key.identity = key.identity.into_boxed_os_str().into_os_string();
        let Ok(bitmap) = IconBitmap::new(
            bitmap.width(),
            bitmap.height(),
            bitmap.stride(),
            bitmap.format(),
            bitmap.pixels().to_vec(),
            self.byte_limit,
        ) else {
            return false;
        };
        if let Some(position) = self.entries.iter().position(|(stored, _)| stored == &key) {
            let (_, previous) = self.entries.remove(position).expect("position exists");
            self.pixel_bytes -= previous.pixels().len();
        }
        while self.entries.len() >= self.entry_limit || bytes > self.byte_limit - self.pixel_bytes {
            if let Some((_, previous)) = self.entries.pop_front() {
                self.pixel_bytes -= previous.pixels().len();
            }
        }
        self.pixel_bytes += bytes;
        self.entries.push_back((key, bitmap));
        true
    }

    /// Evicts every size/revision of an item after an observed change or removal.
    pub fn invalidate(&mut self, identity: &std::ffi::OsStr) {
        self.entries.retain(|(key, bitmap)| {
            if key.identity == identity {
                self.pixel_bytes -= bitmap.pixels().len();
                false
            } else {
                true
            }
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.pixel_bytes = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DibConversionError {
    InvalidStride,
    BufferLengthMismatch,
    ResourceLimitExceeded,
    InvalidBitmap,
}

impl fmt::Display for DibConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidStride => "invalid 32-bit DIB stride",
            Self::BufferLengthMismatch => "32-bit DIB buffer length mismatch",
            Self::ResourceLimitExceeded => "32-bit DIB exceeds the icon byte limit",
            Self::InvalidBitmap => "invalid 32-bit DIB dimensions",
        })
    }
}

impl std::error::Error for DibConversionError {}

/// Copies bottom-up 32-bit rows into an owned top-down [`IconBitmap`].
///
/// `dib_stride` is the byte distance between source rows. `pixel_format` records
/// already-verified channel and alpha semantics; this function does not infer them.
/// The copied bitmap is tightly packed, and allocation is bounded by `byte_limit`.
pub fn icon_bitmap_from_bottom_up_dib32(
    width: u32,
    height: u32,
    dib_stride: usize,
    pixel_format: PixelFormat,
    dib_pixels: &[u8],
    byte_limit: usize,
) -> Result<IconBitmap, DibConversionError> {
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or(DibConversionError::InvalidBitmap)?;
    if dib_stride < row_bytes {
        return Err(DibConversionError::InvalidStride);
    }
    let source_height =
        usize::try_from(height).map_err(|_| DibConversionError::ResourceLimitExceeded)?;
    let source_len = dib_stride
        .checked_mul(source_height)
        .ok_or(DibConversionError::ResourceLimitExceeded)?;
    if dib_pixels.len() != source_len {
        return Err(DibConversionError::BufferLengthMismatch);
    }

    let stride = u32::try_from(row_bytes).map_err(|_| DibConversionError::ResourceLimitExceeded)?;
    let output_len =
        IconBitmap::required_bytes(width, height, stride, byte_limit).map_err(map_domain_error)?;
    let mut pixels = Vec::with_capacity(output_len);
    for row in (0..source_height).rev() {
        let start = row
            .checked_mul(dib_stride)
            .ok_or(DibConversionError::ResourceLimitExceeded)?;
        let end = start
            .checked_add(row_bytes)
            .ok_or(DibConversionError::ResourceLimitExceeded)?;
        pixels.extend_from_slice(&dib_pixels[start..end]);
    }

    IconBitmap::new(width, height, stride, pixel_format, pixels, byte_limit)
        .map_err(map_domain_error)
}

fn map_domain_error(error: DomainError) -> DibConversionError {
    match error {
        DomainError::LimitExceeded => DibConversionError::ResourceLimitExceeded,
        DomainError::Invalid(_) => DibConversionError::InvalidBitmap,
        _ => DibConversionError::InvalidBitmap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(identity: &str, pixel_size: u32, revision: u64) -> IconCacheKey {
        IconCacheKey {
            identity: identity.into(),
            pixel_size,
            revision,
        }
    }

    fn bitmap(width: u32, value: u8) -> IconBitmap {
        IconBitmap::new(
            width,
            1,
            width * 4,
            PixelFormat::Bgra8,
            vec![value; width as usize * 4],
            64,
        )
        .unwrap()
    }

    #[test]
    fn cache_evicts_least_recently_used_pixels_within_byte_budget() {
        let mut cache = IconCache::new(12, 10);
        assert!(cache.insert(key("a", 1, 0), bitmap(1, 1)));
        assert!(cache.insert(key("b", 1, 0), bitmap(1, 2)));
        assert_eq!(cache.get(&key("a", 1, 0)).unwrap().pixels(), &[1; 4]);
        assert!(cache.insert(key("c", 2, 0), bitmap(2, 3)));
        assert!(cache.get(&key("b", 1, 0)).is_none());
        assert!(cache.get(&key("a", 1, 0)).is_some());
        assert_eq!(cache.pixel_bytes(), 12);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn cache_replacement_accounts_for_old_pixels_and_rejects_oversized_atomically() {
        let mut cache = IconCache::new(8, 2);
        let cache_key = key("a", 3, 0);
        assert!(cache.insert(cache_key.clone(), bitmap(1, 1)));
        assert!(cache.insert(cache_key.clone(), bitmap(2, 2)));
        assert_eq!(cache.pixel_bytes(), 8);
        assert_eq!(cache.len(), 1);
        assert!(!cache.insert(cache_key.clone(), bitmap(3, 3)));
        assert_eq!(cache.get(&cache_key).unwrap().pixels(), &[2; 8]);
        assert_eq!(cache.pixel_bytes(), 8);
    }

    #[test]
    fn cache_keys_distinguish_scale_revision_and_literal_identity() {
        let mut cache = IconCache::new(64, 4);
        for (entry, value) in [
            (key("a", 1, 0), 1),
            (key("a", 2, 0), 2),
            (key("a", 1, 1), 3),
            (key("A", 1, 0), 4),
        ] {
            assert!(cache.insert(entry, bitmap(1, value)));
        }
        assert_eq!(cache.get(&key("a", 1, 0)).unwrap().pixels(), &[1; 4]);
        assert_eq!(cache.get(&key("a", 2, 0)).unwrap().pixels(), &[2; 4]);
        assert_eq!(cache.get(&key("a", 1, 1)).unwrap().pixels(), &[3; 4]);
        cache.invalidate(std::ffi::OsStr::new("a"));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.pixel_bytes(), 4);
        assert!(cache.get(&key("A", 1, 0)).is_some());
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.pixel_bytes(), 0);
    }

    #[test]
    fn cache_entry_limit_bounds_tiny_icons_and_zero_disables_caching() {
        let mut cache = IconCache::new(64, 1);
        assert!(cache.insert(key("a", 1, 0), bitmap(1, 1)));
        assert!(cache.insert(key("b", 1, 0), bitmap(1, 2)));
        assert!(cache.get(&key("a", 1, 0)).is_none());
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.pixel_bytes(), 4);
        assert!(!IconCache::new(0, 1).insert(key("a", 1, 0), bitmap(1, 1)));
        assert!(!IconCache::new(4, 0).insert(key("a", 1, 0), bitmap(1, 1)));
    }

    #[test]
    fn cache_rejects_unbounded_identity_and_wrong_requested_size() {
        let mut cache = IconCache::new(16, 4);
        assert!(!cache.insert(key("", 1, 0), bitmap(1, 1)));
        assert!(!cache.insert(key(&"x".repeat(65_537), 1, 0), bitmap(1, 1)));
        assert!(!cache.insert(key("a", 0, 0), bitmap(1, 1)));
        assert!(!cache.insert(key("a", 1, 0), bitmap(2, 1)));
        assert!(cache.is_empty());
        assert_eq!(cache.pixel_bytes(), 0);
    }

    #[test]
    fn dib_conversion_skips_source_padding_and_preserves_channel_bytes() {
        let bitmap = icon_bitmap_from_bottom_up_dib32(
            1,
            2,
            8,
            PixelFormat::Rgba8,
            &[5, 6, 7, 8, 99, 99, 99, 99, 1, 2, 3, 4, 99, 99, 99, 99],
            8,
        )
        .unwrap();
        assert_eq!(bitmap.pixels(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(bitmap.stride(), 4);
    }

    #[test]
    fn dib_rows_are_copied_as_owned_top_down_pixels() {
        let source = [5, 6, 7, 8, 1, 2, 3, 4];
        let bitmap =
            icon_bitmap_from_bottom_up_dib32(1, 2, 4, PixelFormat::Bgra8, &source, 8).unwrap();

        assert_eq!(bitmap.pixels(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(bitmap.stride(), 4);
    }

    #[test]
    fn dib_conversion_rejects_short_stride_and_mismatched_buffer() {
        assert_eq!(
            icon_bitmap_from_bottom_up_dib32(2, 1, 7, PixelFormat::Bgra8, &[0; 8], 8,),
            Err(DibConversionError::InvalidStride)
        );
        assert_eq!(
            icon_bitmap_from_bottom_up_dib32(1, 2, 4, PixelFormat::Bgra8, &[0; 7], 8,),
            Err(DibConversionError::BufferLengthMismatch)
        );
    }

    #[test]
    fn dib_conversion_enforces_the_owned_pixel_byte_limit() {
        assert_eq!(
            icon_bitmap_from_bottom_up_dib32(1, 2, 4, PixelFormat::Bgra8, &[0; 8], 7,),
            Err(DibConversionError::ResourceLimitExceeded)
        );
    }
}
