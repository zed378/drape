# ARCHITECTURE

How Drape is structured, with what, and how its data is shaped.

| | Document | Covers |
|---|---|---|
| 00 | [System Architecture](./00-SYSTEM-ARCHITECTURE.md) | layers, dependency rules, components, interfaces, startup/shutdown/crash flows, threading, errors |
| 01 | [Tech Stack](./01-TECH-STACK.md) | technology choices and alternatives, Cargo candidates, native backend/renderer, licensing and future build gates |
| 02 | [Data Model](./02-DATA-MODEL.md) | entities, `config.json`, item identity, atomic writes, versioning, migration |
| 03 | [Rust Migration](./03-RUST-MIGRATION.md) | legacy/target boundaries, replacement tasks, parity and measurement policy |

Decisions behind these documents: [`MEMORY/DECISIONS.md`](../../MEMORY/DECISIONS.md) — ADR-005 (config storage), ADR-006 (icon strategy), ADR-007 (window per Drape and host layer), ADR-008 (single process + Guard), ADR-025 (Rust + Slint), ADR-015 (platform capabilities).
Platform-specific mechanics: [`../PLATFORM/`](../PLATFORM/README.md).
