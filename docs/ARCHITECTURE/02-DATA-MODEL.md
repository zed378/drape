# 02 — Data Model

Status: Rust target contract, ADR-005 and ADR-025. The language migration preserves schema version 1, field names, enum values and existing data locations. Implementation is future R1-02 work.

## File Locations

| File | Windows path | Contents |
|---|---|---|
| Main config | `%LOCALAPPDATA%\Drape\config.json` | Settings, pages, Drapes, rules |
| Runtime state | `%LOCALAPPDATA%\Drape\state.json` | Versioned recovery ownership/intent, original visibility and validated process identity; legacy records remain recoverable |
| Backups | `%LOCALAPPDATA%\Drape\backup\config-YYYYMMDD-HHmmss.json` | Last 10 (rolling) |
| Snapshots | `%LOCALAPPDATA%\Drape\snapshots\<id>.json` | Saved layouts |
| Logs | `%LOCALAPPDATA%\Drape\logs\Drape-YYYYMMDD.log` | Rolling, 14-day retention |
| Corrupt files | `%LOCALAPPDATA%\Drape\quarantine\` | Config that failed to load, kept for diagnosis |

Why `LocalAppData`: the layout depends on the machine (monitors, DPI), so it is not roamed.

Other platforms (assumption, confirmed in Phase 6): macOS `~/Library/Application Support/Drape`, Linux `$XDG_CONFIG_HOME/drape` (config) and `$XDG_STATE_HOME/drape` (state, logs), Android/iOS the app-private sandbox. The base directory is supplied by the platform through DI, never assembled from `%LOCALAPPDATA%` in shared code.

## Entities

```
Config (conceptual field inventory; not an exhaustive accepted wire schema)
 ├─ schemaVersion : int
 ├─ settings : Settings
 ├─ pages    : Page[]
 ├─ drapes   : Drape[]
 └─ rules    : Rule[]

Page      { id, name, order }
Drape     { id, pageId, title, kind, bounds, style, state, items[], portal?, tabs?, quickHideExcluded }
DrapeItem { id, kind, path, fileId?, displayName?, order, pinnedByUser, addedAt, missingSince? }
Bounds    { monitorId, x, y, width, height, dpi, fx, fy, fw, fh }   // x,y,w,h in DIP relative to the monitor work area; f* = fractions 0..1 for fallback
Style     { tint, opacity, blur, cornerRadius, titleFont, titleSize, iconSize, iconSpacing, labelMode, theme? }
State     { rolledUp, locked, sortMode, zOrderHint }
Portal    { folderPath, recursive, filter? }
Rule      { id, name, enabled, priority, conditions[], targetDrapeId }
Snapshot  { id, name, createdAt, drapes[] }  // position, style, membership
Settings  { language, theme, hideIconsWhenActive, quickHide{enabled}, autostart, inboxEnabled, dropMode, logLevel, updateCheck }
```

The entity is `Drape` in Rust and UI ([ADR-020](../../MEMORY/DECISIONS.md)); package/import naming follows Rust conventions (CS-6–10).
`Drape.kind`: `normal`, `inbox`, `portal`.
`DrapeItem.kind`: `file`, `folder`, `shortcut` (v1); `virtual` (later).
`Settings.updateCheck`: `off` (default) | `manual` | `periodic` ([ADR-017](../../MEMORY/DECISIONS.md)).

## Example `config.json` (abridged)

```json
{
  "schemaVersion": 1,
  "settings": {
    "language": "en-US",
    "theme": "system",
    "hideIconsWhenActive": true,
    "quickHide": { "enabled": true },
    "autostart": false,
    "inboxEnabled": true,
    "dropMode": "reference",
    "logLevel": "Information",
    "updateCheck": "off"
  },
  "pages": [
    { "id": "6f1c2b8e-0000-4000-8000-000000000001", "name": "Main", "order": 0 }
  ],
  "drapes": [
    {
      "id": "6f1c2b8e-0000-4000-8000-000000000010",
      "pageId": "6f1c2b8e-0000-4000-8000-000000000001",
      "title": "Reports",
      "kind": "normal",
      "bounds": {
        "monitorId": "\\\\?\\DISPLAY#EXAMPLE#1",
        "x": 40, "y": 60, "width": 320, "height": 240,
        "dpi": 144, "fx": 0.021, "fy": 0.056, "fw": 0.167, "fh": 0.222
      },
      "style": { "tint": "#3A6EA5", "opacity": 0.45, "blur": "acrylic", "cornerRadius": 8, "iconSize": 48, "iconSpacing": 12, "labelMode": "two-lines" },
      "state": { "rolledUp": false, "locked": false, "sortMode": "manual" },
      "quickHideExcluded": false,
      "items": [
        { "id": "6f1c2b8e-0000-4000-8000-000000000100", "kind": "file", "path": "C:\\Users\\example\\Desktop\\Q3.xlsx", "order": 0, "pinnedByUser": true, "addedAt": "2026-10-09T07:43:00Z" }
      ]
    }
  ],
  "rules": []
}
```

Paths in the example are fictional. Never put real user paths or configs in the repository.

## Item Identity

- Primary identity: a platform-supplied comparison key for the stored path. Preserve the original path losslessly for launch/display/persistence. Windows comparison is normally case-insensitive but per-directory case-sensitive settings, UNC roots and extended-length paths must be handled by the platform adapter; Core never applies global lowercase/Unicode normalisation.
- Normalisation is lexical unless the adapter explicitly requests a metadata lookup. Preserve drive/UNC roots and do not strip their final separator. File paths received from enumeration/drop/config are literal: do not expand `%NAME%` or other environment syntax inside a real filename. User-entered path expressions, if ever supported, are parsed separately before storage. Canonicalisation must not require a missing/network/cloud file to become available or trigger OneDrive hydration.
- Secondary tracker: an optional filesystem identity supplied by the platform (for example volume ID + file ID on supporting Windows filesystems, device + inode on supporting POSIX filesystems). Its availability, reuse and rename semantics are not universal. Correlate only within a validated volume/provider scope and observation window; otherwise retain path-based matching. Do not merge unrelated items solely because a recycled file ID matches.
- Item not found: set `missingSince`; drop the reference after 24 hours without reappearing (configurable).
- Desktop locations come from the platform (`DesktopLocations`; Windows: `FOLDERID_Desktop`, `FOLDERID_PublicDesktop`), never from `%USERPROFILE%\Desktop`, so they are right when redirected by OneDrive.

## Atomic Writes and Durability

1. Serialize writes through one ConfigStore writer; a validated per-user/session ownership protocol excludes another App instance. Serialise to an exclusively created temporary file in the same directory (a unique temporary name is allowed); never follow a stale link/reparse point or let concurrent writers overwrite the same temporary file.
2. Flush user-space buffers and call `std::fs::File::sync_all()`; errors must be propagated.
3. Replace the live file: Windows uses an OS-aware atomic replacement adapter (`ReplaceFileW` for an existing destination; validated first-write operation when absent), including backup/error handling; POSIX uses same-filesystem rename and directory durability as supported. Wrapped behind `FileSystem`; do not assume `std::fs::rename` has identical Windows semantics.
4. Only after the replacement is confirmed successful, prune Drape-managed backups to the newest 10. Cleanup failure is reported but does not invalidate the successful save or block desktop restoration. Failed saves retain the last valid destination/backup and remove only the writer's own temporary artefact when safe.
5. Save is debounced by 500 ms; shutdown saves synchronously.

On load:
- Parse/structural failure → preserve the original Drape-managed config in `quarantine\` without overwriting an earlier quarantined file, try the newest valid backup, tell the user once. Never quarantine or rewrite a merely newer schema. This policy moves app-managed config artefacts, not Desktop item files; it is not permission to move user content referenced by config.
- No valid backup → start from a clean empty config, without touching the user's Desktop files.

## Schema Versioning and Migration

- `schemaVersion` is an integer incremented for each incompatible change.
- A migration is a pure version-to-version transformation in Core, tested with one JSON fixture per version (`crates/drape-core/tests/fixtures/config-v1.json`, …).
- Config with a newer version than the app: open read-only, never overwrite, tell the user to update.
- Unknown properties are preserved on save (so a downgrade does not drop data), using explicit extension maps (for example serde flatten with JSON values), including nested extensible entities. Preserve unknown numeric values without precision loss; choose the serde_json representation/features and test with hand-written legacy fixtures. Unknown enum/discriminator values must not silently default to a known value or disappear: preserve their raw representation, disable the unsupported operation, and use read-only mode if a safe lossless save cannot be established. An unsupported setting is not a reason to discard an entire valid configuration.

## Validation

- IDs are GUIDs and unique. Reject or explicitly repair duplicate/invalid identities without overwriting unrelated records; retain original config/backup and report the repair. No fresh ID or timestamp is obtained from the real environment inside Core: supply them through caller inputs/ports.
- `Drape.pageId` must exist in `pages`; otherwise associate the Drape with the first valid page. If there is none, the application supplies a validated default page/ID before reconciliation; never index an empty pages array. Repairs affect references only.
- Numeric dimensions/coordinates/opacity/DPI must be finite and within validated ranges before math/rendering. Clamp bounds to minimum size (120 × 80 DIP) and applicable work area; define safe accessibility when the available area is smaller than that minimum. The stored logical position remains the reference for future monitor restoration.
- An item present in several Drapes: keep the membership with newest valid `addedAt`; exact ties use stable persisted Drape order then ID. Missing/invalid timestamps require explicit validation/repair, not wall-clock substitution. Removing duplicate membership never deletes the underlying file.
- Parse size, nesting depth, collection counts, string lengths, pixel dimensions and total decoded image/cache bytes have explicit limits selected in the relevant feature spec. Reject oversized untrusted imports before allocating unbounded memory and report the limit without truncating existing user data silently.

## Data Privacy

Config contains the user's file paths: treat it as personal data. It is never transmitted. Export offers an anonymise option. See [`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md).

## Rust compatibility and recovery state

Use explicit serde names/rename rules to retain camelCase JSON and legacy enum/log-level spellings. Rust snake_case fields do not change the wire schema. Preserve UUID/timestamp semantics and test non-ASCII, UNC, long/redirected paths without lossy conversion. JSON parsing belongs to adapters; pure migration/validation logic belongs to Core. A language change alone does not increment schemaVersion.

R1-02 uses hand-written legacy input/output fixtures, unknown/nested fields, corrupt/truncated files, future-version read-only mode and injected failures at every atomic-write step. The old C# ConfigStore report is not Rust evidence.

state.json has separate safety semantics: durably record original visibility, ownership, intended change and process identity before hiding, arm/acknowledge Guard, then mutate/verify. Record successful restoration only after verifying it. Version and validate recovery records; legacy iconsHiddenByDrape=true records must trigger conservative recovery, never be silently ignored. A new recovery-state format needs migration/compatibility design before implementation; PID alone is insufficient identity. Respect user-hidden icons and user changes during runtime.

App, Guard, next-start recovery and standalone restore serialize state transitions under the same per-user/session coordination protocol. A second App instance must not restore a still-valid owner's hidden icons before performing the single-instance check. Recovery intent records describe pending versus completed mutations, with generation/session identity, so a crash between intent, mutation and acknowledgement has a defined idempotent outcome. R0-05 specifies stale/invalid records, legacy state, concurrent restore requests and a Guard crash after readiness; no component issues a blind visibility toggle based only on an old boolean. Corrupt/unsupported recovery metadata cannot establish mutation authority. Authenticate IPC peer/generation and reject malformed/oversized messages; ambiguous state prevents new hiding and produces recovery diagnostics/manual restoration guidance. App restores owned changes on Guard loss and cannot hide again before a new authenticated Guard is armed.

## Feature representation prerequisites

The field inventory above preserves the existing schema-1 baseline; it does not define storage semantics for every future feature. F-22 pin-to-all-pages cannot be represented merely by changing a single pageId. F-24 tabs need tab identity, ordering and item ownership semantics. Portal entries are filesystem projections whose overlap with manually referenced items and the one-owning-Drape rule requires an explicit contract. Q-23 records these gaps; their features remain in planned scope.

Before P4-01 (portal), P4-04 (pages/pinning) or P4-07 (tabs) implementation, the relevant feature spec must define persisted versus derived data, ownership/projection rules, validation/repair, compatibility with schema 1, import/export and migration fixtures. Do not invent optional fields now or present unknown-property round-tripping as proof that a future schema already works. Any incompatible schema change requires a versioned migration and ADR through the existing protocol.
