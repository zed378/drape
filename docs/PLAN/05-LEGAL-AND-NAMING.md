# 05 — Legal and Naming

Note: this document is a working guideline for the team, **not legal advice**. Consult counsel before any public or commercial release.

## Project Position

Drape is an independent product in the general "desktop organizer" category. The idea and general function (grouping icons in a panel) are not monopolised by any one product; specific *expression* (code, assets, text, distinctive design) is protected.

## Mandatory Rules

1. **Name.** Do not use "Fences", "Stardock" or related names in the product name, file names, namespaces, icons, website or marketing material. Product name: **Drape** (final, owner decision, [ADR-019](../../MEMORY/DECISIONS.md)). A trademark search is still a pre-release checklist item below: the name is chosen, not cleared.
2. **Code.** No copying, decompiling or deriving from another product's code. Clean-room implementation based on generally observable behaviour and public platform documentation.
3. **Assets.** All icons, images, fonts and sounds are original or clearly licensed (recorded in `THIRD_PARTY_NOTICES.md`).
4. **Text.** UI text, documentation and examples are written in-house.
5. **Visuals.** Do not imitate another product's distinctive look pixel for pixel, including branded feature names.
6. **Reverse engineering.** Disassembling another product's binaries is forbidden. Platform spikes study only the operating system's own behaviour (Spy++/Inspect on Explorer), which is standard practice.
7. **Comparison.** Other products may be mentioned factually in internal documents to describe the category ("a Stardock-Fences-style organizer"), not in public material without review.
8. **Patents.** If a specific feature raises a patent concern, record it in [`../../TASKS/BACKLOG.md`](../../TASKS/BACKLOG.md) and review it before release.

### Internal identifiers

Code uses `Drape` for the panel concept as well ([ADR-020](../../MEMORY/DECISIONS.md)); no identifier, namespace, file or config key contains the word "fence". The word appears in the repository only when referring to the competing product.

## Project Licence

**MIT**, free of charge — owner decision ([ADR-019](../../MEMORY/DECISIONS.md), closes Q-02). The `LICENSE` file is created in P0-01. MIT carries no patent grant and no trademark grant. Dependency licences must stay compatible with MIT distribution.

## Dependencies and Licences

- Only permissively licensed packages (MIT, Apache-2.0, BSD) with no obligations that conflict with the distribution plan. Avalonia is MIT-licensed; verify the exact licence of each package when versions are locked in P0-01.
- Avoid packages that move to commercial licences or add usage restrictions.
- Maintain `THIRD_PARTY_NOTICES.md`, generated from the package list at release.
- Fonts and icons (e.g. Segoe Fluent Icons) are used only per Microsoft's and each platform's licence; check restrictions before bundling.
- Store distribution (App Store, Play Store, Mac App Store) has its own rules, and some prohibit behaviour Drape relies on (global hooks, Finder/desktop manipulation). Review per platform in Phase 6 before committing to a channel.

## Trademarks and References

Windows, OneDrive, macOS and other product names are trademarks of their owners. Use their official names, only as compatibility references.

## Contributions

If the project accepts contributions: use the DCO (`Signed-off-by`) or a CLA; contributors affirm the code is their own and not copied from an incompatible source.

## Pre-Public-Release Checklist

- [ ] Trademark search for the final name in target regions.
- [ ] Asset audit (icons, fonts, images, text).
- [ ] `THIRD_PARTY_NOTICES.md` complete.
- [ ] `LICENSE` (MIT) present and dependency licences checked against it.
- [ ] Short privacy statement (no telemetry) and a statement that data is local.
- [ ] Legal review if the release is commercial.
