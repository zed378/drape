# 02 — Data Model

Status: draft. Storage decision: ADR-005.

## File Locations

| File | Windows path | Contents |
|---|---|---|
| Main config | `%LOCALAPPDATA%\Drape\config.json` | Settings, pages, Drapes, rules |
| Runtime state | `%LOCALAPPDATA%\Drape\state.json` | Icons-hidden flag, PID, version; used for recovery |
| Backups | `%LOCALAPPDATA%\Drape\backup\config-YYYYMMDD-HHmmss.json` | Last 10 (rolling) |
| Snapshots | `%LOCALAPPDATA%\Drape\snapshots\<id>.json` | Saved layouts |
| Logs | `%LOCALAPPDATA%\Drape\logs\Drape-YYYYMMDD.log` | Rolling, 14-day retention |
| Corrupt files | `%LOCALAPPDATA%\Drape\quarantine\` | Config that failed to load, kept for diagnosis |

Why `LocalAppData`: the layout depends on the machine (monitors, DPI), so it is not roamed.

Other platforms (assumption, confirmed in Phase 6): macOS `~/Library/Application Support/Drape`, Linux `$XDG_CONFIG_HOME/drape` (config) and `$XDG_STATE_HOME/drape` (state, logs), Android/iOS the app-private sandbox. The base directory is supplied by the platform through DI, never assembled from `%LOCALAPPDATA%` in shared code.

## Entities

```
Config
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

The entity is called `Drape` in code and UI alike ([ADR-020](../../MEMORY/DECISIONS.md)); the root namespace is also `Drape`, see the naming rule CS-10 in the coding standards.
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

- Primary identity: the normalised `path` (case-insensitive on Windows, environment variables expanded, `\` separators, no trailing slash). Case sensitivity is a per-platform rule supplied by `IFileSystem`; it is not hard-coded in Core.
- Secondary tracker: the file ID (Windows NTFS: volume ID + file ID; macOS/Linux: device + inode) to follow renames within one volume. If the ID matches but the path differs, update the path instead of creating a new item.
- Item not found: set `missingSince`; drop the reference after 24 hours without reappearing (configurable).
- Desktop locations come from the platform (`IDesktopLocations`; Windows: `FOLDERID_Desktop`, `FOLDERID_PublicDesktop`), never from `%USERPROFILE%\Desktop`, so they are right when redirected by OneDrive.

## Atomic Writes and Durability

1. Serialise to `config.json.tmp` in the same directory.
2. Flush to disk (`File.Flush(true)`).
3. Replace the live file: Windows `File.Replace(tmp, config.json, backupPath)` (or an atomic `Move` if the file does not exist yet); POSIX atomic `rename`. Wrapped behind `IFileSystem`.
4. Prune backups to the newest 10.
5. Save is debounced by 500 ms; shutdown saves synchronously.

On load:
- Parse failure → move the file to `quarantine\`, try the newest valid backup, tell the user once.
- No valid backup → start from a clean empty config, without touching the user's Desktop files.

## Schema Versioning and Migration

- `schemaVersion` is an integer incremented for each incompatible change.
- A migration is a pure function `Migrate(vN) → vN+1` in Core, tested with one JSON fixture per version (`tests/Drape.Core.Tests/Fixtures/config-v1.json`, …).
- Config with a newer version than the app: open read-only, never overwrite, tell the user to update.
- Unknown properties are preserved on save (so a downgrade does not drop data), using `JsonExtensionData`.

## Validation

- IDs are GUIDs and unique.
- `Drape.pageId` must exist in `pages`; otherwise move the Drape to the first page.
- `bounds` is clamped to the minimum size (120 × 80 DIP) and to the work area.
- An item present in several Drapes: keep the one with the newest `addedAt`.

## Data Privacy

Config contains the user's file paths: treat it as personal data. It is never transmitted. Export offers an anonymise option. See [`../SECURITY/00-SECURITY-AND-PRIVACY.md`](../SECURITY/00-SECURITY-AND-PRIVACY.md).
