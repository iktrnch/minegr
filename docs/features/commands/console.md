---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
---
# console-command

## Summary
`minegr console` opens an interactive TUI for viewing live server logs and sending console commands.
## Behaviour
The Ratatui interface follows the Codex CLI layout: unframed logs, a bordered composer, then the server name and state.
```text
 live server logs
 …
╭────────────────────────────────────────────╮
│ › Enter server command                     │
╰────────────────────────────────────────────╯
 server-name  ● running
```
It loads up to 10,000 daemon-session lines, starts at the newest line, and displays new logs live. The state uses green for `running`, yellow for `starting` or `stopping`, and red for `failed` or `stopped`.

## Workflow
1. Load the configuration and acquire the daemon's single console connection.
2. Enter the alternate screen and render history, composer, and status.
3. Trim each submitted line and send non-empty commands to the daemon.
4. Restore the terminal when the user presses `Ctrl+C`.
## Controls
| Input                                | Action                           |
| ------------------------------------ | -------------------------------- |
| `Enter`                              | Submit the command               |
| `Up` / `Down`                        | Navigate session command history |
| `PageUp` / `PageDown` or mouse wheel | Scroll logs                      |
| `Ctrl+End`                           | Resume live log following        |
| `Home` / `End`                       | Move within the input            |
| `Ctrl+C`                             | Close the client                 |
## Rules
- Only one console client may connect per daemon.
- Commands are not echoed and daemon queueing is not shown.
- Clear input only after the daemon accepts the command; accepted commands execute once even if the client disconnects.
- Keep the console attached across restart and queue commands until the server is running.
- When scrolled up, pause auto-follow and show the number of unseen lines.
- Keep the composer focused; mouse input only scrolls logs.
- Keep history in memory for this TUI session only. Support single-line editing, Unicode, and paste; do not provide completion.
- Strip terminal control sequences. Mute timestamps, show warnings in yellow and errors in red, and otherwise use terminal-default colors.
- Soft-wrap logs and horizontally scroll input. Show `Terminal too small` when the layout cannot fit.
- `Ctrl+C` closes only the client and never stops the server.
## Failure cases
- A stopped server reports `Server is not running, start it with minegr start`.
- A second client reports `Console is already attached`.
- If the server stops after connection, preserve final logs, show `stopped`, disable input, and wait for `Ctrl+C`.
- If command submission fails, retain the input.
## Implementation
Ratatui renders the interface while Tokio handles terminal events and the daemon's live socket stream. Terminal cleanup must run on normal exit and errors.
## Related
- [daemon](daemon.md)
- [logs-command](logs-command.md)
- [sockets](../architecture/sockets.md)
