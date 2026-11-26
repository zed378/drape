use super::{
    Class, DRAPE_KINDS, DROP_MODES, Document, ITEM_KINDS, LABEL_MODES, SORT_MODES, UPDATE_CHECKS,
    scan, time,
};
use drape_core::TimestampMillis;
use drape_core::domain::{
    Bounds, DomainConfig, DomainError, Drape, DrapeItem, DrapeState, Guid, ItemKind, Page, PathKey,
    Settings, Style,
};
use drape_core::ports::{PathKeys, PortError};
use serde_json::{Map, Number, Value};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;

enum Invalid {
    Corrupt,
    Port(PortError),
}

impl From<DomainError> for Invalid {
    fn from(_: DomainError) -> Self {
        Self::Corrupt
    }
}

type ReadResult<T> = Result<T, Invalid>;
type Object = Map<String, Value>;

pub(super) fn classify(
    bytes: &[u8],
    keys: &dyn PathKeys,
    fallback: &Page,
) -> Result<Class, PortError> {
    let found = match scan::scan(bytes) {
        Ok(found) => found,
        Err(_) => return Ok(Class::TooLarge),
    };
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let parsed = serde_json::from_slice::<Value>(bytes);
    if let Ok(value) = &parsed
        && let Some(version) = newer_schema(value)
    {
        return Ok(Class::Newer(version));
    }
    let value = match (parsed, found) {
        (Ok(value), scan::Scan::Clear) => value,
        (Ok(_), _) | (Err(_), scan::Scan::Opaque) => return Ok(Class::Unsupported(None)),
        (Err(_), _) => return Ok(Class::Corrupt),
    };
    let mut reader = Reader {
        keys,
        unsupported: false,
        identities: HashMap::new(),
        id_times: HashMap::new(),
        path_times: HashMap::new(),
    };
    match reader.project(&value, fallback) {
        Ok((config, repairs)) if !reader.unsupported => Ok(Class::Valid(Box::new(Document {
            value,
            config,
            repairs,
        }))),
        Ok((config, _)) => Ok(Class::Unsupported(Some(Box::new(config)))),
        Err(Invalid::Corrupt) => Ok(Class::Corrupt),
        Err(Invalid::Port(error)) => Err(error),
    }
}

fn positive_integer(number: &Number) -> Option<u64> {
    let text = number.to_string();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let value = text.parse::<u64>().unwrap_or(u64::MAX);
    (value > 0).then_some(value)
}

fn newer_schema(value: &Value) -> Option<u32> {
    value.as_object()?.iter().find_map(|(key, value)| {
        if !key.eq_ignore_ascii_case("schemaVersion") {
            return None;
        }
        let version = positive_integer(value.as_number()?)?;
        (version > 1).then(|| u32::try_from(version).unwrap_or(u32::MAX))
    })
}

fn field<'a>(object: &'a Object, name: &str) -> Option<&'a Value> {
    object.get(name).filter(|value| !value.is_null())
}

// A required property spelled differently still projects; the case check marks it unsupported.
fn required<'a>(object: &'a Object, name: &str) -> Option<&'a Value> {
    field(object, name).or_else(|| {
        object
            .iter()
            .find(|(key, value)| key.eq_ignore_ascii_case(name) && !value.is_null())
            .map(|(_, value)| value)
    })
}

fn needed<'a, T>(
    object: &'a Object,
    name: &str,
    convert: impl Fn(&'a Value) -> Option<T>,
) -> ReadResult<T> {
    required(object, name)
        .and_then(convert)
        .ok_or(Invalid::Corrupt)
}

fn optional<'a, T>(
    object: &'a Object,
    name: &str,
    convert: impl Fn(&'a Value) -> Option<T>,
) -> ReadResult<Option<T>> {
    field(object, name)
        .map(|value| convert(value).ok_or(Invalid::Corrupt))
        .transpose()
}

fn text<'a>(object: &'a Object, name: &str) -> ReadResult<Option<&'a str>> {
    optional(object, name, Value::as_str)
}

fn boolean(object: &Object, name: &str, default: bool) -> ReadResult<bool> {
    Ok(optional(object, name, Value::as_bool)?.unwrap_or(default))
}

fn array<'a>(object: &'a Object, name: &str) -> ReadResult<Option<&'a Vec<Value>>> {
    optional(object, name, Value::as_array)
}

fn guid(object: &Object, name: &str) -> ReadResult<Guid> {
    Ok(Guid::parse(needed(object, name, Value::as_str)?)?)
}

struct Reader<'a> {
    keys: &'a dyn PathKeys,
    unsupported: bool,
    identities: HashMap<Guid, PathKey>,
    id_times: HashMap<(Guid, i64), String>,
    path_times: HashMap<(PathKey, i64), String>,
}

impl Reader<'_> {
    fn object<'a>(&mut self, value: &'a Value, known: &[&str]) -> ReadResult<&'a Object> {
        let object = value.as_object().ok_or(Invalid::Corrupt)?;
        self.unsupported |= object.keys().any(|key| {
            known
                .iter()
                .any(|name| key.as_str() != *name && key.eq_ignore_ascii_case(name))
        });
        Ok(object)
    }

    fn name(&mut self, value: Option<&str>, placeholder: &str) -> String {
        let value = value.unwrap_or("");
        if value.trim().is_empty() {
            self.unsupported = true;
            placeholder.into()
        } else {
            value.into()
        }
    }

    fn number(
        &mut self,
        object: &Object,
        name: &str,
        default: f64,
        range: (f64, f64),
    ) -> ReadResult<f64> {
        Ok(match optional(object, name, Value::as_number)? {
            Some(number) => self.bounded(number, default, range),
            None => default,
        })
    }

    fn bounded(&mut self, number: &Number, default: f64, range: (f64, f64)) -> f64 {
        let Some(value) = number.as_f64().filter(|value| value.is_finite()) else {
            self.unsupported = true;
            return default;
        };
        let clamped = value.clamp(range.0, range.1);
        self.unsupported |= clamped != value;
        clamped
    }

    fn integer(&mut self, object: &Object, name: &str, min: i64, max: i64) -> ReadResult<i64> {
        let Some(number) = optional(object, name, Value::as_number)? else {
            return Ok(0);
        };
        if let Some(value) = number.as_i64() {
            let clamped = value.clamp(min, max);
            self.unsupported |= value != clamped;
            return Ok(clamped);
        }
        self.unsupported = true;
        let text = number.to_string();
        let digits = text.strip_prefix('-').unwrap_or(&text);
        if digits.bytes().all(|byte| byte.is_ascii_digit()) {
            Ok(if text.starts_with('-') { min } else { max })
        } else {
            Ok(0)
        }
    }

    fn enumeration<T: Copy>(
        &mut self,
        object: &Object,
        name: &str,
        table: &[(&str, T)],
        default: T,
    ) -> ReadResult<T> {
        let Some(value) = text(object, name)? else {
            return Ok(default);
        };
        if let Some((_, found)) = table
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(value))
        {
            Ok(*found)
        } else {
            self.unsupported = true;
            Ok(default)
        }
    }

    fn order(&mut self, object: &Object) -> ReadResult<usize> {
        let Some(number) = optional(object, "order", Value::as_number)? else {
            return Ok(0);
        };
        if let Some(value) = number.as_u64()
            && let Ok(value) = usize::try_from(value)
        {
            return Ok(value);
        }
        self.unsupported = true;
        let text = number.to_string();
        Ok(if text.bytes().all(|byte| byte.is_ascii_digit()) {
            usize::MAX
        } else {
            0
        })
    }

    fn project(
        &mut self,
        value: &Value,
        fallback: &Page,
    ) -> ReadResult<(DomainConfig, Vec<drape_core::domain::Repair>)> {
        let root = self.object(
            value,
            &["schemaVersion", "settings", "pages", "drapes", "rules"],
        )?;
        let version = required(root, "schemaVersion")
            .and_then(Value::as_number)
            .and_then(positive_integer);
        if version != Some(1) {
            return Err(Invalid::Corrupt);
        }
        let mut pages = Vec::new();
        let mut page_ids = HashSet::new();
        for value in needed(root, "pages", Value::as_array)? {
            let object = self.object(value, &["id", "name", "order"])?;
            let id = guid(object, "id")?;
            if !page_ids.insert(id) {
                return Err(Invalid::Corrupt);
            }
            let name = self.name(text(object, "name")?, "Page");
            let order = u32::try_from(self.integer(object, "order", 0, i64::from(u32::MAX))?)
                .map_err(|_| Invalid::Corrupt)?;
            pages.push(Page::new(id, name, order)?);
        }
        let mut drapes = Vec::new();
        let mut drape_ids = HashSet::new();
        if let Some(values) = array(root, "drapes")? {
            for value in values {
                let drape = self.drape(value)?;
                if !drape_ids.insert(drape.id()) {
                    return Err(Invalid::Corrupt);
                }
                drapes.push(drape);
            }
        }
        let settings = self.settings(field(root, "settings"))?;
        let (config, repairs) = DomainConfig::reconcile(pages, drapes, Some(fallback.clone()))?;
        Ok((config.with_settings(settings)?, repairs))
    }

    fn settings(&mut self, value: Option<&Value>) -> ReadResult<Settings> {
        let mut settings = Settings::default();
        let Some(value) = value else {
            return Ok(settings);
        };
        let object = self.object(
            value,
            &[
                "language",
                "theme",
                "hideIconsWhenActive",
                "quickHideEnabled",
                "quickHide",
                "autostart",
                "inboxEnabled",
                "dropMode",
                "logLevel",
                "updateCheck",
            ],
        )?;
        if let Some(value) = text(object, "language")? {
            settings.language = value.into();
        }
        if let Some(value) = text(object, "theme")? {
            settings.theme = value.into();
        }
        if let Some(value) = text(object, "logLevel")? {
            settings.log_level = value.into();
        }
        settings.hide_icons_when_active = boolean(
            object,
            "hideIconsWhenActive",
            settings.hide_icons_when_active,
        )?;
        settings.autostart = boolean(object, "autostart", settings.autostart)?;
        settings.inbox_enabled = boolean(object, "inboxEnabled", settings.inbox_enabled)?;
        settings.drop_mode =
            self.enumeration(object, "dropMode", DROP_MODES, settings.drop_mode)?;
        settings.update_check =
            self.enumeration(object, "updateCheck", UPDATE_CHECKS, settings.update_check)?;
        let flat = optional(object, "quickHideEnabled", Value::as_bool)?;
        let nested = if let Some(value) = field(object, "quickHide") {
            let quick_hide = self.object(value, &["enabled"])?;
            optional(quick_hide, "enabled", Value::as_bool)?
        } else {
            None
        };
        settings.quick_hide_enabled = match (flat, nested) {
            (Some(a), Some(b)) if a != b => {
                self.unsupported = true;
                settings.quick_hide_enabled
            }
            _ => flat.or(nested).unwrap_or(settings.quick_hide_enabled),
        };
        Ok(settings)
    }

    fn bounds(&mut self, value: &Value) -> ReadResult<Bounds> {
        let object = self.object(
            value,
            &[
                "monitorId",
                "x",
                "y",
                "width",
                "height",
                "dpi",
                "fx",
                "fy",
                "fw",
                "fh",
            ],
        )?;
        let monitor = needed(object, "monitorId", Value::as_str)?;
        let monitor = self.name(Some(monitor), "Monitor");
        let mut rectangle = [0.0, 0.0, Bounds::MIN_WIDTH, Bounds::MIN_HEIGHT];
        let ranges = [
            ("x", f64::MIN),
            ("y", f64::MIN),
            ("width", Bounds::MIN_WIDTH),
            ("height", Bounds::MIN_HEIGHT),
        ];
        for (slot, (name, minimum)) in rectangle.iter_mut().zip(ranges) {
            let number = needed(object, name, Value::as_number)?;
            *slot = self.bounded(number, *slot, (minimum, f64::MAX));
        }
        let dpi = self.number(object, "dpi", 96.0, (f64::from_bits(1), f64::MAX))?;
        let fractions = [
            self.number(object, "fx", 0.0, (0.0, 1.0))?,
            self.number(object, "fy", 0.0, (0.0, 1.0))?,
            self.number(object, "fw", 0.0, (0.0, 1.0))?,
            self.number(object, "fh", 0.0, (0.0, 1.0))?,
        ];
        Ok(Bounds::new(monitor, rectangle, dpi, fractions)?)
    }

    fn style(&mut self, value: Option<&Value>) -> ReadResult<Style> {
        let Some(value) = value else {
            return Ok(Style::default());
        };
        let object = self.object(
            value,
            &[
                "opacity",
                "cornerRadius",
                "titleSize",
                "iconSize",
                "iconSpacing",
                "tint",
                "blur",
                "titleFont",
                "labelMode",
                "theme",
            ],
        )?;
        let mut style = Style::new(
            self.number(object, "opacity", 0.45, (0.0, 1.0))?,
            self.number(object, "cornerRadius", 8.0, (0.0, f64::MAX))?,
            self.number(object, "titleSize", 13.0, (f64::from_bits(1), f64::MAX))?,
            self.number(object, "iconSize", 48.0, (16.0, 256.0))?,
            self.number(object, "iconSpacing", 12.0, (0.0, f64::MAX))?,
        )?;
        style.tint = text(object, "tint")?.map(str::to_owned);
        style.blur = text(object, "blur")?.map(str::to_owned);
        style.title_font = text(object, "titleFont")?.map(str::to_owned);
        style.theme = text(object, "theme")?.map(str::to_owned);
        style.label_mode = self.enumeration(object, "labelMode", LABEL_MODES, style.label_mode)?;
        Ok(style)
    }

    fn state(&mut self, value: Option<&Value>) -> ReadResult<DrapeState> {
        let mut state = DrapeState::default();
        let Some(value) = value else {
            return Ok(state);
        };
        let object = self.object(value, &["rolledUp", "locked", "sortMode", "zOrderHint"])?;
        state.rolled_up = boolean(object, "rolledUp", state.rolled_up)?;
        state.locked = boolean(object, "locked", state.locked)?;
        state.sort_mode = self.enumeration(object, "sortMode", SORT_MODES, state.sort_mode)?;
        state.z_order_hint = i32::try_from(self.integer(
            object,
            "zOrderHint",
            i64::from(i32::MIN),
            i64::from(i32::MAX),
        )?)
        .map_err(|_| Invalid::Corrupt)?;
        Ok(state)
    }

    fn drape(&mut self, value: &Value) -> ReadResult<Drape> {
        let object = self.object(
            value,
            &[
                "id",
                "pageId",
                "title",
                "kind",
                "bounds",
                "style",
                "state",
                "quickHideExcluded",
                "items",
            ],
        )?;
        let id = guid(object, "id")?;
        let page_id = guid(object, "pageId")?;
        let title = self.name(text(object, "title")?, "Drape");
        let bounds = self.bounds(required(object, "bounds").ok_or(Invalid::Corrupt)?)?;
        let mut items = Vec::new();
        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        if let Some(values) = array(object, "items")? {
            for value in values {
                if let Some((order, item)) = self.item(value)? {
                    let new_id = ids.insert(item.id());
                    let new_path = paths.insert(item.path_key().clone());
                    if new_id && new_path {
                        items.push((order, item));
                    } else {
                        self.unsupported = true;
                    }
                }
            }
        }
        items.sort_by_key(|(order, _)| *order);
        let mut drape = Drape::new(
            id,
            page_id,
            title,
            bounds,
            items.into_iter().map(|(_, item)| item).collect(),
        )?;
        drape.kind = self.enumeration(object, "kind", DRAPE_KINDS, drape.kind)?;
        drape.style = self.style(field(object, "style"))?;
        drape.state = self.state(field(object, "state"))?;
        drape.quick_hide_excluded = boolean(object, "quickHideExcluded", false)?;
        Ok(drape)
    }

    fn item(&mut self, value: &Value) -> ReadResult<Option<(usize, DrapeItem)>> {
        let object = self.object(
            value,
            &[
                "id",
                "kind",
                "path",
                "order",
                "addedAt",
                "missingSince",
                "pinnedByUser",
                "fileId",
                "displayName",
            ],
        )?;
        let id = guid(object, "id")?;
        let path = OsString::from(needed(object, "path", Value::as_str)?);
        let kind = self.enumeration(object, "kind", ITEM_KINDS, ItemKind::File)?;
        let order = self.order(object)?;
        let added = text(object, "addedAt")?.and_then(time::parse);
        self.unsupported |= added.is_none();
        let added = added.unwrap_or(time::Instant {
            millis: TimestampMillis(0),
            sub_millis: String::new(),
        });
        let missing_text = text(object, "missingSince")?;
        let missing = missing_text.and_then(time::parse);
        self.unsupported |= missing_text.is_some() && missing.is_none();
        let file_id = text(object, "fileId")?.map(str::to_owned);
        let display_name = text(object, "displayName")?.map(str::to_owned);
        let pinned = boolean(object, "pinnedByUser", false)?;
        if path.is_empty() {
            self.unsupported = true;
            return Ok(None);
        }
        let key = self.keys.path_key(&path).map_err(Invalid::Port)?;
        if let Some(previous) = self.identities.insert(id, key.clone())
            && previous != key
        {
            return Err(Invalid::Corrupt);
        }
        let previous_id = self
            .id_times
            .insert((id, added.millis.0), added.sub_millis.clone());
        let previous_path = self
            .path_times
            .insert((key.clone(), added.millis.0), added.sub_millis.clone());
        self.unsupported |= previous_id
            .iter()
            .chain(previous_path.iter())
            .any(|previous| previous != &added.sub_millis);
        let mut item = DrapeItem::new(id, kind, path, key, added.millis)?;
        if let Some(missing) = missing {
            item.mark_missing(missing.millis);
        }
        item.file_id = file_id;
        item.display_name = display_name;
        item.pinned_by_user = pinned;
        Ok(Some((order, item)))
    }
}
