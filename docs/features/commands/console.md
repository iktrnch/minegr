---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
---
# console-command

## Summary
`minegr console` opens an interactive TUI for viewing live server logs and sending Console inputs.
## Behaviour
The Ratatui interface follows the Codex CLI layout: unframed logs, a bordered composer, then the server name and state.
```text
 live server logs
 …
╭────────────────────────────────────────────╮
│ › Enter console input                      │
╰────────────────────────────────────────────╯
 server-name  ● running
```
It loads up to 10,000 daemon-session lines, starts at the newest line, and displays new logs live. The state uses green for `running`, yellow for `starting` or `stopping`, and red for `failed` or `stopped`.

## Workflow
1. Load the configuration and open one console tunnel carrying `Logs`, `Status`, and `Console` Messages.
2. Enter the alternate screen and render history, composer, and status.
3. Trim each submitted line and send non-empty Console inputs to the daemon.
4. Restore the terminal when the user presses `Ctrl+C`.
## Controls
| Input                                | Action                           |
| ------------------------------------ | -------------------------------- |
| `Enter`                              | Submit the input                 |
| `Up` / `Down`                        | Navigate session input history   |
| `PageUp` / `PageDown` or mouse wheel | Scroll logs                      |
| `Ctrl+End`                           | Resume live log following        |
| `Home` / `End`                       | Move within the input            |
| `Ctrl+C`                             | Close the client                 |
## Rules
- Multiple console clients may connect to one daemon.
- Console inputs from every client enter one FIFO in daemon-accepted order.
- Each console tunnel has a UUID and increasing Console input ID. Repeated or older IDs are acknowledged without being enqueued again.
- Console inputs are not echoed and daemon queueing is not shown.
- Clear input only after the daemon accepts it; accepted inputs execute once even if the client disconnects.
- Keep the console attached across restart and queue inputs until the server is running.
- When scrolled up, pause auto-follow and show the number of unseen lines.
- Keep the composer focused; mouse input only scrolls logs.
- Keep history in memory for this TUI session only. Support single-line editing, Unicode, and paste; do not provide completion.
- Strip terminal control sequences. Mute timestamps, show warnings in yellow and errors in red, and otherwise use terminal-default colors.
- Soft-wrap logs and horizontally scroll input. Show `Terminal too small` when the layout cannot fit.
- `Ctrl+C` closes only the client and never stops the server.
## Failure cases
- A stopped server reports `Server is not running, start it with minegr start`.
- If the server stops after connection, preserve final logs, show `stopped`, disable input, and wait for `Ctrl+C`.
- If Console input submission fails, retain the input.
## Implementation
Ratatui renders the interface while Tokio handles terminal events and one daemon tunnel split into read and write halves. That tunnel carries `Logs` and `Status` subscriptions alongside `Console` Messages. Terminal cleanup must run on normal exit and errors.
## Related
- [Daemon](../daemon.md)
- [Logs command](logs.md)
- [IPC protocol](../../architecture/ipc-protocol.md)
