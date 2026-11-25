# Drape tool policy for Antigravity

Read [CLAUDE.md](../../CLAUDE.md) and [AGENTS.md](../../AGENTS.md) for shared project instructions, roles and tool policy. [GEMINI.md](../../GEMINI.md) is the thin entry point.

- Prefix shell commands with rtk; use rtk proxy for unsupported commands/raw output.
- Prefer available codebase-memory graph tools for code discovery; use bounded fallback reads for uncovered code and documentation.
- Rust + Slint is the target; current work is documentation review, not implementation. Keep legacy C# evidence labelled.
- Use only tools/skills actually available. Saving a checkpoint does not manually compact the conversation. Preserve continuity in MEMORY.
- Follow the assigned spec and accessible, original product UI. Generic upstream-tool dashboards, savings tabs, sync/uninstall pickers and Token Saver footer/version requirements do not apply to Drape.

Avoid duplicating full operating rules here; update the linked canonical instructions instead.
