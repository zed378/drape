//! Schema-1 configuration persistence over the injected `FileSystem` port.

mod read;
mod scan;
mod store;
mod time;
mod write;

pub use store::{GuidSource, JsonConfigStore};

use drape_core::domain::{
    DomainConfig, DrapeKind, DropMode, ItemKind, LabelMode, Repair, SortMode, UpdateCheck,
};

pub const MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DEPTH: usize = 64;
/// Bytes between the quotes of one stored string, escapes included.
pub const MAX_STRING_BYTES: usize = 256 * 1024;
pub const MAX_VALUES: usize = 200_000;
pub const MAX_BACKUPS: usize = 1024;
const RETAINED_BACKUPS: usize = 10;

// Accepted stored spellings. The first entry for a variant is the one written.
const DRAPE_KINDS: &[(&str, DrapeKind)] = &[
    ("normal", DrapeKind::Normal),
    ("inbox", DrapeKind::Inbox),
    ("portal", DrapeKind::Portal),
];
const ITEM_KINDS: &[(&str, ItemKind)] = &[
    ("file", ItemKind::File),
    ("folder", ItemKind::Folder),
    ("shortcut", ItemKind::Shortcut),
];
const LABEL_MODES: &[(&str, LabelMode)] = &[
    ("none", LabelMode::None),
    ("oneline", LabelMode::OneLine),
    ("twolines", LabelMode::TwoLines),
    ("one-line", LabelMode::OneLine),
    ("two-lines", LabelMode::TwoLines),
];
const SORT_MODES: &[(&str, SortMode)] = &[
    ("manual", SortMode::Manual),
    ("name", SortMode::Name),
    ("type", SortMode::Type),
    ("dateadded", SortMode::DateAdded),
];
const UPDATE_CHECKS: &[(&str, UpdateCheck)] = &[
    ("off", UpdateCheck::Off),
    ("manual", UpdateCheck::Manual),
    ("periodic", UpdateCheck::Periodic),
];
const DROP_MODES: &[(&str, DropMode)] = &[("reference", DropMode::Reference)];

/// A writable stored document with its domain projection.
struct Document {
    value: serde_json::Value,
    config: DomainConfig,
    repairs: Vec<Repair>,
}

enum Class {
    Valid(Box<Document>),
    /// Read-only; the projection is absent when the bytes are ambiguous.
    Unsupported(Option<Box<DomainConfig>>),
    Newer(u32),
    Corrupt,
    TooLarge,
}
