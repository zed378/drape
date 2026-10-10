# 2026-10-10 — Remaining open questions answered; root README and LICENSE (ADR-022)

**Date:** 2026-10-10
**Task:** — (Q-04 … Q-13 and Q-18; root `README.md` and `LICENSE`)
**Author:** assistant, on the owner's instruction
**Spec refs:** [`docs/DEVOPS/00-BUILD-AND-RELEASE.md`](../../docs/DEVOPS/00-BUILD-AND-RELEASE.md) · [`docs/PLAN/05-LEGAL-AND-NAMING.md`](../../docs/PLAN/05-LEGAL-AND-NAMING.md)

---

## Hook

The research changed two assumptions: Windows 10 is not "past end of life" in practice (consumer ESU now runs to 12 October 2027 and it is still ~28% of Windows desktops), and an MSIX-packaged app redirects HKCU writes, which would silently break the `Run`-key autostart — so direct distribution won over the Store.

---

## What Changed

- ADR-022 closes Q-04, 05, 06, 07, 08, 09, 10, 11, 12, 13 and 18 (table of answers, evidence, sources). `BACKLOG.md` now has no open question; B-09 (Store/MSIX) and three new unverified claims were added.
- Minimum mobile OS versions changed from Android API 26 / iOS 16 to Android 12 (API 31) / iOS 18, following Avalonia's support tiers.
- Windows 10 wording, ARM64 wording (x64 only in v1.0), update-check wording and the distribution section of the build document were updated. `AGENTS.md` gained a "Concurrency Limits" section (WIP 3; at most 4 concurrent subagents).
- Created the root `README.md` and `LICENSE` (MIT, "Drape contributors", 2026).

## Why

Owner instruction: answer every remaining question, research if needed.

## Alternatives Considered

In ADR-022 (Store/MSIX, Windows 11 only, ARM64 in v1.0, default-on update checks).

## What Surprised Me

A web search for MSIX and global hooks returned nothing relevant; the useful fact came from Microsoft's page on how packaged apps run (HKCU copy-on-write). The same page is silent about hooks, so that question is open, not answered.

## Tests

None; documents only. Relative links were re-checked by script after the edits (see the next record's closing check).

## Definition of Done

- [x] ADR written, backlog, index and changelog updated
- [x] README and LICENSE created
- [ ] Any research claim verified on a real machine — **not possible**, no code

**Waived:** none.

## Security and Compliance

No code change. The licence file is the MIT text; the copyright holder is the placeholder "Drape contributors" because the owner's legal name was not supplied.

## What to Watch

- Replace the copyright holder with the owner's legal name or organisation if desired.
- Re-decide Windows 10 support before 12 October 2027.
- P5-03 must prove winget accepts the Velopack installer.

## What Was Not Determined

Whether the Store allows a global low-level mouse hook; whether winget accepts the installer; how x64 emulation behaves on Windows ARM; the exact winget manifest rules (the search results were third-party mirrors). StatCounter numbers are an estimate.

## Follow-Ups

B-09 (Store/MSIX), B-02 (ARM64), Windows 10 re-decision in 2027.

## Links

- ADR: ADR-022 in [`../DECISIONS.md`](../DECISIONS.md)
- Backlog: Q-04 … Q-13, Q-18 closed
