# Parallel Rust continuation

Date: 2026-11-25. Roles: orchestrator/architect (root), release-engineer (ci), core-engineer (core), qa (exclusive validation owner).
Status: WIP; local continuation, no merge or DONE claim.

## Authority and ownership

The owner requested parallel implementation using several agents, with exactly one testing agent and a sequential testing queue. ADR-029 records the local dependency/branch policy. R0-03 continues existing CI work; R1-01 starts the domain/neutral-port subset. Existing dirty files and feat/R0-03-rust-ci are preserved. Root alone updates shared task/index/decision files. CI owns its workflow/helpers/deny policy; Core owns drape-core and drape-test-support. QA alone executes every build, format, lint, test, coverage, audit and documentation validation command.

## Architect interface handoff

Approved R1-01 subset: validated non-nil canonical GUIDs, lossless native path storage with adapter-supplied opaque comparison keys, validated page/item/Drape/config data, explicit timestamp-based reference-only duplicate repair, bounded owned icon pixels, synchronous enumeration/icon ports with operation/cancellation metadata, deterministic fakes. Existing Clock/capability interfaces remain compatible. Call-thread, capability, ownership and cancellation contracts must be documented; no blanket Send/Sync is imposed. Core does not normalize filesystem paths, obtain IDs/time from the environment, or implement native host/recovery. Invalid dimensions are rejected at construction; work-area clamp/layout remains R1-03.

Native host/recovery/FileSystem and other unimplemented ports remain explicit R1-01 acceptance gaps, pending appropriate specs/evidence. This subset cannot establish complete R1-01 parity or unlock dependent acceptance automatically.

## Review and evidence

QA source review found the R0-02 scaffold routes help/restore before renderer startup and fails unavailable recovery explicitly, without native mutation. Root static CI review found a sequential Windows job, pinned Rust/helper versions/action commits, explicit subprocess failures and isolated negative gate probes. Runtime evidence is recorded separately by the sole QA owner in the parallel QA record; no command is claimed passed here before its result.

## Not determined

No hosted CI run, signed release, native desktop/Guard recovery, GUI runtime, Windows matrix, UIA, performance or distribution measurement is implied. No commit or merge is requested.

## Validation ownership continuation

The original QA owner completed 18 targeted Core/test-support tests, seven CI gate tests, two expected tests-first RED phases and a Git-history secret scan. Its later agent turns had tool execution disabled before the integrated gates. The orchestrator replaced the inactive QA worker with qa_continued to continue the same sequential queue; there is exactly one active testing owner, no overlapping validation and no implementation-agent testing. The successful checks are retained rather than restarted.
