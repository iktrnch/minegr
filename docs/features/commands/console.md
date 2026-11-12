---
type: feature
status: unimplemented
created: 2026-08-24
---

# Console command

`minegr console` opens a Ratatui console with unframed logs, a bordered single-line composer, and server name/state below it.

Load up to 10,000 current log lines and begin at the newest. Follow [logging](../../architecture/logging.md) for formatting and restart handling. State colors are green for `running`, yellow for `starting`/`stopping`, and red for `failed`/`stopped`.

## Controls

| Input | Action |
| --- | --- |
| `Enter` | Submit trimmed, non-empty input |
| `Up` / `Down` | Session input history |
| `PageUp` / `PageDown`, mouse wheel | Scroll logs |
| `Ctrl+End` | Resume live following |
| `Home` / `End` | Move within input |
| `Ctrl+C` | Detach without stopping Java |

## Behaviour

- Keep the composer focused; mouse input only scrolls logs.
- Pause following while scrolled up and show the unseen-line count.
- Support Unicode, paste, session-only history, soft-wrapped logs, and horizontally scrolling input. Show `Terminal too small` when needed; no completion.
- Multiple clients share the daemon's Console FIFO. Clear input only after acknowledgement; retain it on submission failure. Do not echo inputs or display queueing.
- Keep clients attached across restart, with input queued until readiness.
- On Java failure or final stop, preserve logs, show the state, disable input, and wait for restart where possible or `Ctrl+C`.

No daemon reports `Server is not running, start it with minegr start`.

Use one Tokio tunnel for `Logs`, `Status`, and `Console`, with the [IPC](../../architecture/ipc-protocol.md) input acknowledgement rules. Restore the terminal after normal exit or errors.

## Related
- [Daemon](../daemon.md)
- [Logs command](logs.md)
- [IPC protocol](../../architecture/ipc-protocol.md)
- [Logging](../../architecture/logging.md)

