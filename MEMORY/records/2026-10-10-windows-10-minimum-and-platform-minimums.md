# 2026-10-10 — Windows 10 22H2 is the minimum OS; minimums and channels for other platforms (ADR-023)

**Date:** 2026-10-10
**Task:** — (decision; overturns the Q-05 answer in ADR-022)
**Author:** assistant, on the owner's decision
**Spec refs:** [`docs/PLATFORM/01-PLATFORM-SUPPORT.md`](../../docs/PLATFORM/01-PLATFORM-SUPPORT.md) · [`docs/PLAN/01-PRODUCT-REQUIREMENTS.md`](../../docs/PLAN/01-PRODUCT-REQUIREMENTS.md)

---

## Hook

The owner promoted Windows 10 from "best effort" to the supported minimum, which turns every Windows 10 test from optional into a release gate and makes the Windows 11-only visual APIs (Mica) a two-path problem — while Microsoft's own security support for Windows 10 ends for most users on 12 October 2027.

---

## What Changed

- ADR-023 added; ADR-009 and the Q-05 answer in ADR-022 marked superseded. The owner set no sunset date.
- Wording changed from "best effort" to "minimum, fully supported" in CLAUDE.md, README, overview, PRD NFR-07, the testing matrix and release criteria, DoD in the task conventions, Phase 0/1/2 cards and templates.
- New risk PR-9; backlog items B-10, B-11; new section "Minimum OS and Distribution" in the platform-support document; winget `MinimumOSVersion: 10.0.19045.0`.
- Other platforms (assistant's decision on the owner's delegation): macOS 14, Ubuntu 22.04/Debian 12 families, Android 12, iOS 18, with a channel per platform.

## Why

Owner decision on reach: Windows 10 was still about 28% of Windows desktops (StatCounter, September 2026).

## Alternatives Considered

See ADR-023 (Windows 11 only; best effort; newer minimums elsewhere).

## What Surprised Me

My own edit script damaged two lines (a regex replacement that wrote a literal back-reference): the Q-05 row of the backlog and the Status line of Phase 6. They were found by re-reading and restored by hand; a scan for leaked back-references was added to the check.

## Tests

None; documents only. Link targets re-checked by script after the edits: 0 broken.

## Definition of Done

- [x] ADR, risk, backlog, platform doc, index and changelog updated
- [x] Link targets checked; no leaked back-references
- [ ] Any Windows 10 behaviour verified — **not possible**, nothing exists yet

**Waived:** none.

## Security and Compliance

No code change. Supporting an OS that Microsoft no longer patches for most users is a security-posture decision, recorded as PR-9.

## What to Watch

Any feature needing a Windows 11-only API; the 12 October 2027 ESU end; P0-06 must deliver a Windows 10 blur path.

## What Was Not Determined

Whether acrylic via the undocumented composition attribute still works on Windows 10 22H2 with Avalonia; Apple/Google/Apple store policies; Avalonia's tiers were read, not exercised.

## Follow-Ups

PR-9 review by 12 October 2027; B-10, B-11.

## Links

- ADR: ADR-023 in [`../DECISIONS.md`](../DECISIONS.md)
