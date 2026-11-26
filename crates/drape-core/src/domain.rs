//! Validated, owned domain data. Wire parsing and filesystem policy belong to adapters.

use crate::TimestampMillis;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fmt;

/// A validation failure; diagnostics deliberately contain no user paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    Invalid(&'static str),
    Duplicate(&'static str),
    UnknownPage,
    ReadOnly,
    NotFound,
    LimitExceeded,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(field) => write!(f, "invalid {field}"),
            Self::Duplicate(field) => write!(f, "duplicate {field}"),
            Self::UnknownPage => f.write_str("unknown page"),
            Self::ReadOnly => f.write_str("configuration is read-only"),
            Self::NotFound => f.write_str("entity not found"),
            Self::LimitExceeded => f.write_str("resource limit exceeded"),
        }
    }
}

impl std::error::Error for DomainError {}

/// Non-nil GUID supplied by the caller; Core never generates random identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Guid([u8; 16]);

impl Guid {
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, DomainError> {
        if bytes == [0; 16] {
            return Err(DomainError::Invalid("id"));
        }
        Ok(Self(bytes))
    }

    /// Accept hyphenated GUID syntax, without restricting UUID version or letter case.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        let input = value.as_bytes();
        if input.len() != 36 {
            return Err(DomainError::Invalid("id"));
        }
        let mut bytes = [0; 16];
        let mut output = 0;
        let mut high = None;
        for (index, byte) in input.iter().copied().enumerate() {
            if [8, 13, 18, 23].contains(&index) {
                if byte != b'-' {
                    return Err(DomainError::Invalid("id"));
                }
                continue;
            }
            let digit = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return Err(DomainError::Invalid("id")),
            };
            if let Some(first) = high.take() {
                bytes[output] = first * 16 + digit;
                output += 1;
            } else {
                high = Some(digit);
            }
        }
        Self::from_bytes(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, byte) in self.0.iter().enumerate() {
            if [4, 6, 8, 10].contains(&index) {
                f.write_str("-")?;
            }
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Opaque comparison key supplied by the filesystem adapter, never case-folded in Core.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathKey(String);

impl PathKey {
    pub fn new(value: String) -> Result<Self, DomainError> {
        if value.is_empty() {
            return Err(DomainError::Invalid("path key"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn nonblank(value: &str, field: &'static str) -> Result<(), DomainError> {
    if value.trim().is_empty() {
        Err(DomainError::Invalid(field))
    } else {
        Ok(())
    }
}

/// Stored logical rectangle and monitor fallback fractions. No scaling/clamping occurs here.
#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    monitor_id: String,
    rectangle: [f64; 4],
    dpi: f64,
    fractions: [f64; 4],
}

impl Bounds {
    pub const MIN_WIDTH: f64 = 120.0;
    pub const MIN_HEIGHT: f64 = 80.0;

    /// Rectangle is x/y/width/height in DIP; fractions are fx/fy/fw/fh.
    pub fn new(
        monitor_id: String,
        rectangle: [f64; 4],
        dpi: f64,
        fractions: [f64; 4],
    ) -> Result<Self, DomainError> {
        nonblank(&monitor_id, "monitor id")?;
        if rectangle.iter().any(|n| !n.is_finite())
            || rectangle[2] < Self::MIN_WIDTH
            || rectangle[3] < Self::MIN_HEIGHT
            || !dpi.is_finite()
            || dpi <= 0.0
            || fractions
                .iter()
                .any(|n| !n.is_finite() || !(0.0..=1.0).contains(n))
        {
            return Err(DomainError::Invalid("bounds"));
        }
        Ok(Self {
            monitor_id,
            rectangle,
            dpi,
            fractions,
        })
    }

    pub fn monitor_id(&self) -> &str {
        &self.monitor_id
    }
    pub fn rectangle(&self) -> [f64; 4] {
        self.rectangle
    }
    pub fn dpi(&self) -> f64 {
        self.dpi
    }
    pub fn fractions(&self) -> [f64; 4] {
        self.fractions
    }
    pub fn with_position(&self, x: f64, y: f64) -> Result<Self, DomainError> {
        Self::new(
            self.monitor_id.clone(),
            [x, y, self.rectangle[2], self.rectangle[3]],
            self.dpi,
            self.fractions,
        )
    }
    pub fn with_size(&self, width: f64, height: f64) -> Result<Self, DomainError> {
        Self::new(
            self.monitor_id.clone(),
            [self.rectangle[0], self.rectangle[1], width, height],
            self.dpi,
            self.fractions,
        )
    }
    pub fn with_dpi(&self, dpi: f64) -> Result<Self, DomainError> {
        Self::new(self.monitor_id.clone(), self.rectangle, dpi, self.fractions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    id: Guid,
    name: String,
    order: u32,
}

impl Page {
    pub fn new(id: Guid, name: String, order: u32) -> Result<Self, DomainError> {
        nonblank(&name, "page name")?;
        Ok(Self { id, name, order })
    }
    pub fn id(&self) -> Guid {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn order(&self) -> u32 {
        self.order
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    File,
    Folder,
    Shortcut,
}

/// A literal lossless path reference; removing this value never touches a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrapeItem {
    id: Guid,
    kind: ItemKind,
    path: OsString,
    path_key: PathKey,
    added_at: TimestampMillis,
    missing_since: Option<TimestampMillis>,
    order: usize,
    pub file_id: Option<String>,
    pub display_name: Option<String>,
    pub pinned_by_user: bool,
}

impl DrapeItem {
    pub fn new(
        id: Guid,
        kind: ItemKind,
        path: OsString,
        path_key: PathKey,
        added_at: TimestampMillis,
    ) -> Result<Self, DomainError> {
        if path.is_empty() {
            return Err(DomainError::Invalid("path"));
        }
        Ok(Self {
            id,
            kind,
            path,
            path_key,
            added_at,
            missing_since: None,
            order: 0,
            file_id: None,
            display_name: None,
            pinned_by_user: false,
        })
    }
    pub fn id(&self) -> Guid {
        self.id
    }
    pub fn kind(&self) -> ItemKind {
        self.kind
    }
    pub fn path(&self) -> &OsStr {
        &self.path
    }
    pub fn path_key(&self) -> &PathKey {
        &self.path_key
    }
    pub fn added_at(&self) -> TimestampMillis {
        self.added_at
    }
    pub fn missing_since(&self) -> Option<TimestampMillis> {
        self.missing_since
    }
    pub fn order(&self) -> usize {
        self.order
    }
    /// Caller supplies observation time; repeated observations retain the first missing time.
    pub fn mark_missing(&mut self, observed: TimestampMillis) {
        self.missing_since.get_or_insert(observed);
    }
    pub fn mark_present(&mut self) {
        self.missing_since = None;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DrapeKind {
    #[default]
    Normal,
    Inbox,
    Portal,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LabelMode {
    None,
    OneLine,
    #[default]
    TwoLines,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    Manual,
    Name,
    Type,
    DateAdded,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    opacity: f64,
    corner_radius: f64,
    title_size: f64,
    icon_size: f64,
    icon_spacing: f64,
    pub tint: Option<String>,
    pub blur: Option<String>,
    pub title_font: Option<String>,
    pub label_mode: LabelMode,
    pub theme: Option<String>,
}

impl Style {
    pub fn new(
        opacity: f64,
        corner_radius: f64,
        title_size: f64,
        icon_size: f64,
        icon_spacing: f64,
    ) -> Result<Self, DomainError> {
        if [opacity, corner_radius, title_size, icon_size, icon_spacing]
            .iter()
            .any(|n| !n.is_finite())
            || !(0.0..=1.0).contains(&opacity)
            || corner_radius < 0.0
            || title_size <= 0.0
            || !(16.0..=256.0).contains(&icon_size)
            || icon_spacing < 0.0
        {
            return Err(DomainError::Invalid("style"));
        }
        Ok(Self {
            opacity,
            corner_radius,
            title_size,
            icon_size,
            icon_spacing,
            tint: Some("#3A6EA5".into()),
            blur: Some("acrylic".into()),
            title_font: None,
            label_mode: LabelMode::TwoLines,
            theme: None,
        })
    }
    pub fn opacity(&self) -> f64 {
        self.opacity
    }
    pub fn corner_radius(&self) -> f64 {
        self.corner_radius
    }
    pub fn title_size(&self) -> f64 {
        self.title_size
    }
    pub fn icon_size(&self) -> f64 {
        self.icon_size
    }
    pub fn icon_spacing(&self) -> f64 {
        self.icon_spacing
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            opacity: 0.45,
            corner_radius: 8.0,
            title_size: 13.0,
            icon_size: 48.0,
            icon_spacing: 12.0,
            tint: Some("#3A6EA5".into()),
            blur: Some("acrylic".into()),
            title_font: None,
            label_mode: LabelMode::TwoLines,
            theme: None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DrapeState {
    pub rolled_up: bool,
    pub locked: bool,
    pub sort_mode: SortMode,
    pub z_order_hint: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheck {
    #[default]
    Off,
    Manual,
    Periodic,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DropMode {
    #[default]
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub hide_icons_when_active: bool,
    pub quick_hide_enabled: bool,
    pub autostart: bool,
    pub inbox_enabled: bool,
    pub drop_mode: DropMode,
    pub log_level: String,
    pub update_check: UpdateCheck,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en-US".into(),
            theme: "system".into(),
            hide_icons_when_active: true,
            quick_hide_enabled: true,
            autostart: false,
            inbox_enabled: true,
            drop_mode: DropMode::Reference,
            log_level: "Information".into(),
            update_check: UpdateCheck::Off,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Drape {
    id: Guid,
    page_id: Guid,
    title: String,
    bounds: Bounds,
    items: Vec<DrapeItem>,
    pub kind: DrapeKind,
    pub style: Style,
    pub state: DrapeState,
    pub quick_hide_excluded: bool,
}

impl Drape {
    pub fn new(
        id: Guid,
        page_id: Guid,
        title: String,
        bounds: Bounds,
        mut items: Vec<DrapeItem>,
    ) -> Result<Self, DomainError> {
        nonblank(&title, "title")?;
        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        for (order, item) in items.iter_mut().enumerate() {
            if !ids.insert(item.id) {
                return Err(DomainError::Duplicate("item id"));
            }
            if !paths.insert(item.path_key.clone()) {
                return Err(DomainError::Duplicate("membership"));
            }
            item.order = order;
        }
        Ok(Self {
            id,
            page_id,
            title,
            bounds,
            items,
            kind: DrapeKind::Normal,
            style: Style::default(),
            state: DrapeState::default(),
            quick_hide_excluded: false,
        })
    }
    pub fn id(&self) -> Guid {
        self.id
    }
    pub fn page_id(&self) -> Guid {
        self.page_id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn bounds(&self) -> &Bounds {
        &self.bounds
    }
    pub fn items(&self) -> &[DrapeItem] {
        &self.items
    }
    pub fn rename(&self, title: String) -> Result<Self, DomainError> {
        nonblank(&title, "title")?;
        Ok(Self {
            title,
            ..self.clone()
        })
    }
    pub fn with_bounds(&self, bounds: Bounds) -> Self {
        Self {
            bounds,
            ..self.clone()
        }
    }
    pub fn add_item(&self, mut item: DrapeItem) -> Result<Self, DomainError> {
        if self.items.iter().any(|existing| existing.id == item.id) {
            return Err(DomainError::Duplicate("item id"));
        }
        if self
            .items
            .iter()
            .any(|existing| existing.path_key == item.path_key)
        {
            return Err(DomainError::Duplicate("membership"));
        }
        let mut updated = self.clone();
        item.order = updated.items.len();
        updated.items.push(item);
        Ok(updated)
    }
    pub fn remove_item(&self, id: Guid) -> Self {
        let mut updated = self.clone();
        updated.items.retain(|item| item.id != id);
        updated.renumber();
        updated
    }
    fn renumber(&mut self) {
        for (order, item) in self.items.iter_mut().enumerate() {
            item.order = order;
        }
    }
}

/// Immutable position, style and membership for one Drape in a saved layout.
///
/// The original bounds and item order are retained unchanged. Monitor resolution,
/// file existence checks and restoring references belong to consuming services.
#[derive(Debug, Clone, PartialEq)]
pub struct DrapeSnapshot {
    drape_id: Guid,
    bounds: Bounds,
    style: Style,
    items: Vec<DrapeItem>,
}

impl DrapeSnapshot {
    /// Rejects duplicate item identities or adapter-supplied path keys.
    pub fn new(
        drape_id: Guid,
        bounds: Bounds,
        style: Style,
        items: Vec<DrapeItem>,
    ) -> Result<Self, DomainError> {
        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        for item in &items {
            if !ids.insert(item.id) {
                return Err(DomainError::Duplicate("item id"));
            }
            if !paths.insert(&item.path_key) {
                return Err(DomainError::Duplicate("membership"));
            }
        }
        Ok(Self {
            drape_id,
            bounds,
            style,
            items,
        })
    }

    pub fn drape_id(&self) -> Guid {
        self.drape_id
    }
    pub fn bounds(&self) -> &Bounds {
        &self.bounds
    }
    pub fn style(&self) -> &Style {
        &self.style
    }
    pub fn items(&self) -> &[DrapeItem] {
        &self.items
    }
}

/// Validated passive layout value with caller-supplied identity and creation time.
///
/// This is not a wire document or a restore operation. Saving, previewing and
/// applying a snapshot must preserve this original value and perform their own
/// validation and user confirmation at the appropriate boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutSnapshot {
    id: Guid,
    name: String,
    created_at: TimestampMillis,
    drapes: Vec<DrapeSnapshot>,
}

impl LayoutSnapshot {
    /// Rejects blank names and duplicate Drape/item identities or memberships.
    /// An empty saved layout is valid; no default Drape or timestamp is generated.
    pub fn new(
        id: Guid,
        name: String,
        created_at: TimestampMillis,
        drapes: Vec<DrapeSnapshot>,
    ) -> Result<Self, DomainError> {
        nonblank(&name, "snapshot name")?;
        let mut drape_ids = HashSet::new();
        let mut item_ids = HashSet::new();
        let mut paths = HashSet::new();
        for drape in &drapes {
            if !drape_ids.insert(drape.drape_id) {
                return Err(DomainError::Duplicate("Drape id"));
            }
            for item in &drape.items {
                if !item_ids.insert(item.id) {
                    return Err(DomainError::Duplicate("item id"));
                }
                if !paths.insert(&item.path_key) {
                    return Err(DomainError::Duplicate("membership"));
                }
            }
        }
        Ok(Self {
            id,
            name,
            created_at,
            drapes,
        })
    }

    pub fn id(&self) -> Guid {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn created_at(&self) -> TimestampMillis {
        self.created_at
    }
    pub fn drapes(&self) -> &[DrapeSnapshot] {
        &self.drapes
    }
}

/// Report every reference-only repair; callers retain original input/backup and notify users.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repair {
    ReassignedPage {
        drape: Guid,
        previous_page: Guid,
        page: Guid,
    },
    RemovedDuplicateMembership {
        drape: Guid,
        item: Guid,
        kept_by: Guid,
    },
}

/// Validated in-memory configuration, not a lossless JSON document.
///
/// R1-02 must retain unknown wire values/fields separately before round-tripping.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainConfig {
    schema_version: u32,
    pages: Vec<Page>,
    drapes: Vec<Drape>,
    settings: Settings,
}

impl DomainConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
    pub fn new(
        schema_version: u32,
        pages: Vec<Page>,
        drapes: Vec<Drape>,
    ) -> Result<Self, DomainError> {
        if schema_version == 0 {
            return Err(DomainError::Invalid("schema version"));
        }
        if pages.is_empty() {
            return Err(DomainError::Invalid("pages"));
        }
        let mut page_ids = HashSet::new();
        for page in &pages {
            if !page_ids.insert(page.id) {
                return Err(DomainError::Duplicate("page id"));
            }
        }
        let mut drape_ids = HashSet::new();
        let mut item_ids = HashSet::new();
        let mut paths = HashSet::new();
        for drape in &drapes {
            if !drape_ids.insert(drape.id) {
                return Err(DomainError::Duplicate("Drape id"));
            }
            if !page_ids.contains(&drape.page_id) {
                return Err(DomainError::UnknownPage);
            }
            for item in &drape.items {
                if !item_ids.insert(item.id) {
                    return Err(DomainError::Duplicate("item id"));
                }
                if !paths.insert(item.path_key.clone()) {
                    return Err(DomainError::Duplicate("membership"));
                }
            }
        }
        Ok(Self {
            schema_version,
            pages,
            drapes,
            settings: Settings::default(),
        })
    }
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }
    pub fn drapes(&self) -> &[Drape] {
        &self.drapes
    }
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
    pub fn is_read_only(&self) -> bool {
        self.schema_version > Self::CURRENT_SCHEMA_VERSION
    }
    pub fn with_settings(&self, settings: Settings) -> Result<Self, DomainError> {
        if self.is_read_only() {
            return Err(DomainError::ReadOnly);
        }
        Ok(Self {
            settings,
            ..self.clone()
        })
    }
    pub fn add_drape(&self, drape: Drape) -> Result<Self, DomainError> {
        if self.is_read_only() {
            return Err(DomainError::ReadOnly);
        }
        let mut drapes = self.drapes.clone();
        drapes.push(drape);
        let mut updated = Self::new(self.schema_version, self.pages.clone(), drapes)?;
        updated.settings = self.settings.clone();
        Ok(updated)
    }
    pub fn replace_drape(&self, drape: Drape) -> Result<Self, DomainError> {
        if self.is_read_only() {
            return Err(DomainError::ReadOnly);
        }
        let index = self
            .drapes
            .iter()
            .position(|existing| existing.id == drape.id)
            .ok_or(DomainError::NotFound)?;
        let mut drapes = self.drapes.clone();
        drapes[index] = drape;
        let mut updated = Self::new(self.schema_version, self.pages.clone(), drapes)?;
        updated.settings = self.settings.clone();
        Ok(updated)
    }
    /// Removes references only; no file operation can occur in Core.
    pub fn remove_drape(&self, id: Guid) -> Result<Self, DomainError> {
        if self.is_read_only() {
            return Err(DomainError::ReadOnly);
        }
        let mut updated = self.clone();
        updated.drapes.retain(|drape| drape.id != id);
        Ok(updated)
    }

    /// Explicit schema-1 repair. No new IDs/times are synthesized.
    ///
    /// Membership winners: newest supplied timestamp, then persisted Drape vector
    /// order, then GUID. Duplicate page/Drape IDs remain errors, never repairs.
    pub fn reconcile(
        mut pages: Vec<Page>,
        mut drapes: Vec<Drape>,
        fallback: Option<Page>,
    ) -> Result<(Self, Vec<Repair>), DomainError> {
        if pages.is_empty() {
            pages.push(fallback.ok_or(DomainError::Invalid("pages"))?);
        }
        let page_ids: HashSet<_> = pages.iter().map(|page| page.id).collect();
        if page_ids.len() != pages.len() {
            return Err(DomainError::Duplicate("page id"));
        }
        let drape_ids: HashSet<_> = drapes.iter().map(|drape| drape.id).collect();
        if drape_ids.len() != drapes.len() {
            return Err(DomainError::Duplicate("Drape id"));
        }
        // A corrupt identity is not evidence that two different resources match.
        let mut identity_keys = std::collections::HashMap::new();
        for item in drapes.iter().flat_map(|drape| &drape.items) {
            if let Some(previous) = identity_keys.insert(item.id, &item.path_key)
                && previous != &item.path_key
            {
                return Err(DomainError::Duplicate("item id"));
            }
        }
        let mut repairs = Vec::new();
        for drape in &mut drapes {
            if !page_ids.contains(&drape.page_id) {
                repairs.push(Repair::ReassignedPage {
                    drape: drape.id,
                    previous_page: drape.page_id,
                    page: pages[0].id,
                });
                drape.page_id = pages[0].id;
            }
        }
        let mut candidates: Vec<_> = drapes
            .iter()
            .enumerate()
            .flat_map(|(d, drape)| {
                drape
                    .items
                    .iter()
                    .enumerate()
                    .map(move |(i, item)| (d, i, item.added_at, drape.id))
            })
            .collect();
        candidates.sort_by(|a, b| {
            b.2.cmp(&a.2)
                .then(a.0.cmp(&b.0))
                .then(a.3.cmp(&b.3))
                .then(a.1.cmp(&b.1))
        });
        let mut ids = std::collections::HashMap::new();
        let mut paths = std::collections::HashMap::new();
        let mut keep = HashSet::new();
        for (d, i, _, _) in candidates {
            let drape = &drapes[d];
            let item = &drape.items[i];
            let owner = ids.get(&item.id).or_else(|| paths.get(&item.path_key));
            if let Some(kept_by) = owner {
                repairs.push(Repair::RemovedDuplicateMembership {
                    drape: drape.id,
                    item: item.id,
                    kept_by: *kept_by,
                });
            } else {
                ids.insert(item.id, drape.id);
                paths.insert(item.path_key.clone(), drape.id);
                keep.insert((d, i));
            }
        }
        for (d, drape) in drapes.iter_mut().enumerate() {
            let mut index = 0;
            drape.items.retain(|_| {
                let retain = keep.contains(&(d, index));
                index += 1;
                retain
            });
            drape.renumber();
        }
        Ok((
            Self::new(Self::CURRENT_SCHEMA_VERSION, pages, drapes)?,
            repairs,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8,
    Bgra8,
}

/// Owned top-down pixels; alpha is straight (not premultiplied). Row padding is allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconBitmap {
    width: u32,
    height: u32,
    stride: u32,
    format: PixelFormat,
    pixels: Vec<u8>,
}

impl IconBitmap {
    /// Validate expected byte size before an adapter allocates a decode buffer.
    pub fn required_bytes(
        width: u32,
        height: u32,
        stride: u32,
        byte_limit: usize,
    ) -> Result<usize, DomainError> {
        let row = width.checked_mul(4).ok_or(DomainError::LimitExceeded)?;
        if width == 0 || height == 0 || stride < row {
            return Err(DomainError::Invalid("bitmap dimensions or stride"));
        }
        let bytes = usize::try_from(stride)
            .ok()
            .and_then(|s| usize::try_from(height).ok().and_then(|h| s.checked_mul(h)))
            .ok_or(DomainError::LimitExceeded)?;
        if bytes > byte_limit {
            return Err(DomainError::LimitExceeded);
        }
        Ok(bytes)
    }
    pub fn new(
        width: u32,
        height: u32,
        stride: u32,
        format: PixelFormat,
        pixels: Vec<u8>,
        byte_limit: usize,
    ) -> Result<Self, DomainError> {
        if pixels.len() != Self::required_bytes(width, height, stride, byte_limit)? {
            return Err(DomainError::Invalid("bitmap buffer"));
        }
        Ok(Self {
            width,
            height,
            stride,
            format,
            pixels,
        })
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn stride(&self) -> u32 {
        self.stride
    }
    pub fn format(&self) -> PixelFormat {
        self.format
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}
