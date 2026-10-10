# PLATFORM

Everything that depends on the operating system: how Drape sits on the desktop, hides native icons, hooks input, handles DPI — and which of that is possible elsewhere.

| | Document | Covers |
|---|---|---|
| 00 | [Windows Integration](./00-WINDOWS-INTEGRATION.md) | desktop window hierarchy, host strategies, hiding icons, enumeration, context menu, drag-drop, mouse hook, DPI, effects, events, spikes, risks |
| 01 | [Platform Support](./01-PLATFORM-SUPPORT.md) | capability matrix; macOS, Linux (X11/Wayland), Android and iOS notes; sequencing |

**Every technique in these documents is a hypothesis until a spike proves it on the target OS build.** Spike results are written as records in [`../../MEMORY/records/`](../../MEMORY/README.md) and, when they decide something, as ADRs in [`../../MEMORY/DECISIONS.md`](../../MEMORY/DECISIONS.md); this folder is then amended through the deviation protocol.

Document `00` covers Windows only; the code it describes lives only in `Drape.Shell.Windows` (and `Drape.Guard`).
