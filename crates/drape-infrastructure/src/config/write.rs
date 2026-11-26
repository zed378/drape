use std::collections::HashMap;

use drape_core::TimestampMillis;
use drape_core::domain::{
    Bounds, DomainConfig, Drape, DrapeItem, DrapeKind, DrapeState, Guid, ItemKind, LabelMode, Page,
    Settings, SortMode, Style,
};
use serde_json::{Map, Number, Value};

use super::{DRAPE_KINDS, DROP_MODES, ITEM_KINDS, LABEL_MODES, SORT_MODES, UPDATE_CHECKS, time};

pub(super) struct Unrepresentable;

type Object = Map<String, Value>;

pub(super) fn merge(base: &Value, config: &DomainConfig) -> Result<Value, Unrepresentable> {
    let mut root = object(Some(base));
    if !root.contains_key("schemaVersion") {
        root.insert("schemaVersion".into(), config.schema_version().into());
    }
    merge_settings(&mut root, config.settings())?;
    let pages = identities(array(base.get("pages")));
    let new_pages = config
        .pages()
        .iter()
        .map(|page| merge_page(pages.get(&page.id()).copied(), page))
        .collect::<Result<Vec<_>, _>>()?;
    root.insert("pages".into(), Value::Array(new_pages));

    let stored_drapes = array(base.get("drapes"));
    let drapes = identities(stored_drapes);
    let mut all_items = HashMap::new();
    for drape in stored_drapes {
        for item in array(drape.get("items")) {
            if let Some(id) = identity(item) {
                all_items.entry(id).or_insert(item);
            }
        }
    }
    if root.contains_key("drapes") || !config.drapes().is_empty() {
        let new_drapes = config
            .drapes()
            .iter()
            .map(|drape| merge_drape(drapes.get(&drape.id()).copied(), drape, &all_items))
            .collect::<Result<Vec<_>, _>>()?;
        root.insert("drapes".into(), Value::Array(new_drapes));
    }
    Ok(Value::Object(root))
}

fn object(base: Option<&Value>) -> Object {
    base.and_then(Value::as_object).cloned().unwrap_or_default()
}

fn array(value: Option<&Value>) -> &[Value] {
    value.and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn identity(value: &Value) -> Option<Guid> {
    value
        .get("id")?
        .as_str()
        .and_then(|id| Guid::parse(id).ok())
}

fn identities(values: &[Value]) -> HashMap<Guid, &Value> {
    let mut result = HashMap::new();
    for value in values {
        if let Some(id) = identity(value) {
            result.entry(id).or_insert(value);
        }
    }
    result
}

fn is_new(base: Option<&Value>) -> bool {
    base.and_then(Value::as_object).is_none()
}

fn string(fields: &mut Object, key: &str, value: &str, default: &str, new: bool) {
    if new || fields.get(key).and_then(Value::as_str).unwrap_or(default) != value {
        fields.insert(key.into(), value.into());
    }
}

fn optional_string(fields: &mut Object, key: &str, value: Option<&str>, new: bool) {
    if fields.get(key).and_then(Value::as_str) != value || new {
        if let Some(value) = value {
            fields.insert(key.into(), value.into());
        } else {
            fields.shift_remove(key);
        }
    }
}

fn boolean(fields: &mut Object, key: &str, value: bool, default: bool, new: bool) {
    if new || fields.get(key).and_then(Value::as_bool).unwrap_or(default) != value {
        fields.insert(key.into(), value.into());
    }
}

fn number(
    fields: &mut Object,
    key: &str,
    value: f64,
    default: f64,
    new: bool,
) -> Result<(), Unrepresentable> {
    if new || fields.get(key).and_then(Value::as_f64).unwrap_or(default) != value {
        let number = Number::from_f64(value).ok_or(Unrepresentable)?;
        fields.insert(key.into(), Value::Number(number));
    }
    Ok(())
}

// The reader rejects a fraction here, so integers must not be written through f64.
fn integer(fields: &mut Object, key: &str, value: i64, new: bool) {
    if new || fields.get(key).and_then(Value::as_i64).unwrap_or(0) != value {
        fields.insert(key.into(), value.into());
    }
}

fn guid(fields: &mut Object, key: &str, value: Guid, new: bool) {
    let stored = fields
        .get(key)
        .and_then(Value::as_str)
        .and_then(|text| Guid::parse(text).ok());
    if new || stored != Some(value) {
        fields.insert(key.into(), value.to_string().into());
    }
}

fn enumeration<T: Copy + PartialEq>(
    fields: &mut Object,
    key: &str,
    value: T,
    default: T,
    table: &[(&str, T)],
    new: bool,
) -> Result<(), Unrepresentable> {
    let stored = fields.get(key).and_then(Value::as_str).and_then(|text| {
        table
            .iter()
            .find(|(name, _)| text.eq_ignore_ascii_case(name))
            .map(|(_, value)| *value)
    });
    if new || stored.unwrap_or(default) != value {
        let (name, _) = table
            .iter()
            .find(|(_, variant)| *variant == value)
            .ok_or(Unrepresentable)?;
        fields.insert(key.into(), (*name).into());
    }
    Ok(())
}

fn timestamp(
    fields: &mut Object,
    key: &str,
    value: Option<TimestampMillis>,
    new: bool,
) -> Result<(), Unrepresentable> {
    let stored = fields
        .get(key)
        .and_then(Value::as_str)
        .and_then(time::parse)
        .map(|instant| instant.millis);
    if let Some(value) = value {
        if new || stored != Some(value) {
            fields.insert(
                key.into(),
                time::format(value).ok_or(Unrepresentable)?.into(),
            );
        }
    } else if stored.is_some() {
        fields.shift_remove(key);
    }
    Ok(())
}

fn merge_page(base: Option<&Value>, page: &Page) -> Result<Value, Unrepresentable> {
    let mut fields = object(base);
    let new = is_new(base);
    guid(&mut fields, "id", page.id(), new);
    string(&mut fields, "name", page.name(), "", new);
    integer(&mut fields, "order", i64::from(page.order()), new);
    Ok(Value::Object(fields))
}

fn merge_drape(
    base: Option<&Value>,
    drape: &Drape,
    all_items: &HashMap<Guid, &Value>,
) -> Result<Value, Unrepresentable> {
    let mut fields = object(base);
    let new = is_new(base);
    guid(&mut fields, "id", drape.id(), new);
    guid(&mut fields, "pageId", drape.page_id(), new);
    string(&mut fields, "title", drape.title(), "", new);
    enumeration(
        &mut fields,
        "kind",
        drape.kind,
        DrapeKind::Normal,
        DRAPE_KINDS,
        new,
    )?;
    merge_bounds(&mut fields, drape.bounds())?;
    merge_style(&mut fields, &drape.style)?;
    merge_state(&mut fields, &drape.state)?;
    boolean(
        &mut fields,
        "quickHideExcluded",
        drape.quick_hide_excluded,
        false,
        new,
    );
    if fields.contains_key("items") || !drape.items().is_empty() {
        let local_items = identities(array(base.and_then(|value| value.get("items"))));
        let items = drape
            .items()
            .iter()
            .map(|item| {
                let stored = local_items
                    .get(&item.id())
                    .or_else(|| all_items.get(&item.id()));
                merge_item(stored.copied(), item)
            })
            .collect::<Result<Vec<_>, _>>()?;
        fields.insert("items".into(), Value::Array(items));
    }
    Ok(Value::Object(fields))
}

fn merge_item(base: Option<&Value>, item: &DrapeItem) -> Result<Value, Unrepresentable> {
    let mut fields = object(base);
    let new = is_new(base);
    guid(&mut fields, "id", item.id(), new);
    enumeration(
        &mut fields,
        "kind",
        item.kind(),
        ItemKind::File,
        ITEM_KINDS,
        new,
    )?;
    string(
        &mut fields,
        "path",
        item.path().to_str().ok_or(Unrepresentable)?,
        "",
        new,
    );
    optional_string(&mut fields, "fileId", item.file_id.as_deref(), new);
    optional_string(
        &mut fields,
        "displayName",
        item.display_name.as_deref(),
        new,
    );
    let order = i64::try_from(item.order()).map_err(|_| Unrepresentable)?;
    integer(&mut fields, "order", order, new);
    timestamp(&mut fields, "addedAt", Some(item.added_at()), new)?;
    timestamp(&mut fields, "missingSince", item.missing_since(), new)?;
    boolean(&mut fields, "pinnedByUser", item.pinned_by_user, false, new);
    Ok(Value::Object(fields))
}

fn merge_bounds(parent: &mut Object, bounds: &Bounds) -> Result<(), Unrepresentable> {
    let base = parent.get("bounds");
    let new = is_new(base);
    let mut fields = object(base);
    string(&mut fields, "monitorId", bounds.monitor_id(), "", new);
    for (key, value) in ["x", "y", "width", "height"]
        .into_iter()
        .zip(bounds.rectangle())
    {
        number(&mut fields, key, value, 0.0, new)?;
    }
    number(&mut fields, "dpi", bounds.dpi(), 96.0, new)?;
    for (key, value) in ["fx", "fy", "fw", "fh"].into_iter().zip(bounds.fractions()) {
        number(&mut fields, key, value, 0.0, new)?;
    }
    parent.insert("bounds".into(), Value::Object(fields));
    Ok(())
}

fn merge_style(parent: &mut Object, style: &Style) -> Result<(), Unrepresentable> {
    let base = parent.get("style");
    let new = is_new(base);
    if new && *style == Style::default() {
        return Ok(());
    }
    let mut fields = object(base);
    for (key, value, default) in [
        ("opacity", style.opacity(), 0.45),
        ("cornerRadius", style.corner_radius(), 8.0),
        ("titleSize", style.title_size(), 13.0),
        ("iconSize", style.icon_size(), 48.0),
        ("iconSpacing", style.icon_spacing(), 12.0),
    ] {
        number(&mut fields, key, value, default, new)?;
    }
    optional_string(&mut fields, "tint", style.tint.as_deref(), new);
    optional_string(&mut fields, "blur", style.blur.as_deref(), new);
    optional_string(&mut fields, "titleFont", style.title_font.as_deref(), new);
    enumeration(
        &mut fields,
        "labelMode",
        style.label_mode,
        LabelMode::TwoLines,
        LABEL_MODES,
        new,
    )?;
    optional_string(&mut fields, "theme", style.theme.as_deref(), new);
    parent.insert("style".into(), Value::Object(fields));
    Ok(())
}

fn merge_state(parent: &mut Object, state: &DrapeState) -> Result<(), Unrepresentable> {
    let base = parent.get("state");
    let new = is_new(base);
    if new && *state == DrapeState::default() {
        return Ok(());
    }
    let mut fields = object(base);
    boolean(&mut fields, "rolledUp", state.rolled_up, false, new);
    boolean(&mut fields, "locked", state.locked, false, new);
    enumeration(
        &mut fields,
        "sortMode",
        state.sort_mode,
        SortMode::Manual,
        SORT_MODES,
        new,
    )?;
    integer(
        &mut fields,
        "zOrderHint",
        i64::from(state.z_order_hint),
        new,
    );
    parent.insert("state".into(), Value::Object(fields));
    Ok(())
}

fn merge_settings(parent: &mut Object, settings: &Settings) -> Result<(), Unrepresentable> {
    let base = parent.get("settings");
    let new = is_new(base);
    let defaults = Settings::default();
    if new && *settings == defaults {
        return Ok(());
    }
    let mut fields = object(base);
    string(
        &mut fields,
        "language",
        &settings.language,
        &defaults.language,
        new,
    );
    string(&mut fields, "theme", &settings.theme, &defaults.theme, new);
    boolean(
        &mut fields,
        "hideIconsWhenActive",
        settings.hide_icons_when_active,
        defaults.hide_icons_when_active,
        new,
    );
    merge_quick_hide(
        &mut fields,
        settings.quick_hide_enabled,
        defaults.quick_hide_enabled,
        new,
    );
    boolean(
        &mut fields,
        "autostart",
        settings.autostart,
        defaults.autostart,
        new,
    );
    boolean(
        &mut fields,
        "inboxEnabled",
        settings.inbox_enabled,
        defaults.inbox_enabled,
        new,
    );
    enumeration(
        &mut fields,
        "dropMode",
        settings.drop_mode,
        defaults.drop_mode,
        DROP_MODES,
        new,
    )?;
    string(
        &mut fields,
        "logLevel",
        &settings.log_level,
        &defaults.log_level,
        new,
    );
    enumeration(
        &mut fields,
        "updateCheck",
        settings.update_check,
        defaults.update_check,
        UPDATE_CHECKS,
        new,
    )?;
    parent.insert("settings".into(), Value::Object(fields));
    Ok(())
}

fn merge_quick_hide(fields: &mut Object, value: bool, default: bool, new: bool) {
    let flat = fields.contains_key("quickHideEnabled");
    let nested = fields
        .get("quickHide")
        .and_then(Value::as_object)
        .is_some_and(|object| object.contains_key("enabled"));
    if flat || !nested {
        boolean(fields, "quickHideEnabled", value, default, new);
    }
    if nested && let Some(object) = fields.get_mut("quickHide").and_then(Value::as_object_mut) {
        boolean(object, "enabled", value, default, new);
    }
}
