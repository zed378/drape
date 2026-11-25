use drape_core::domain::{Guid, IconBitmap, ItemKind, PathKey, PixelFormat};
use drape_core::ports::*;
use drape_test_support::*;
use std::ffi::OsString;

#[test]
fn locations_are_scripted_owned_and_never_read_real_desktop() {
    let fake = FakeDesktopLocations::new(vec![Ok(vec![
        OsString::from("fictional/Desktop"),
        OsString::from("fictional/Public"),
    ])]);
    let locations = fake.desktop_locations().unwrap();
    assert_eq!(locations.len(), 2);
    assert_eq!(fake.desktop_locations(), Err(PortError::Unavailable));
}

#[test]
fn snapshots_enforce_limit_cancellation_and_preserve_generation() {
    let entry = DesktopEntry {
        path: OsString::from("literal/%FILE%"),
        key: PathKey::new("key".into()).unwrap(),
        kind: ItemKind::File,
        file_id: None,
    };
    let fake = FakeDesktopItemSource::new(vec![Ok(vec![entry.clone()]), Ok(vec![entry])]);
    let cancellation = Cancellation::default();
    let operation = OperationId(42);
    assert_eq!(
        fake.snapshot(operation, 0, &cancellation),
        Err(PortError::LimitExceeded)
    );
    let snapshot = fake.snapshot(operation, 1, &cancellation).unwrap();
    assert_eq!(snapshot.operation, operation);
    assert_eq!(snapshot.entries[0].path, OsString::from("literal/%FILE%"));
    cancellation.cancel();
    assert_eq!(
        fake.snapshot(OperationId(43), 1, &cancellation),
        Err(PortError::Cancelled)
    );
}

#[test]
fn icon_requests_report_scripted_errors_and_cancel_without_consuming_script() {
    let bitmap = IconBitmap::new(1, 1, 4, PixelFormat::Rgba8, vec![1; 4], 4).unwrap();
    let fake = FakeIconProvider::new(vec![Err(PortError::Unsupported), Ok(bitmap)]);
    let request = IconRequest {
        operation: OperationId(7),
        item: Guid::from_bytes([1; 16]).unwrap(),
        path: OsString::from("fictional/icon"),
        pixel_size: 16,
        byte_limit: 4,
    };
    let cancellation = Cancellation::default();
    assert_eq!(
        fake.load_icon(&request, &cancellation),
        Err(PortError::Unsupported)
    );
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        fake.load_icon(&request, &cancelled),
        Err(PortError::Cancelled)
    );
    let response = fake.load_icon(&request, &cancellation).unwrap();
    assert_eq!(response.operation, OperationId(7));
    assert_eq!(response.item, request.item);
    assert_eq!(response.bitmap.pixels(), &[1; 4]);
    assert_eq!(
        fake.load_icon(&request, &cancellation),
        Err(PortError::Unavailable)
    );
}
