---
type: feature
status: unimplemented
created: 2026-08-24
---

# Logs command

`minegr logs` prints the current Minecraft log tail through the daemon, without a pager or TUI.

| Flag | Meaning |
| --- | --- |
| `--last <lines>` | Initial lines, `1..=10000`; default `1000` |
| `--follow` | Continue streaming after the tail |

Print lines once and in order using the shared [logging](../../architecture/logging.md) contract. `--follow` stays attached across restart and failed Java states, then closes successfully on final daemon stop. Empty history succeeds; followers wait for new lines.

Write only log lines to stdout and errors to stderr. Apply shared display formatting; disable color for redirected output or `NO_COLOR`. `Ctrl+C` detaches without stopping Java. A broken stdout pipe ends normally.

No daemon reports `Server daemon is not running, start it with minegr start`. Invalid line counts or handshakes fail. A slow subscriber reports `Log stream fell behind; reconnect to continue` and exits non-zero.

Use Tokio to stream daemon responses directly to stdout; the client never reads Minecraft log files itself.

## Related
- [Daemon](../daemon.md)
- [Console command](console.md)
- [IPC protocol](../../architecture/ipc-protocol.md)
- [Logging](../../architecture/logging.md)

