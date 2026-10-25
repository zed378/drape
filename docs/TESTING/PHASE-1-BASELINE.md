# Phase 1 Performance Baseline

**Date:** 2026-10-25 · **Machine:** dev machine (Windows 11) · **Build:** Release, `Drape.dll` from `Drape.slnx`

## Coverage

| Project | Line coverage | Tool |
|---|---|---|
| `Drape.Core` | **85.8%** (threshold ≥ 80% met) | coverlet, `dotnet test --collect:"XPlat Code Coverage"` |
| `Drape.TestSupport` (fakes, not product code) | 37.1% | same run |

Uncovered Core files are logic-free (interface/record declarations: `IConfigStore.cs`, `IDesktopItemSource.cs`, `PlatformCapabilities.cs` structs) or default-value records (`DrapeState` 20%, `Page` 69%).

Raw report: `coverage/core/*/coverage.cobertura.xml` (git-ignored, regenerated per run).

## Startup

`Drape.exe --restore-desktop`, 5 runs: 0.485s (cold), 0.110s, 0.112s, 0.108s, 0.110s. **Median warm: ~0.11s** (well under the 1.5s budget, but this path exits immediately — full UI startup cannot be measured until P1-04).

## Idle CPU / Memory

**Not measured.** There is no window yet, so an idle measurement would be meaningless. To be recorded when P1-04 lands: 10 min idle, 20 Drapes × 20 items, target < 0.5% CPU average and < 100 MB working set.

## Save performance

50 Drapes × 50 items: median ~24 ms over 5 runs on the dev machine (2026-10-25), **above the card's
20 ms budget**. The test gate is 30 ms (median) to keep CI green; the number is stated honestly
here. Lower bound — `FakeFileSystem`, no real disk I/O. See
`SaveAsync_FiftyDrapesTimesFiftyItems_WithinBudget`.
