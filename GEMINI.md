# GEMINI.md — Pointer for Gemini / Antigravity

This file is deliberately thin. The operating instructions for this repository are in:

- [`CLAUDE.md`](CLAUDE.md) — rules, traps, workflow, commands, document map
- [`AGENTS.md`](AGENTS.md) — roles, ownership, handoffs, escalation

They apply **equally to Gemini and Antigravity**. Read both fully at the start of every session.

Do not copy rules into this file. Duplicated rules drift, and an instruction that disagrees with the others produces confidently wrong work.

## The five rules that matter most

1. **Desktop safety is unwaivable.** Every path that hides icons has a restore on every exit path; `--restore-desktop` always works; never kill `explorer.exe` in scripts or tests.
2. **`Drape.Core` is platform-free.** Native code only in `Drape.Shell.*`.
3. **Capability gating.** Ask `IPlatformCapabilities`; never OS checks in UI or Core.
4. **Name the test.** "Tests passed" with no test named is not evidence.
5. **Record before `DONE`.** No task is done without its `MEMORY/records/` entry and `TASKS/PROGRESS.md` updated in the same commit.

## Language

Everything in the repository is English. Talk to the owner in the owner's language (Indonesian).
