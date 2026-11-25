# Phase 5 — Polish and Release

> Estimates in retained cards are provisional legacy planning allowances, not Rust delivery commitments. Re-estimate/split at claim time after the Rust spikes; follow actual per-card prerequisites.

> Rust + Slint target (ADR-025). R0/R1 dependencies refer to [Rust migration cards](./RUST-MIGRATION.md), not historical P0/P1 results. These implementation tasks remain unstarted; no code is authorised by the documentation update.

**Status:** TODO (Rust target work not started; legacy C# code retained)

**Goal:** Release quality for v1.0, **Windows only** (ADR-014): full id-ID/en-US localisation, performance inside the NFR budgets, installer and opt-in updater, code signing, security and legal audits, user documentation, full-environment testing and a limited public beta. Spec: `docs/DEVOPS/00-BUILD-AND-RELEASE.md`, `docs/TESTING/00-TESTING-STRATEGY.md`, `docs/PLAN/05-LEGAL-AND-NAMING.md`, `docs/SECURITY/00-SECURITY-AND-PRIVACY.md`.

**Exit criteria:** see Phase 5 in [`../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md`](../docs/PLAN/03-IMPLEMENTATION-ROADMAP.md). Card conventions: [`00-TASK-CONVENTIONS.md`](./00-TASK-CONVENTIONS.md).

---

### P5-01 — Full id-ID and en-US localisation

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | ui-engineer |
| **Depends on** | P4-03 |
| **Spec refs** | docs/UI-UX/00-UI-UX-SPEC.md · docs/TESTING/00-TESTING-STRATEGY.md |
| **Spec required** | no |

**Why:** Hard-coded strings are cheap now and expensive to find later; the second language is the proof that none remain.

All user-facing strings live in resources for `id-ID` and `en-US`; a language picker; pseudo-localisation to expose truncation; dates and numbers formatted by culture.

**Definition of Done**
- [ ] no hard-coded user-facing string (checked by a script or analyzer that fails the build)
- [ ] layout holds with 40% longer text (pseudo-locale screenshots reviewed)
- [ ] switching language at runtime updates open windows, tray menu and Narrator names
- [ ] both locales are complete: a test fails when a key exists in one resource and not the other

**Abuse cases**
- Auditing Rust strings while ignoring .slint strings, or the reverse
- English text copied into the id-ID resource to satisfy the "complete" check

---

### P5-02 — Performance and memory optimisation

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 4 days |
| **Role** | core-engineer |
| **Depends on** | P4-01 |
| **Spec refs** | docs/PLAN/01-PRODUCT-REQUIREMENTS.md (NFR-01..03) · docs/PLAN/04-RISK-REGISTER.md (PR-5) · docs/ARCHITECTURE/01-TECH-STACK.md · docs/TESTING/00-TESTING-STRATEGY.md |
| **Spec required** | no |

**Why:** PR-5 is the risk that a Slint window per Drape cannot meet the CPU and memory budgets; this card is where that risk is closed or escalated with numbers.

Profile CPU, memory, startup and GDI/USER handles; bound allocations, caches, queues and workers; icon cache; avoid allocations on hot paths. Re-measure the Slint memory/CPU footprint against PR-5 using the Phase 0 baseline. Compare supported renderers and measured LTO/codegen-unit/stripping/CRT profiles; Rust is already native-compiled. Record panic strategy and keep Guard independent.

**Definition of Done**
- [ ] combined App + Guard idle CPU <0.5% (machine-normalised), summed working set <100 MB with 20 x 20 after 30 min, startup median <1.5 s; private bytes/GPU/footprint also recorded per TESTING/00 with backend, build, DPI, monitors and machine
- [ ] no handle leak across a 1,000-cycle test (create/destroy Drapes, rollup, page switch)
- [ ] the PR-5 entry in the risk register is updated with measured results
- [ ] selected renderer/LTO/codegen/stripping/CRT/panic profiles have measured gains and supported-feature parity; actual shipped Release App + Guard pass recovery and functional smoke tests

**Abuse cases**
- Measuring with debugger attached, in Debug configuration, or with 1 Drape
- Reporting a smaller executable after stripping/feature changes while omitting renderer prerequisites, Guard, native drop/UIA or recovery failures

---

### P5-03 — Installer and updater

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 4 days |
| **Role** | release-engineer |
| **Depends on** | P4-06 |
| **Spec refs** | docs/DEVOPS/00-BUILD-AND-RELEASE.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md · ADR-017, ADR-022 in MEMORY/DECISIONS.md |
| **Spec required** | yes |

**Why:** Uninstall must restore owned icon changes. Update networking is permitted only by explicit manual/periodic opt-in and needs authenticated release/pair compatibility and rollback design.

Evaluate the existing Velopack preference for a native Rust App + Guard, including runtime footprint, supported Windows minimum, signing, rollback and uninstall restoration (Q-22). Keep GitHub Releases + per-user installer + portable zip + winget channels. If unsuitable, write a replacement ADR before changing tooling; verify installer silent switches and winget compatibility. Per-user install, no admin. Updates are **OFF by default** and opt-in (ADR-017); delta updates where supported. Uninstall restores desktop icons (runs the `--restore-desktop` path) and offers to delete user data.

**Definition of Done**
- [ ] installer and portable operation, update and uninstall succeed non-elevated on clean Windows 10 22H2 and Windows 11 VMs; exact builds and prerequisites recorded
- [ ] test compatible App + Guard pair staging, interrupted/failed update, pinned signer/trust-root rejection, tampered hash/metadata, downgrade/schema rollback and offline recovery
- [ ] no active recovery binary is removed/replaced before verified icon restoration; uninstall failure retains a usable restore path
- [ ] uninstall restores the desktop icons (verified with icons hidden before uninstall, including when the app is not running)
- [ ] a fresh install makes no network request until the user opts in to update checks (verified by capture)
- [ ] updates come only from the official channel and are signature/hash verified
- [ ] a winget manifest is validated (or the failure is recorded) and the installer is installable with it

**Abuse cases**
- Update check enabled by default "just to show the version" and described as opt-in in the docs
- Uninstaller that restores icons only when the app process is still running

---

### P5-04 — Code signing and checksums

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | release-engineer |
| **Depends on** | P5-03 |
| **Spec refs** | docs/DEVOPS/00-BUILD-AND-RELEASE.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md |
| **Spec required** | no |

**Why:** An unsigned binary that hides desktop icons and hooks the mouse looks exactly like malware to users and to antivirus.

Sign `drape-app.exe`, `drape-guard.exe` and the installer; publish `SHA256SUMS.txt`; keep keys safe in CI.

**Definition of Done**
- [ ] all artefacts carry a valid signature (verified by script on the released files, not the build output)
- [ ] no key or certificate in the repository, CI logs or artefacts (secret-scan clean)
- [ ] `SHA256SUMS.txt` matches the published files

**Abuse cases**
- Signing the build output but publishing the unsigned pre-signing copy
- Passing the certificate password on the command line where CI logs capture it

---

### P5-05 — Security and privacy audit

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | architect |
| **Depends on** | P4-06 |
| **Spec refs** | docs/SECURITY/00-SECURITY-AND-PRIVACY.md · docs/PLATFORM/00-WINDOWS-INTEGRATION.md · ADR-017 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** The app sees file names and (through a hook) mouse input; "we don't send anything" must be a tested property, not a sentence.

Review hooks, storage, logs, dependencies and update flow. Produce `SECURITY.md` (reporting policy) and a short privacy statement. Add the automated test that asserts **no network calls from the Shell and Core assemblies** (no references to `System.Net.*`, sockets or HTTP types; allowed only in the opt-in updater project).

**Definition of Done**
- [ ] the release security checklist in `docs/SECURITY/00-SECURITY-AND-PRIVACY.md` is complete, each item pointing at its evidence
- [ ] an automated test fails when `drape-core` or any `drape-shell-*` assembly references a network API (mutation check: add a reference, watch it fail)
- [ ] `SECURITY.md` and the privacy statement exist and match actual behaviour (logs, config, update opt-in)
- [ ] logs and diagnostics contain no user paths or file names at default level; dependency vulnerability scan is clean or each finding is triaged

**Abuse cases**
- A network-call test that scans only for `HttpClient` by name and misses sockets or `WebClient`
- A privacy statement written from intent rather than from a read of what the code logs and stores

---

### P5-06 — Legal and asset audit

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 2 days |
| **Role** | orchestrator |
| **Depends on** | P4-06 |
| **Spec refs** | docs/PLAN/05-LEGAL-AND-NAMING.md · ADR-019, ADR-020 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** A release carries a name, a licence and third-party code; getting any of them wrong after publication is hard to undo.

Trademark search for "Drape" (chosen by the owner, ADR-019, but never searched), audit of assets and UI text, `THIRD_PARTY_NOTICES.md`, and a check of the MIT `LICENSE` against dependency licences. If the trademark search fails, raise it with the owner before anything else.

**Definition of Done**
- [ ] the pre-public-release checklist in `docs/PLAN/05-LEGAL-AND-NAMING.md` is complete
- [ ] trademark search for "Drape" done in the target regions and its result recorded (name per ADR-019)
- [ ] `LICENSE` is MIT and every dependency licence is compatible
- [ ] `THIRD_PARTY_NOTICES.md` lists every shipped dependency with licence, generated from the lock/package data and not by hand
- [ ] no third-party product name, icon or asset in the shipped UI or package

**Abuse cases**
- Reporting the name as "cleared" because the owner chose it
- A notices file written by hand from memory that omits transitive dependencies

---

### P5-07 — Full-environment testing and regression

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 4 days |
| **Role** | qa |
| **Depends on** | P5-02 |
| **Spec refs** | docs/TESTING/00-TESTING-STRATEGY.md |
| **Spec required** | no |

**Why:** "Works on my machine" is the failure this card exists to prevent.

OS x DPI x monitor x theme x OneDrive matrix; risky scenarios (Explorer restart in a VM, crash recovery, sleep/resume, monitor hot-plug); upgrade from alpha/beta builds.

**Definition of Done**
- [ ] every matrix row and checklist item is ticked or recorded as a known limitation with an owner
- [ ] no open blocker/major bug
- [ ] Explorer-restart scenarios were run manually in a VM or test account (never scripted against the developer's Explorer)
- [ ] each result names the build, DPI and monitor setup

**Abuse cases**
- Marking rows "pass" that were only run on one DPI
- Quietly narrowing the matrix to what passes

---

### P5-08 — User documentation

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 3 days |
| **Role** | docs-keeper |
| **Depends on** | P4-03 |
| **Spec refs** | docs/UI-UX/00-UI-UX-SPEC.md · docs/PLATFORM/00-WINDOWS-INTEGRATION.md · docs/SECURITY/00-SECURITY-AND-PRIVACY.md |
| **Spec required** | no |

**Why:** The most important page is the one that tells a stranded user how to get their icons back.

Quick start, FAQ (including manual desktop-icon recovery, UIPI limits, antivirus notes) and feature pages.

**Definition of Done**
- [ ] a new user can create a first Drape using only the guide (tested with someone who has not seen the app)
- [ ] the FAQ contains the manual icon-recovery steps and they were followed once on a VM to confirm they work
- [ ] documentation exists in id-ID and en-US consistent with P5-01

**Abuse cases**
- Recovery steps copied from the design docs and never executed
- Screenshots showing paths or the developer's real desktop

---

### P5-09 — Limited public beta and fixes

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P1 |
| **Estimate** | 5 days |
| **Role** | orchestrator |
| **Depends on** | P5-04, P5-07 |
| **Spec refs** | docs/DEVOPS/00-BUILD-AND-RELEASE.md · docs/TESTING/00-TESTING-STRATEGY.md |
| **Spec required** | no |

**Why:** Real desktops are more varied than any matrix; the beta is how the unknowns surface before v1.0.

Release a `-beta` build, collect reports, triage, fix blockers/majors.

**Definition of Done**
- [ ] bug-report flow works (issue template asks for build, DPI, monitors; asks for no personal paths)
- [ ] every blocker/major from the beta is closed or scheduled with an owner and a date
- [ ] beta builds are signed (P5-04) and carry opt-in-only update checks

**Abuse cases**
- Closing reports as "cannot reproduce" without asking for the missing environment details
- Beta builds with debug logging that records user file names

---

### P5-10 — Release v1.0 (Windows only)

| | |
|---|---|
| **Status** | TODO |
| **Priority** | P0 |
| **Estimate** | 1 day |
| **Role** | release-engineer |
| **Depends on** | P5-05, P5-06, P5-08, P5-09 |
| **Spec refs** | docs/DEVOPS/00-BUILD-AND-RELEASE.md (release checklist) · ADR-014 in MEMORY/DECISIONS.md |
| **Spec required** | no |

**Why:** v1.0 is a Windows release (ADR-014); macOS, Linux and mobile are Phase 6 and must not appear in release notes as promises.

Tag `v1.0.0`, signed artefacts, release notes, changelog, archived test results.

**Definition of Done**
- [ ] the release checklist in `docs/DEVOPS/00-BUILD-AND-RELEASE.md` is complete
- [ ] release notes and README state Windows only; no claim of macOS/Linux/mobile support
- [ ] a post-release incident recovery plan is ready (rollback, advisory, hotfix path)
- [ ] the tagged commit builds reproducibly from clean with `-warnaserror`, tests green

**Abuse cases**
- Ticking the checklist before P5-05/P5-06 produce their evidence
- Marketing text that mentions "cross-platform" as a current feature
