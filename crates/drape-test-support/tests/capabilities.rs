use drape_core::PlatformCapabilities;
use drape_test_support::FakePlatformCapabilities;

#[test]
fn fake_capabilities_default_to_unsupported() {
    assert_eq!(
        FakePlatformCapabilities::default().capabilities(),
        PlatformCapabilities::default()
    );
}

#[test]
fn fake_capabilities_preserve_explicit_partial_support() {
    let expected = PlatformCapabilities {
        desktop_host: true,
        blur: true,
        ..PlatformCapabilities::default()
    };
    let fake = FakePlatformCapabilities::new(expected);

    assert_eq!(fake.capabilities(), expected);
    assert!(fake.capabilities().desktop_host);
    assert!(fake.capabilities().blur);
    assert!(!fake.capabilities().native_icon_visibility);
    assert!(!fake.capabilities().global_mouse_hook);
    assert!(!fake.capabilities().global_hotkeys);
}
