# P1-01, P1-02, P1-03, P1-06, P1-11 — Core Domain, Contracts, ConfigStore, Layout, Persistence

**Date:** 2026-10-23
**Task:** P1-01, P1-02, P1-03, P1-06, P1-11 — the Core-only cards of Phase 1
**Author:** code agent
**Spec refs:** [`docs/ARCHITECTURE/02-DATA-MODEL.md`](../../docs/ARCHITECTURE/02-DATA-MODEL.md) · [`docs/ENGINEERING/00-CODING-STANDARDS.md`](../../docs/ENGINEERING/00-CODING-STANDARDS.md) · ADR-005, ADR-011, ADR-015, ADR-017

---

## Hook

The domain type `Drape` cannot have an explicit constructor. `public Drape(...)` does not compile:
in that position `Drape` resolves to the *root namespace*, not the type. CS-10 says to alias the
type — which works in *consumers* but not in the declaration itself. The record had to be rewritten
to use a primary constructor with `init` accessors, and the same trap then hit every test file.

---

## What Changed

**`src/Drape.Core/`**

- `Model/` — `Drape`, `DrapeItem`, `DrapeBounds`, `DrapeStyle`, `DrapeState`, `DrapeConfig`, `Page`,
  `DrapeSettings`, plus the `DrapeKind`, `DrapeItemKind`, `LabelMode`, `SortMode`, `UpdateCheckMode`
  and `DropMode` enums.
- `IClock`, `IFileSystem`, `IConfigStore` (+ `ConfigLoadResult`, `ConfigLoadOutcome`),
  `IIconProvider` (+ `IconBitmap`), `IDesktopIcons`, `IDesktopLocations`, `IAutostart`,
  `IDesktopItemSource` (+ `DesktopEntry`), `IDesktopHost` (+ `DesktopHostStatus`),
  `IDisplayMonitor` (+ `BoundsResolution`, `MonitorResolution`), `IBlurEffect` (+ `BlurKind`).
- `Layout/LayoutService` — clamp, minimum size, visible sliver, fractions.
- `Layout/MonitorResolver` — monitor-relative resolution, fallback by fraction, DPI rescale.
- `Persistence/ConfigPersistence` — 500 ms debounce with a flush for shutdown.

**`src/Drape.Infrastructure/Configuration/`**

- `ConfigJson` — DTOs separate from the domain, `JsonExtensionData` for unknown root properties.
- `ConfigMigrations` — migration table plus `Reconcile` for orphaned Drapes.
- `FileConfigStore` — temp-file write, atomic replace, 10 rolling backups, quarantine.

**`tests/Drape.TestSupport/`** — `FakeClock`, `FakeFileSystem`, `FakeConfigStore`,
`FakeDesktopIcons`, `FakeDesktopHost`, `FakeIconProvider`, `FakeDisplayMonitor`,
`FakeDesktopLocations`, `FakeAutostart`, `FakeDesktopItemSource`, `FakeBlurEffect`.

**`tests/`** — `DrapeModelTests` (26), `LayoutServiceTests` (20), `ConfigPersistenceTests` (7),
`FileConfigStoreTests` (13).

---

## Why

These five cards are the part of Phase 1 that does not need a live desktop, a VM, mixed-DPI hardware
or a decision from spike P0-03. Everything they assert is provable with a fake clock and an
in-memory file system, so they could be finished while ADR-007 is still `Proposed`.

`FakeFileSystem` deliberately *has* a `Delete` method and records calls to it. The abuse case in
P1-07 is a fake with no delete method, which makes "no file was deleted" a test that cannot fail.

`ConfigJson` is a separate DTO layer rather than serialising the records. The records validate in
their constructors; a serialiser writing them directly would either bypass that validation or force
the domain model into a JSON-shaped design that cannot be migrated.

---

## Alternatives Considered

- **Serialise the domain records directly with `System.Text.Json`.** Rejected: constructor
  validation is bypassed by deserialisation, and a schema change would require changing the domain.
- **A `Timer`-based debounce.** Rejected: untestable without sleeping. `ConfigPersistence` exposes
  "is a save due?" so a fake clock drives it.
- **Clamp the position before the size.** Rejected: the position clamp needs the final size to know
  how far a Drape may hang off an edge.
- **Store absolute monitor coordinates.** Rejected by the data model: they are meaningless once the
  monitor is unplugged. Fractions are stored alongside as the fallback.
- **Make `IConfigStore.SaveAsync` silently skip when read-only.** Rejected: a caller that believes it
  saved is worse than one that gets an exception.

---

## What Surprised Me

1. **`public Drape(...)` does not compile.** The type name is the root namespace, so the constructor
   declaration resolves the name to the namespace. A primary constructor sidesteps it because it
   never names the type. The alias from CS-10 fixes *consumers*, not the declaration.
2. **A property named `SortMode` shadows the enum `SortMode`** inside its own class body, so
   `SortMode.Name` resolved to the property. Fixed with type aliases (`SortModeValue`).
3. **`ClampPosition` had a real bug** for a work area narrower than the 40 DIP sliver: `maxX` fell
   below `workArea.X` and the Drape was clamped to a negative coordinate. The test caught it; the
   sliver is now also limited by the work area.
4. **`dotnet format` enforces `charset = utf-8-bom`** from `.editorconfig` and reports it as
   `CHARSET: Fix file encoding`, which does not mention the BOM. Every file written by the editor had
   to be re-saved with a BOM.
5. **`JsonExtensionData` alone does not preserve anything.** It captures unknown properties on read,
   but unless the store keeps them and writes them back, the next save silently drops them. The test
   caught this; `_unknownRoot` was added.

---

## Tests

- Unit (`Drape.Core.Tests`, 61 tests):
  - `DrapeModelTests` — `Drape_WithBlankTitle_Throws`, `Drape_WithEmptyId_Throws`,
    `Drape_WithEmptyPageId_Throws`, `Bounds_WithWidthBelowMinimum_Throws`,
    `Bounds_WithHeightBelowMinimum_Throws`, `Bounds_AtMinimumSize_IsAccepted`,
    `Bounds_WithFractionOutsideRange_Throws`, `Drape_WithDuplicateItemIds_Throws`,
    `Drape_WithDuplicatePaths_Throws`, `AddItem_AppendsAndAssignsOrder`,
    `AddItem_WithDuplicatePath_Throws`, `RemoveItem_RenumbersRemainingItems`,
    `Items_ExposesReadOnlyView`, `Item_WithBlankPath_Throws`, `Item_WithNegativeOrder_Throws`,
    `MarkMissing_KeepsFirstTimestamp`, `Config_WithNoPages_Throws`,
    `Config_WithDuplicatePageIds_Throws`, `Config_WithDrapeOnMissingPage_Throws`,
    `Config_WithItemInTwoDrapes_Throws`, `Config_WithNonPositiveSchemaVersion_Throws`,
    `RemoveDrape_RemovesOnlyTheDrape`, `ReplaceDrape_WithUnknownId_Throws`,
    `Style_WithOpacityOutsideRange_Throws`, `Style_WithIconSizeBelowMinimum_Throws`,
    `DefaultStyle_IsValid`, `IconBitmap_WithMismatchedBuffer_Throws`
  - `LayoutServiceTests` — `ClampPosition_InsideWorkArea_IsUnchanged`,
    `ClampPosition_PastRightEdge_KeepsVisibleSliver`, `ClampPosition_PastBottomEdge_KeepsVisibleSliver`,
    `ClampPosition_PastLeftEdge_KeepsVisibleSliver`, `ClampPosition_WithTinyWorkArea_DoesNotEscape`,
    `ClampSize_BelowMinimum_RaisesToMinimum`, `ClampSize_LargerThanWorkArea_IsReducedToWorkArea`,
    `Clamp_ShrinksThenRepositions`, `ToFractions_ComputesProportions`,
    `ToFractions_WithZeroWorkArea_DoesNotThrow`, `Resolve_WithMonitorPresent_ReturnsExact`,
    `Resolve_WithMonitorMissing_ScalesOntoFallback`, `Resolve_WithShrunkWorkArea_ReturnsClamped`,
    `Resolve_WithNoMonitors_Throws`, `Resolve_TenCycles_DoesNotDrift`,
    `Rescale_WithNewDpi_KeepsDipPosition`, `Rescale_WithZeroDpi_Throws`
  - `ConfigPersistenceTests` — `SaveIfDue_BeforeInterval_DoesNotSave`, `SaveIfDue_AfterInterval_Saves`,
    `Queue_TwiceWithinInterval_WritesOnlyTheLast`, `Flush_BeforeDebounceElapses_StillSaves`,
    `Flush_WithNothingPending_DoesNotSave`, `SaveIfDue_ClearsPendingFlag`,
    `Flush_WhenStoreFails_Throws`
- Integration (`Drape.Infrastructure.Tests`, 13 tests):
  - `LoadAsync_WithNoFile_ReportsFirstRun`, `SaveThenLoad_RoundTripsTheDocument`,
    `SaveAsync_Twice_KeepsPreviousVersionAsBackup`, `SaveAsync_WhenReplaceFails_LeavesOldConfigIntact`,
    `LoadAsync_WithCorruptConfig_QuarantinesAndUsesBackup`,
    `LoadAsync_WithCorruptConfigAndNoBackup_StartsEmpty`, `LoadAsync_WithNewerSchema_IsReadOnly`,
    `LoadAsync_WithDomainViolation_TreatsAsCorrupt`, `SaveAsync_PrunesBackupsBeyondLimit`,
    `SaveThenLoad_PreservesUnknownProperties`, `LoadAsync_WhenFileCannotBeRead_Recovers`,
    `LoadAsync_WithCorruptConfig_DoesNotDeleteIt`, `SaveAsync_AlwaysWritesCurrentSchemaVersion`
- **Desktop-safety test:** not applicable — no icon, host or hook code in this change.
  `IDesktopIcons` has no toggle by design (P1-02 DoD); the fake records call order so a future test
  can prove hide-then-restore.
- **Mutation checks:** two, both observed to fail and then reverted.
  1. Removed the atomic replace from `FileConfigStore.Write` (wrote in place instead) →
     `SaveAsync_WhenReplaceFails_LeavesOldConfigIntact` failed with "should throw IOException but
     did not".
  2. (From the previous session) added Avalonia to `Drape.Core` → the layering test failed.

---

## Platform Matrix

| Windows build | DPI scale | Monitors | Result |
|---|---|---|---|
| Development machine (Windows 11), compile + unit tests | — | — | Build 0 warnings; 81 tests pass; format clean |
| — | — | — | No live desktop, no VM, no mixed DPI |

---

## Definition of Done

- [x] Every entity matches `docs/ARCHITECTURE/02-DATA-MODEL.md`
- [x] Validation tested including the failing branches (26 rejection/acceptance assertions)
- [x] `Drape.Core` has no Avalonia, Win32 or platform reference — the P0-01 layering test is green
- [x] All interfaces have XML documentation, including threading and failure behaviour
- [x] A fake exists for each interface; each fake is used by at least one named test
- [x] No contract exposes an Avalonia, WPF or Win32 type
- [x] `IDesktopIcons` exposes an explicit read; there is no blind toggle
- [x] Simulated replace failure leaves the old config intact
- [x] A corrupt config is quarantined and the newest valid backup is used
- [x] A config with a newer version opens read-only and is never overwritten
- [x] Mutation check: removing the replace-step atomicity made the crash test fail
- [ ] Saving 50 Drapes × 50 items under 20 ms — **not measured**
- [ ] Core coverage ≥ 80 % — **not measured**
- [ ] Feature spec written first in `MEMORY/specs/` for P1-03 — **not written**
- [ ] P1-06: smoothness at 144 Hz — **not applicable yet**, there is no view

**Waived:** none. The unticked items are not waived. P1-03 in particular is `WIP`, not `DONE`: its
spec and its performance number are missing.

---

## Desktop Safety

- **Icon hide path:** none. `IDesktopIcons` is a contract; nothing implements it yet.
- **Restore paths exercised:** none.
- **Handles:** no native handle in this change.
- **Hook callback:** none.
- **Capability gating:** `SupportsDesktopLevelWindow` still returns `false`; nothing here changes it.
- **Config:** schema v1 introduced with a migration table and a version check. Atomic write is
  `temp → replace`. Backups are pruned to 10. A corrupt file is moved to `quarantine\`, never
  deleted — asserted by `LoadAsync_WithCorruptConfig_DoesNotDeleteIt`.
- **Privacy:** no network call. The config contains user file paths, so it is treated as personal
  data; nothing logs a path (CS-37).

---

## What to Watch

- `Drape` cannot gain an explicit constructor. Any refactor that "tidies" the primary constructor
  into one will not compile, and the error (CS1520 / CS0118) does not mention the namespace clash.
- `MonitorResolver.Rescale` currently only updates the recorded DPI. If a future change makes DIP
  DPI-dependent, this is the function that becomes wrong.
- `ConfigMigrations.Migrate` has a v1→v2 case that does nothing. When v2 is defined, that case must
  be replaced and a hand-written `config-v1.json` fixture added; a fixture generated from the
  migrator's own output would hide a dropped field.
- `FileConfigStore` treats a document that parses but violates a domain rule as corrupt and
  quarantines it. If a future version relaxes a rule, old files that violate it will be quarantined
  rather than loaded.

---

## What Was Not Determined

- The 20 ms save budget for 50 × 50 was not measured. The fake file system has no I/O cost, so a
  measurement against it would be meaningless; this needs a real-disk benchmark.
- Core coverage was not measured; no coverage tool is configured yet.
- No feature spec exists in `MEMORY/specs/` for P1-03, which its DoD requires before `DONE`.
- The migration path is untested against a real v1 file, because v1 is the only version that exists.
  `Migrate` is exercised only through the read-only-newer-version path.
- Nothing here was run on Windows 10 22H2 or at a DPI other than the development machine's.
- P1-04, P1-05, P1-07, P1-08, P1-09, P1-10 (the window and shell cards) were not started: they need
  ADR-007 to be accepted, which needs spike P0-03, which needs hardware and VMs.
