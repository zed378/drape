# Product Documentation — Drape (Desktop Organizer)

Reference documentation for **Drape**: a desktop organizer that groups desktop icons, shortcuts and files into semi-transparent panels ("Drapes") drawn above the wallpaper.

**Status: planning phase, no code yet.** Until code exists these documents describe the *intended* system. From the first milestone on, `docs/` becomes **as-built** reference: every fact must name the file, spike result or ADR it comes from. A document that disagrees with the code produces confidently wrong work, and the confidence is the dangerous part — if you find a claim the code contradicts, the code wins and correcting the document is part of the fix.

The execution plan lives in [`../TASKS/`](../TASKS/README.md). The record of what was built, decided and learned lives in [`../MEMORY/`](../MEMORY/README.md). `docs/` is reference material: it is amended deliberately through the **deviation protocol** in [`../TASKS/00-TASK-CONVENTIONS.md`](../TASKS/00-TASK-CONVENTIONS.md), never as a side effect of implementation.

## Recommended Reading Order

```
1.  PLAN/00-PROJECT-OVERVIEW.md            → vision, scope, non-goals
2.  PLAN/01-PRODUCT-REQUIREMENTS.md       → personas, user stories, FR/NFR
3.  PLAN/02-FEATURE-SPECIFICATION.md      → behaviour of every feature
4.  ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md
5.  ARCHITECTURE/01-TECH-STACK.md  →  02-DATA-MODEL.md
6.  PLATFORM/00-WINDOWS-INTEGRATION.md    → MANDATORY before touching any Shell code (highest-risk area)
7.  PLATFORM/01-PLATFORM-SUPPORT.md       → what is and is not possible on macOS, Linux, Android, iOS
8.  UI-UX/00-UI-UX-SPEC.md                → before touching any UI
9.  ENGINEERING/ → TESTING/ → DEVOPS/     → working standards, tests, release
10. SECURITY/00-SECURITY-AND-PRIVACY.md   → before touching hooks, config or the updater
```

## Folder Structure

```
PLAN/           7 files — vision, requirements, features, roadmap, risk register, legal, glossary
ARCHITECTURE/   4 files — layers, lifecycle, tech stack, data model
PLATFORM/       3 files — Windows (Win32) integration; per-platform support matrix
UI-UX/          1 file  — anatomy, states, interactions, accessibility
ENGINEERING/    1 file  — how code is written, layered, named, reviewed
TESTING/        1 file  — pyramid, key tests, manual matrix, performance
DEVOPS/         1 file  — build, CI, release, signing, rollback
SECURITY/       1 file  — threat model, privacy, hook rules
```

Architecture Decision Records are **not** in `docs/`: they live in [`../MEMORY/DECISIONS.md`](../MEMORY/DECISIONS.md), as in every sibling project that uses this layout.

## Document Index

| Document | Contents | Primary owner role |
|---|---|---|
| [PLAN/00](./PLAN/00-PROJECT-OVERVIEW.md) | Vision, problem, principles, scope, non-goals, success metrics | orchestrator |
| [PLAN/01](./PLAN/01-PRODUCT-REQUIREMENTS.md) | Personas, user stories, FR, NFR, constraints, MVP acceptance criteria | orchestrator |
| [PLAN/02](./PLAN/02-FEATURE-SPECIFICATION.md) | Feature matrix and detailed behaviour (F-01 … F-27) | orchestrator |
| [PLAN/03](./PLAN/03-IMPLEMENTATION-ROADMAP.md) | Phases, estimates, exit criteria, dependencies | orchestrator |
| [PLAN/04](./PLAN/04-RISK-REGISTER.md) | Project and platform risks (`PR-n`) | orchestrator |
| [PLAN/05](./PLAN/05-LEGAL-AND-NAMING.md) | Trademarks, assets, licences | orchestrator |
| [PLAN/06](./PLAN/06-GLOSSARY.md) | Product and technical terms | docs-keeper |
| [ARCHITECTURE/00](./ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md) | Layers, components, flows, threading, errors | architect |
| [ARCHITECTURE/01](./ARCHITECTURE/01-TECH-STACK.md) | Technology choices, packages, solution setup | architect |
| [ARCHITECTURE/02](./ARCHITECTURE/02-DATA-MODEL.md) | Entities, config schema, versioning, migration | architect |
| [PLATFORM/00](./PLATFORM/00-WINDOWS-INTEGRATION.md) | Desktop hierarchy, host strategies, icons, hooks, DPI, risks | shell-engineer-windows |
| [PLATFORM/01](./PLATFORM/01-PLATFORM-SUPPORT.md) | Capability matrix and per-platform notes (macOS, Linux, Android, iOS) | architect |
| [UI-UX/00](./UI-UX/00-UI-UX-SPEC.md) | Drape anatomy, states, interaction, menus, accessibility | ui-engineer |
| [ENGINEERING/00](./ENGINEERING/00-CODING-STANDARDS.md) | C# conventions, boundaries, interop, git, PRs | architect |
| [TESTING/00](./TESTING/00-TESTING-STRATEGY.md) | Test pyramid, mandatory tests, manual matrix, performance | qa |
| [DEVOPS/00](./DEVOPS/00-BUILD-AND-RELEASE.md) | Build, CI, versioning, installer, signing | release-engineer |
| [SECURITY/00](./SECURITY/00-SECURITY-AND-PRIVACY.md) | Threat model, privacy, release checklist | architect |

## Rules for These Documents

1. **Ground every claim.** Once code exists, name the file. Before that, name the spike, ADR or backlog item that will settle it. A claim marked **(assumption)** or **(hypothesis)** is waiting for confirmation — see [`../TASKS/BACKLOG.md`](../TASKS/BACKLOG.md).
2. **Record contradictions rather than smoothing them.** If the platform does something surprising, say so plainly; the surprise is the useful part.
3. **Amend, do not rewrite.** A `docs/` change is an event: it means reality taught the specification something. Follow the deviation protocol and write the record.
4. **Product behaviour changes update [`PLAN/02`](./PLAN/02-FEATURE-SPECIFICATION.md)** and, where needed, PLAN/01.
5. **New terms go in [`PLAN/06`](./PLAN/06-GLOSSARY.md).**
6. **Links are relative.** Check them before merging; a link to a path that does not exist is a broken link, not a hidden document.
7. **Language.** All documentation is English. User-visible strings ship in id-ID and en-US (NFR-08); example strings in the UI spec show both.
