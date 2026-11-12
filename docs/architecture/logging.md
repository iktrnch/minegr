---
type: architecture
status: current
created: 2026-09-07
---

# Logging

Minecraft owns `logs/latest.log` and its rotations. The daemon tails that file through the `Logs` port; Minegr keeps neither a duplicate Minecraft log file nor an in-memory history buffer.

## History and following

Read the requested tail and continue from its offset without a subscription gap or duplicate lines. Followers remain attached across restart and follow replacement or truncation from the new file's beginning. New requests see only the current file.

If the file is not yet present, finite history is empty and followers wait. After the daemon stops, users inspect Minecraft's files directly.

Display clients strip terminal controls, mute timestamps, and color warnings yellow and errors red only where terminal output allows it. Display formatting never rewrites stored files. Disconnect slow subscribers so they cannot delay the daemon.

## Daemon diagnostics

Write `tracing` diagnostics under `.minegr/logs/`, one owner-only file per daemon session, retaining the newest seven. Exclude Minecraft output, Console input contents, and secrets.

## Related

- [IPC protocol](ipc-protocol.md)
- [Logs command](../features/commands/logs.md)
- [Console command](../features/commands/console.md)
- [Tail Minecraft latest.log](../decisions/0007-tail-minecraft-latest-log.md)

