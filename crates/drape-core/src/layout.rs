//! Pure monitor-relative layout. Callers retain the original snapshot across display changes.
//! Adapter-supplied monitor keys are opaque; no operating-system identity is inferred here.

use crate::domain::Bounds;
use std::fmt;

/// Fallible geometry/topology failures; no invalid input is silently treated as a monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutError {
    InvalidGeometry,
    InvalidTopology,
    ArithmeticOverflow,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidGeometry => "invalid layout geometry",
            Self::InvalidTopology => "no unique primary monitor for fallback",
            Self::ArithmeticOverflow => "layout arithmetic overflow",
        })
    }
}

impl std::error::Error for LayoutError {}

fn finite(value: f64) -> Result<f64, LayoutError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(LayoutError::ArithmeticOverflow)
    }
}

fn valid_area(size: [f64; 2]) -> Result<(), LayoutError> {
    if size.iter().all(|n| n.is_finite() && *n > 0.) {
        Ok(())
    } else {
        Err(LayoutError::InvalidGeometry)
    }
}

/// Validated x/y/width/height rectangle in DIP, with finite endpoints and positive extents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect([f64; 4]);

impl Rect {
    /// Rejects non-finite inputs, nonpositive dimensions and endpoint overflow.
    pub fn new(value: [f64; 4]) -> Result<Self, LayoutError> {
        if value.iter().any(|n| !n.is_finite()) || value[2] <= 0. || value[3] <= 0. {
            return Err(LayoutError::InvalidGeometry);
        }
        finite(value[0] + value[2])?;
        finite(value[1] + value[3])?;
        Ok(Self(value))
    }

    pub fn values(self) -> [f64; 4] {
        self.0
    }
}

/// A current work area in global DIP coordinates. Keys come from validated adapter metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    key: String,
    area: Rect,
    dpi: f64,
    primary: bool,
}

impl Monitor {
    /// Positive finite DPI is metadata; DIP geometry is never multiplied by DPI.
    pub fn new(key: String, area: Rect, dpi: f64, primary: bool) -> Result<Self, LayoutError> {
        if key.trim().is_empty() || !dpi.is_finite() || dpi <= 0. {
            return Err(LayoutError::InvalidGeometry);
        }
        Ok(Self {
            key,
            area,
            dpi,
            primary,
        })
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn work_area(&self) -> Rect {
        self.area
    }
    pub fn dpi(&self) -> f64 {
        self.dpi
    }
    pub fn is_primary(&self) -> bool {
        self.primary
    }
}

/// Original persisted position plus its reference work-area dimensions in DIP.
/// This is runtime data, not an addition to schema 1. The persistence adapter must supply
/// the reference dimensions; temporary fallback output must never replace this snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredLayout {
    bounds: Bounds,
    original_area: [f64; 2],
}

impl StoredLayout {
    pub fn new(bounds: Bounds, original_area: [f64; 2]) -> Result<Self, LayoutError> {
        valid_area(original_area)?;
        Rect::new(bounds.rectangle())?;
        Ok(Self {
            bounds,
            original_area,
        })
    }
    pub fn bounds(&self) -> &Bounds {
        &self.bounds
    }
    pub fn original_area(&self) -> [f64; 2] {
        self.original_area
    }
}

/// Why a rendered placement differs from its stored reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Exact,
    Clamped,
    Rescaled,
    MissingMonitor,
    AmbiguousMonitor,
}

/// Transient placement. Saving it as the original would lose reconnect/no-drift semantics.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLayout {
    bounds: Bounds,
    origin: [f64; 2],
    reason: Resolution,
}

impl ResolvedLayout {
    pub fn bounds(&self) -> &Bounds {
        &self.bounds
    }
    pub fn reason(&self) -> Resolution {
        self.reason
    }
    /// Converts at the adapter boundary; rejects overflow even with individually finite inputs.
    pub fn global_rectangle(&self) -> Result<Rect, LayoutError> {
        let [x, y, w, h] = self.bounds.rectangle();
        Rect::new([
            finite(self.origin[0] + x)?,
            finite(self.origin[1] + y)?,
            w,
            h,
        ])
    }
}

/// Clamps size first, then position in local DIP coordinates. At least 40 DIP is visible
/// on each axis, limited by the available area. The top edge stays accessible. On areas
/// smaller than the minimum panel size, retain 120x80 and expose the entire available
/// sliver; fitting both minimum size and full panel visibility is then impossible.
pub fn clamp(rectangle: Rect, area: [f64; 2]) -> Result<Rect, LayoutError> {
    valid_area(area)?;
    let [x, y, width, height] = rectangle.0;
    let width = width.clamp(Bounds::MIN_WIDTH, area[0].max(Bounds::MIN_WIDTH));
    let height = height.clamp(Bounds::MIN_HEIGHT, area[1].max(Bounds::MIN_HEIGHT));
    let visible_x = 40_f64.min(area[0]);
    let visible_y = 40_f64.min(area[1]);
    let x = x.clamp(-width + visible_x, area[0] - visible_x);
    let y = y.clamp(0., area[1] - visible_y);
    Rect::new([x, y, width, height])
}

/// Derives schema-1 fallback fractions from monitor-local coordinates, bounded to 0..1.
/// Global negative monitor origins must be removed by the adapter before calling this.
pub fn fractions(rectangle: Rect, area: [f64; 2]) -> Result<[f64; 4], LayoutError> {
    valid_area(area)?;
    let [x, y, w, h] = rectangle.0;
    Ok([
        finite(x / area[0])?.clamp(0., 1.),
        finite(y / area[1])?.clamp(0., 1.),
        finite(w / area[0])?.clamp(0., 1.),
        finite(h / area[1])?.clamp(0., 1.),
    ])
}

/// Resolves only from the original snapshot. A unique exact key wins independently of
/// primary metadata. Missing/duplicate keys require exactly one primary monitor; no
/// enumeration-order/device-name guess is made. Fractions drive fallback, original
/// rectangle/reference dimensions drive proportional resolution changes. DPI-only
/// changes with unchanged DIP work area preserve logical size and position.
pub fn resolve(stored: &StoredLayout, monitors: &[Monitor]) -> Result<ResolvedLayout, LayoutError> {
    let mut matches = monitors
        .iter()
        .filter(|m| m.key == stored.bounds.monitor_id());
    let first = matches.next();
    let ambiguous = matches.next().is_some();
    let (target, mut reason) = if let Some(exact) = first.filter(|_| !ambiguous) {
        (exact, Resolution::Exact)
    } else {
        let mut primaries = monitors.iter().filter(|m| m.primary);
        let primary = primaries.next().ok_or(LayoutError::InvalidTopology)?;
        if primaries.next().is_some() {
            return Err(LayoutError::InvalidTopology);
        }
        (
            primary,
            if ambiguous {
                Resolution::AmbiguousMonitor
            } else {
                Resolution::MissingMonitor
            },
        )
    };
    let [ox, oy, width, height] = target.area.0;
    let area = [width, height];
    let fallback = matches!(
        reason,
        Resolution::MissingMonitor | Resolution::AmbiguousMonitor
    );
    let values = if fallback {
        let [x, y, w, h] = stored.bounds.fractions();
        [
            finite(x * width)?,
            finite(y * height)?,
            finite(w * width)?,
            finite(h * height)?,
        ]
    } else if area != stored.original_area {
        reason = Resolution::Rescaled;
        let [x, y, w, h] = stored.bounds.rectangle();
        let sx = finite(width / stored.original_area[0])?;
        let sy = finite(height / stored.original_area[1])?;
        [
            finite(x * sx)?,
            finite(y * sy)?,
            finite(w * sx)?,
            finite(h * sy)?,
        ]
    } else {
        stored.bounds.rectangle()
    };
    // Zero fallback fractions are valid schema data. Raise extents to domain minimum
    // before constructing Rect rather than constructing invalid intermediate Bounds.
    let desired = Rect::new([
        values[0],
        values[1],
        values[2].max(Bounds::MIN_WIDTH),
        values[3].max(Bounds::MIN_HEIGHT),
    ])?;
    let rectangle = clamp(desired, area)?;
    if reason == Resolution::Exact && rectangle != desired {
        reason = Resolution::Clamped;
    }
    let bounds = Bounds::new(
        target.key.clone(),
        rectangle.0,
        target.dpi,
        stored.bounds.fractions(),
    )
    .map_err(|_| LayoutError::InvalidGeometry)?;
    Ok(ResolvedLayout {
        bounds,
        origin: [ox, oy],
        reason,
    })
}

fn overlap(a: f64, extent_a: f64, b: f64, extent_b: f64) -> bool {
    a <= b + extent_b && b <= a + extent_a
}

fn candidate(best: &mut Option<f64>, delta: f64) {
    if delta.is_finite() && delta.abs() <= 8. && best.is_none_or(|old| delta.abs() < old.abs()) {
        *best = Some(delta);
    }
}

/// Pure movement snap, preserving size. Inclusive 8 DIP threshold; Alt bypasses snapping.
/// Neighbors must share a perpendicular span. Closest delta wins independently per axis;
/// ties retain work-area edge, then supplied neighbor order, then grid. Grid aligns the
/// local top-left to its nearest multiple. Call `clamp` after snapping before rendering;
/// this primitive deliberately does not clamp negative positions or enforce panel minima.
/// Rejects invalid area/grid and any non-finite arithmetic result, including grid overflow.
pub fn snap(
    rectangle: Rect,
    area: [f64; 2],
    neighbors: &[Rect],
    grid: Option<f64>,
    alt: bool,
) -> Result<Rect, LayoutError> {
    valid_area(area)?;
    if grid.is_some_and(|g| !g.is_finite() || g <= 0.) {
        return Err(LayoutError::InvalidGeometry);
    }
    if alt {
        return Ok(rectangle);
    }
    let [x, y, width, height] = rectangle.0;
    let mut dx = None;
    let mut dy = None;
    candidate(&mut dx, -x);
    candidate(&mut dx, finite(area[0] - finite(x + width)?)?);
    candidate(&mut dy, -y);
    candidate(&mut dy, finite(area[1] - finite(y + height)?)?);
    for neighbor in neighbors {
        let [nx, ny, nw, nh] = neighbor.0;
        if overlap(y, height, ny, nh) {
            for target in [
                nx,
                finite(nx + nw)?,
                finite(nx - width)?,
                finite(finite(nx + nw)? - width)?,
            ] {
                candidate(&mut dx, finite(target - x)?);
            }
        }
        if overlap(x, width, nx, nw) {
            for target in [
                ny,
                finite(ny + nh)?,
                finite(ny - height)?,
                finite(finite(ny + nh)? - height)?,
            ] {
                candidate(&mut dy, finite(target - y)?);
            }
        }
    }
    if let Some(grid) = grid {
        let gx = finite(finite(x / grid)?.round() * grid)?;
        let gy = finite(finite(y / grid)?.round() * grid)?;
        candidate(&mut dx, finite(gx - x)?);
        candidate(&mut dy, finite(gy - y)?);
    }
    Rect::new([
        finite(x + dx.unwrap_or(0.))?,
        finite(y + dy.unwrap_or(0.))?,
        width,
        height,
    ])
}
