use drape_core::domain::Bounds;
use drape_core::layout::{
    LayoutError, Monitor, Rect, Resolution, StoredLayout, clamp, fractions, resolve, snap,
};

fn rect(value: [f64; 4]) -> Rect {
    Rect::new(value).unwrap()
}
fn monitor(key: &str, area: [f64; 4], dpi: f64, primary: bool) -> Monitor {
    Monitor::new(key.into(), rect(area), dpi, primary).unwrap()
}
fn stored() -> StoredLayout {
    StoredLayout::new(
        Bounds::new(
            "original".into(),
            [200., 100., 400., 200.],
            96.,
            [0.2, 0.2, 0.4, 0.4],
        )
        .unwrap(),
        [1000., 500.],
    )
    .unwrap()
}

#[test]
fn clamp_preserves_forty_dip_and_minimum_size() {
    assert_eq!(
        clamp(rect([-999., 999., 20., 10.]), [1000., 500.])
            .unwrap()
            .values(),
        [-80., 460., 120., 80.]
    );
    assert_eq!(
        clamp(rect([10., 10., 4000., 2000.]), [1000., 500.])
            .unwrap()
            .values(),
        [10., 10., 1000., 500.]
    );
}

#[test]
fn tiny_work_area_keeps_minimum_size_and_accessible_top() {
    assert_eq!(
        clamp(rect([500., -500., 300., 300.]), [20., 10.])
            .unwrap()
            .values(),
        [0., 0., 120., 80.]
    );
}

#[test]
fn invalid_geometry_and_arithmetic_overflow_fail_explicitly() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Rect::new([value, 0., 100., 100.]).is_err());
        assert!(Rect::new([0., 0., value, 100.]).is_err());
        assert!(Monitor::new("x".into(), rect([0., 0., 100., 100.]), value, true).is_err());
    }
    assert!(Rect::new([0., 0., 0., 100.]).is_err());
    assert!(Rect::new([f64::MAX, 0., f64::MAX, 100.]).is_err());
    assert!(clamp(rect([0., 0., 100., 100.]), [0., 500.]).is_err());
    assert!(StoredLayout::new(stored().bounds().clone(), [f64::NAN, 10.]).is_err());
    assert!(Monitor::new(" ".into(), rect([0., 0., 100., 100.]), 96., true).is_err());
}

#[test]
fn fractions_use_monitor_local_coordinates_and_clamp_negative_offsets() {
    assert_eq!(
        fractions(rect([-50., 50., 400., 100.]), [1000., 500.]).unwrap(),
        [0., 0.1, 0.4, 0.2]
    );
    assert_eq!(
        fractions(rect([1100., 700., 2000., 1000.]), [1000., 500.]).unwrap(),
        [1.; 4]
    );
}

#[test]
fn resolution_and_dpi_cycles_always_use_original_snapshot_without_drift() {
    let original = stored();
    for _ in 0..10 {
        let changed = resolve(
            &original,
            &[monitor("original", [-800., -600., 800., 400.], 192., true)],
        )
        .unwrap();
        assert_eq!(changed.bounds().rectangle(), [160., 80., 320., 160.]);
        assert_eq!(
            changed.global_rectangle().unwrap().values(),
            [-640., -520., 320., 160.]
        );
        assert_eq!(changed.reason(), Resolution::Rescaled);
        let restored = resolve(
            &original,
            &[monitor("original", [0., 0., 1000., 500.], 96., true)],
        )
        .unwrap();
        assert_eq!(restored.bounds(), original.bounds());
        assert_eq!(restored.reason(), Resolution::Exact);
    }
    let dpi_only = resolve(
        &original,
        &[monitor("original", [0., 0., 1000., 500.], 288., true)],
    )
    .unwrap();
    assert_eq!(dpi_only.bounds().rectangle(), original.bounds().rectangle());
    assert_eq!(dpi_only.bounds().dpi(), 288.);
    assert_eq!(original.bounds().dpi(), 96.);
}

#[test]
fn missing_monitor_uses_fractions_on_primary_and_returns_without_overwriting_home() {
    let original = stored();
    let fallback = resolve(
        &original,
        &[monitor("other", [-1200., 0., 1200., 800.], 144., true)],
    )
    .unwrap();
    assert_eq!(fallback.bounds().rectangle(), [240., 160., 480., 320.]);
    assert_eq!(fallback.bounds().monitor_id(), "other");
    assert_eq!(fallback.reason(), Resolution::MissingMonitor);
    assert_eq!(original.bounds().monitor_id(), "original");
    assert_eq!(
        resolve(
            &original,
            &[monitor("original", [0., 0., 1000., 500.], 96., true)]
        )
        .unwrap()
        .bounds(),
        original.bounds()
    );
}

#[test]
fn ambiguous_ids_use_unique_primary_and_invalid_fallback_topologies_fail() {
    let original = stored();
    let a = monitor("original", [0., 0., 1000., 500.], 96., false);
    let b = monitor("original", [1000., 0., 1000., 500.], 96., false);
    let primary = monitor("primary", [0., 500., 800., 400.], 96., true);
    let resolved = resolve(&original, &[a.clone(), b.clone(), primary.clone()]).unwrap();
    assert_eq!(resolved.reason(), Resolution::AmbiguousMonitor);
    assert_eq!(resolved.bounds().monitor_id(), "primary");
    assert_eq!(
        resolve(&original, &[]).unwrap_err(),
        LayoutError::InvalidTopology
    );
    assert_eq!(
        resolve(
            &original,
            &[monitor("missing", [0., 0., 1000., 500.], 96., false)]
        )
        .unwrap_err(),
        LayoutError::InvalidTopology
    );
    assert_eq!(
        resolve(
            &original,
            &[
                primary.clone(),
                monitor("second-primary", [0., 0., 1000., 500.], 96., true)
            ]
        )
        .unwrap_err(),
        LayoutError::InvalidTopology
    );
    assert_eq!(
        resolve(&original, &[a, b]).unwrap_err(),
        LayoutError::InvalidTopology
    );
}

#[test]
fn matching_is_opaque_and_does_not_guess_case_or_device_aliases() {
    let result = resolve(
        &stored(),
        &[monitor("ORIGINAL", [0., 0., 1000., 500.], 96., true)],
    )
    .unwrap();
    assert_eq!(result.reason(), Resolution::MissingMonitor);
}

#[test]
fn unique_exact_monitor_does_not_require_a_primary_fallback() {
    let original = stored();
    let exact = monitor("original", [0., 0., 1000., 500.], 96., false);
    assert_eq!(
        resolve(&original, &[exact]).unwrap().bounds(),
        original.bounds()
    );
    let monitors = [
        monitor("original", [0., 0., 1000., 500.], 96., true),
        monitor("other", [1000., 0., 1000., 500.], 96., true),
    ];
    assert_eq!(
        resolve(&original, &monitors).unwrap().bounds(),
        original.bounds()
    );
}

#[test]
fn snap_threshold_is_inclusive_and_alt_bypasses_all_targets() {
    let area = [1000., 500.];
    assert_eq!(
        snap(rect([8., 8., 120., 80.]), area, &[], None, false)
            .unwrap()
            .values(),
        [0., 0., 120., 80.]
    );
    assert_eq!(
        snap(rect([8.01, 8.01, 120., 80.]), area, &[], None, false)
            .unwrap()
            .values(),
        [8.01, 8.01, 120., 80.]
    );
    let input = rect([8., 8., 120., 80.]);
    assert_eq!(snap(input, area, &[], Some(10.), true).unwrap(), input);
    assert_eq!(
        snap(rect([872., 412., 120., 80.]), area, &[], None, false)
            .unwrap()
            .values(),
        [880., 420., 120., 80.]
    );
}

#[test]
fn snap_neighbors_require_perpendicular_overlap_and_ties_are_stable() {
    let input = rect([150., 100., 120., 80.]);
    let near = rect([278., 100., 120., 80.]);
    let far = rect([278., 300., 120., 80.]);
    assert_eq!(
        snap(input, [1000., 500.], &[near], None, false)
            .unwrap()
            .values()[0],
        158.
    );
    assert_eq!(
        snap(input, [1000., 500.], &[far], None, false).unwrap(),
        input
    );
    let left = rect([22., 100., 120., 80.]);
    assert_eq!(
        snap(input, [1000., 500.], &[left, near], None, false)
            .unwrap()
            .values()[0],
        142.
    );
    let tied = rect([136., 20., 120., 80.]);
    assert_eq!(
        snap(
            rect([8., 20., 120., 80.]),
            [1000., 500.],
            &[tied],
            None,
            false
        )
        .unwrap()
        .values()[0],
        0.
    );
}

#[test]
fn grid_snapping_uses_local_origin_and_invalid_grid_is_rejected() {
    assert_eq!(
        snap(
            rect([203., 102., 120., 80.]),
            [1000., 500.],
            &[],
            Some(20.),
            false
        )
        .unwrap()
        .values(),
        [200., 100., 120., 80.]
    );
    assert_eq!(
        snap(
            rect([-203., -102., 120., 80.]),
            [1000., 500.],
            &[],
            Some(20.),
            false
        )
        .unwrap()
        .values(),
        [-200., -100., 120., 80.]
    );
    for grid in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(
            snap(
                rect([203., 102., 120., 80.]),
                [1000., 500.],
                &[],
                Some(grid),
                false
            )
            .is_err()
        );
    }
}

#[test]
fn resolving_large_ratios_reports_overflow_and_clamping_is_explicit() {
    let tiny_original =
        StoredLayout::new(stored().bounds().clone(), [f64::MIN_POSITIVE, 500.]).unwrap();
    assert_eq!(
        resolve(
            &tiny_original,
            &[monitor("original", [0., 0., 1000., 500.], 96., true)]
        )
        .unwrap_err(),
        LayoutError::ArithmeticOverflow
    );
    let offscreen = StoredLayout::new(
        Bounds::new(
            "original".into(),
            [5000., -5000., 400., 200.],
            96.,
            [1., 0., 0.4, 0.4],
        )
        .unwrap(),
        [1000., 500.],
    )
    .unwrap();
    let resolved = resolve(
        &offscreen,
        &[monitor("original", [0., 0., 1000., 500.], 96., true)],
    )
    .unwrap();
    assert_eq!(resolved.reason(), Resolution::Clamped);
    assert_eq!(resolved.bounds().rectangle(), [960., 0., 400., 200.]);
}

#[test]
fn zero_fallback_size_fractions_recover_at_minimum_and_global_overflow_is_reported() {
    let original = StoredLayout::new(
        Bounds::new("gone".into(), [0., 0., 120., 80.], 96., [0.; 4]).unwrap(),
        [1000., 500.],
    )
    .unwrap();
    let resolved = resolve(
        &original,
        &[monitor("primary", [0., 0., 20., 10.], 96., true)],
    )
    .unwrap();
    assert_eq!(resolved.bounds().rectangle(), [0., 0., 120., 80.]);
    assert_eq!(
        resolved.global_rectangle().unwrap().values(),
        [0., 0., 120., 80.]
    );
    assert!(fractions(rect([0., 0., 120., 80.]), [0., 1.]).is_err());
    assert_eq!(
        fractions(rect([0., 0., 120., 80.]), [f64::MIN_POSITIVE, 1.]).unwrap_err(),
        LayoutError::ArithmeticOverflow
    );
    assert!(
        snap(
            rect([203., 102., 120., 80.]),
            [1000., 500.],
            &[],
            Some(f64::MIN_POSITIVE),
            false
        )
        .is_err()
    );
    let current = monitor(
        "original",
        [f64::MAX * 0.75, 0., f64::MAX * 0.2, 500.],
        96.,
        true,
    );
    assert_eq!(current.key(), "original");
    assert_eq!(current.dpi(), 96.);
    assert!(current.is_primary());
    assert_eq!(current.work_area().values()[0], f64::MAX * 0.75);
    let off_right = StoredLayout::new(
        Bounds::new(
            "original".into(),
            [900., 100., 400., 200.],
            96.,
            [0.9, 0.2, 0.4, 0.4],
        )
        .unwrap(),
        [1000., 500.],
    )
    .unwrap();
    assert_eq!(
        resolve(&off_right, &[current])
            .unwrap()
            .global_rectangle()
            .unwrap_err(),
        LayoutError::ArithmeticOverflow
    );
    assert_eq!(stored().original_area(), [1000., 500.]);
    assert_eq!(
        LayoutError::InvalidGeometry.to_string(),
        "invalid layout geometry"
    );
    assert_eq!(
        LayoutError::InvalidTopology.to_string(),
        "no unique primary monitor for fallback"
    );
    assert_eq!(
        LayoutError::ArithmeticOverflow.to_string(),
        "layout arithmetic overflow"
    );
}
