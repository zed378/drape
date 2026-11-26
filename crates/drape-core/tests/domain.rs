use drape_core::TimestampMillis;
use drape_core::domain::*;
use std::ffi::OsString;

fn id(n: u8) -> Guid {
    let mut bytes = [0; 16];
    bytes[15] = n;
    Guid::from_bytes(bytes).unwrap()
}

fn bounds() -> Bounds {
    Bounds::new(
        "monitor".into(),
        [10.0, 20.0, 320.0, 240.0],
        144.0,
        [0.1, 0.2, 0.3, 0.4],
    )
    .unwrap()
}

fn item(n: u8, key: &str, added: i64) -> DrapeItem {
    DrapeItem::new(
        id(n),
        ItemKind::File,
        OsString::from("%LITERAL%/résumé.txt"),
        PathKey::new(key.into()).unwrap(),
        TimestampMillis(added),
    )
    .unwrap()
}

fn panel(n: u8, items: Vec<DrapeItem>) -> Drape {
    Drape::new(id(n), id(1), "Reports".into(), bounds(), items).unwrap()
}

fn snapshot_drape(n: u8, items: Vec<DrapeItem>) -> DrapeSnapshot {
    DrapeSnapshot::new(id(n), bounds(), Style::default(), items).unwrap()
}

#[test]
fn guid_rejects_nil_malformed_and_noncanonical_identity() {
    assert!(Guid::from_bytes([0; 16]).is_err());
    for value in [
        "",
        "00000000-0000-0000-0000-000000000000",
        "x0000000-0000-4000-8000-000000000001",
        "00000000000040008000000000000001",
    ] {
        assert!(Guid::parse(value).is_err(), "{value}");
    }
    let value = Guid::parse("6F1C2B8E-0000-4000-8000-000000000001").unwrap();
    assert_eq!(value.to_string(), "6f1c2b8e-0000-4000-8000-000000000001");
}

#[test]
fn bounds_reject_all_nonfinite_values_and_invalid_dimensions() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..4 {
            let mut rectangle = [0.0, 0.0, 120.0, 80.0];
            rectangle[index] = invalid;
            assert!(Bounds::new("m".into(), rectangle, 96.0, [0.0; 4]).is_err());
            let mut fractions = [0.0; 4];
            fractions[index] = invalid;
            assert!(Bounds::new("m".into(), [0.0, 0.0, 120.0, 80.0], 96.0, fractions).is_err());
        }
        assert!(Bounds::new("m".into(), [0.0, 0.0, 120.0, 80.0], invalid, [0.0; 4]).is_err());
    }
    assert!(bounds().with_size(119.0, 80.0).is_err());
    assert!(bounds().with_size(120.0, 79.0).is_err());
    assert!(bounds().with_dpi(0.0).is_err());
    assert!(Bounds::new(" ".into(), [0.0, 0.0, 120.0, 80.0], 96.0, [0.0; 4]).is_err());
    assert!(
        Bounds::new(
            "m".into(),
            [0.0, 0.0, 120.0, 80.0],
            96.0,
            [1.1, 0.0, 0.0, 0.0]
        )
        .is_err()
    );
    assert_eq!(
        bounds().with_position(-400.0, -20.0).unwrap().rectangle()[0],
        -400.0
    );
}

#[test]
fn literal_paths_and_adapter_comparison_keys_are_preserved() {
    let entry = item(10, "opaque-A", -1);
    assert_eq!(entry.path(), &OsString::from("%LITERAL%/résumé.txt"));
    let distinct = item(11, "opaque-a", -1);
    assert!(panel(2, vec![entry.clone()]).add_item(distinct).is_ok());
    assert!(
        panel(2, vec![entry])
            .add_item(item(12, "opaque-A", 0))
            .is_err()
    );
    assert!(PathKey::new(String::new()).is_err());
    assert!(
        DrapeItem::new(
            id(10),
            ItemKind::File,
            OsString::new(),
            PathKey::new("a".into()).unwrap(),
            TimestampMillis(0)
        )
        .is_err()
    );
}

#[test]
fn panel_edits_validate_and_only_remove_references() {
    assert!(Page::new(id(1), " ".into(), 0).is_err());
    assert!(panel(2, vec![]).rename("".into()).is_err());
    assert!(Drape::new(id(2), id(1), " ".into(), bounds(), vec![]).is_err());
    assert!(
        Drape::new(
            id(2),
            id(1),
            "a".into(),
            bounds(),
            vec![item(10, "a", 0), item(10, "b", 0)]
        )
        .is_err()
    );
    let original = panel(2, vec![item(10, "a", 0), item(11, "b", 0)]);
    let renamed = original.rename("New".into()).unwrap();
    assert_eq!(original.title(), "Reports");
    assert_eq!(renamed.title(), "New");
    let removed = original.remove_item(id(10));
    assert_eq!(removed.items().len(), 1);
    assert_eq!(removed.items()[0].order(), 0);
    assert_eq!(original.items().len(), 2);
    assert!(original.add_item(item(10, "different", 1)).is_err());
}

#[test]
fn layout_snapshot_rejects_blank_names_and_duplicate_drape_ids() {
    assert_eq!(
        LayoutSnapshot::new(id(20), " \t".into(), TimestampMillis(1), vec![]),
        Err(DomainError::Invalid("snapshot name"))
    );

    let drape = snapshot_drape(2, vec![]);
    assert_eq!(
        LayoutSnapshot::new(
            id(20),
            "Morning".into(),
            TimestampMillis(1),
            vec![drape.clone(), drape]
        ),
        Err(DomainError::Duplicate("Drape id"))
    );

    let empty = LayoutSnapshot::new(id(20), "Empty".into(), TimestampMillis(0), vec![]).unwrap();
    assert!(empty.drapes().is_empty());
}

#[test]
fn layout_snapshot_rejects_duplicate_item_ids_and_path_keys_across_drapes() {
    assert_eq!(
        LayoutSnapshot::new(
            id(20),
            "Morning".into(),
            TimestampMillis(1),
            vec![
                snapshot_drape(2, vec![item(10, "first", 2)]),
                snapshot_drape(3, vec![item(10, "second", 3)])
            ]
        ),
        Err(DomainError::Duplicate("item id"))
    );

    assert_eq!(
        LayoutSnapshot::new(
            id(20),
            "Morning".into(),
            TimestampMillis(1),
            vec![
                snapshot_drape(2, vec![item(10, "same-path", 2)]),
                snapshot_drape(3, vec![item(11, "same-path", 3)])
            ]
        ),
        Err(DomainError::Duplicate("membership"))
    );
}

#[test]
fn drape_snapshot_rejects_duplicate_membership_and_layout_values_are_preserved() {
    assert_eq!(
        DrapeSnapshot::new(
            id(2),
            bounds(),
            Style::default(),
            vec![item(10, "first", 2), item(10, "second", 3)]
        ),
        Err(DomainError::Duplicate("item id"))
    );
    assert_eq!(
        DrapeSnapshot::new(
            id(2),
            bounds(),
            Style::default(),
            vec![item(10, "same-path", 2), item(11, "same-path", 3)]
        ),
        Err(DomainError::Duplicate("membership"))
    );

    let saved_bounds = Bounds::new(
        "opaque-display".into(),
        [-333.25, 71.5, 351.75, 211.125],
        192.0,
        [0.17, 0.25, 0.61, 0.79],
    )
    .unwrap();
    let mut saved_style = Style::new(0.73, 7.5, 14.25, 64.0, 9.5).unwrap();
    saved_style.tint = Some("#123456".into());
    let mut entry = item(10, "opaque-A", -1);
    entry.mark_missing(TimestampMillis(7));
    entry.pinned_by_user = true;
    entry.display_name = Some("Literal label".into());
    let source = panel(2, vec![entry, item(11, "opaque-B", 2)]);
    let mut saved_items = source.items().to_vec();
    saved_items.reverse();
    let drape = DrapeSnapshot::new(
        id(2),
        saved_bounds.clone(),
        saved_style.clone(),
        saved_items.clone(),
    )
    .unwrap();
    let snapshot = LayoutSnapshot::new(
        id(20),
        "Saved layout".into(),
        TimestampMillis(-50),
        vec![drape],
    )
    .unwrap();

    assert_eq!(snapshot.id(), id(20));
    assert_eq!(snapshot.name(), "Saved layout");
    assert_eq!(snapshot.created_at(), TimestampMillis(-50));
    assert_eq!(snapshot.drapes()[0].drape_id(), id(2));
    assert_eq!(snapshot.drapes()[0].bounds(), &saved_bounds);
    assert_eq!(snapshot.drapes()[0].style(), &saved_style);
    assert_eq!(snapshot.drapes()[0].items(), saved_items);
}

#[test]
fn config_rejects_empty_pages_duplicate_ids_unknown_pages_and_membership() {
    let page = Page::new(id(1), "Main".into(), 0).unwrap();
    assert!(DomainConfig::new(1, vec![], vec![]).is_err());
    assert!(DomainConfig::new(0, vec![page.clone()], vec![]).is_err());
    assert!(DomainConfig::new(1, vec![page.clone(), page.clone()], vec![]).is_err());
    let p = panel(2, vec![]);
    assert!(DomainConfig::new(1, vec![page.clone()], vec![p.clone(), p]).is_err());
    let unknown = Drape::new(id(2), id(99), "a".into(), bounds(), vec![]).unwrap();
    assert!(DomainConfig::new(1, vec![page.clone()], vec![unknown]).is_err());
    assert!(
        DomainConfig::new(
            1,
            vec![page.clone()],
            vec![
                panel(2, vec![item(10, "same", 0)]),
                panel(3, vec![item(11, "same", 1)])
            ]
        )
        .is_err()
    );
    assert!(
        DomainConfig::new(
            1,
            vec![page],
            vec![
                panel(2, vec![item(10, "a", 0)]),
                panel(3, vec![item(10, "b", 1)])
            ]
        )
        .is_err()
    );
}

#[test]
fn membership_repair_uses_newest_timestamp_then_stable_panel_order() {
    let page = Page::new(id(1), "Main".into(), 0).unwrap();
    let panels = vec![
        panel(3, vec![item(10, "same", 1), item(12, "tie", 5)]),
        panel(2, vec![item(11, "same", 2), item(13, "tie", 5)]),
    ];
    let (config, repairs) = DomainConfig::reconcile(vec![page], panels, None).unwrap();
    assert_eq!(config.drapes()[0].items().len(), 1);
    assert_eq!(config.drapes()[0].items()[0].id(), id(12));
    assert_eq!(config.drapes()[1].items()[0].id(), id(11));
    assert_eq!(repairs.len(), 2);
}

#[test]
fn conflicting_identity_is_rejected_instead_of_removing_unrelated_reference() {
    let page = Page::new(id(1), "Main".into(), 0).unwrap();
    let panels = vec![
        panel(2, vec![item(10, "first", 1)]),
        panel(3, vec![item(10, "unrelated", 2)]),
    ];
    assert_eq!(
        DomainConfig::reconcile(vec![page], panels.clone(), None),
        Err(DomainError::Duplicate("item id"))
    );
    assert_eq!(panels[0].items()[0].path_key().as_str(), "first");
    assert_eq!(panels[1].items()[0].path_key().as_str(), "unrelated");
}

#[test]
fn missing_page_repair_requires_caller_supplied_default_and_reports_changes() {
    let unknown = Drape::new(id(2), id(99), "a".into(), bounds(), vec![]).unwrap();
    assert!(DomainConfig::reconcile(vec![], vec![unknown.clone()], None).is_err());
    let page = Page::new(id(1), "Main".into(), 0).unwrap();
    let (config, repairs) = DomainConfig::reconcile(vec![], vec![unknown], Some(page)).unwrap();
    assert_eq!(config.drapes()[0].page_id(), id(1));
    assert_eq!(repairs.len(), 1);
    assert!(config.remove_drape(id(2)).unwrap().drapes().is_empty());
}

#[test]
fn newer_schema_is_read_only_and_defaults_are_offline() {
    let page = Page::new(id(1), "Main".into(), 0).unwrap();
    let config = DomainConfig::new(2, vec![page], vec![]).unwrap();
    assert!(config.is_read_only());
    assert!(config.add_drape(panel(2, vec![])).is_err());
    assert!(config.remove_drape(id(2)).is_err());
    assert!(config.replace_drape(panel(2, vec![])).is_err());
    assert!(config.with_settings(Settings::default()).is_err());
    assert_eq!(Settings::default().update_check, UpdateCheck::Off);
    assert!(!Settings::default().autostart);
}

#[test]
fn styles_reject_nonfinite_render_values() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..5 {
            let mut values = [0.45, 8.0, 13.0, 48.0, 12.0];
            values[index] = invalid;
            assert!(Style::new(values[0], values[1], values[2], values[3], values[4]).is_err());
        }
    }
    assert!(Style::new(1.1, 8.0, 13.0, 48.0, 12.0).is_err());
    assert!(Style::new(0.45, -1.0, 13.0, 48.0, 12.0).is_err());
    assert!(Style::new(0.45, 8.0, 0.0, 48.0, 12.0).is_err());
    assert!(Style::new(0.45, 8.0, 13.0, 15.0, 12.0).is_err());
    assert!(Style::new(0.45, 8.0, 13.0, 257.0, 12.0).is_err());
    assert!(Style::new(0.45, 8.0, 13.0, 48.0, -1.0).is_err());
}

#[test]
fn missing_observations_preserve_original_time_until_present() {
    let mut entry = item(10, "a", -1);
    entry.mark_missing(TimestampMillis(10));
    entry.mark_missing(TimestampMillis(20));
    assert_eq!(entry.missing_since(), Some(TimestampMillis(10)));
    entry.mark_present();
    assert_eq!(entry.missing_since(), None);
}

#[test]
fn config_edits_preserve_settings_and_validate_replacement() {
    let page = Page::new(id(1), "Main".into(), 0).unwrap();
    let settings = Settings {
        language: "id-ID".into(),
        ..Settings::default()
    };
    let config = DomainConfig::new(1, vec![page], vec![])
        .unwrap()
        .with_settings(settings.clone())
        .unwrap();
    assert!(config.replace_drape(panel(2, vec![])).is_err());
    let config = config.add_drape(panel(2, vec![])).unwrap();
    assert_eq!(config.settings(), &settings);
    let config = config
        .replace_drape(panel(2, vec![]).rename("Changed".into()).unwrap())
        .unwrap();
    assert_eq!(config.drapes()[0].title(), "Changed");
    assert_eq!(config.settings(), &settings);
    assert_eq!(config.remove_drape(id(2)).unwrap().settings(), &settings);
}

#[test]
fn bitmap_rejects_invalid_stride_overflow_and_resource_limit() {
    assert!(IconBitmap::new(0, 1, 4, PixelFormat::Rgba8, vec![0; 4], 32).is_err());
    assert!(IconBitmap::new(2, 1, 4, PixelFormat::Rgba8, vec![0; 4], 32).is_err());
    assert!(IconBitmap::new(1, 2, 4, PixelFormat::Rgba8, vec![0; 7], 32).is_err());
    assert!(IconBitmap::new(1, 2, 4, PixelFormat::Rgba8, vec![0; 8], 7).is_err());
    assert!(IconBitmap::new(u32::MAX, u32::MAX, u32::MAX, PixelFormat::Rgba8, vec![], 32).is_err());
    let pixels = vec![1; 12];
    let icon = IconBitmap::new(1, 2, 6, PixelFormat::Bgra8, pixels, 12).unwrap();
    assert_eq!(icon.pixels(), &[1; 12]);
    assert_eq!(icon.stride(), 6);
}
