---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
related_docs: []
---
# logs-command

## Summary
`minegr logs` prints recent logs from the current daemon session.
### CLI flags
| Flag              | Description                                                  | Mandatory |
| ----------------- | ------------------------------------------------------------ | --------- |
| `--last <lines>`  | Initial line count. Defaults to `1000`; accepts `1..=10000`. | No        |
| `--follow`        | Continue printing new lines.                                 | No        |
## Behaviour
`minegr logs` prints the requested tail and exits. `minegr logs --follow` prints the same tail, then streams new lines until the server stops or the user presses `Ctrl+C`.

`--follow` may be combined with `--last`. It remains connected across `minegr restart`.
## Workflow
1. Load the configuration and connect to its daemon.
2. Request an atomic daemon-session tail, optionally followed by a subscription.
3. Print merged Java stdout and stderr lines in daemon-observed order.
## Rules
- Output only the current daemon session, including its restarts; never read `latest.log` or dated logs.
- Write only log lines to stdout. Write errors to stderr.
- Use colors only on a terminal and disable them for redirected output or `NO_COLOR`.
- Strip control sequences, mute timestamps, show warnings in yellow and errors in red, and otherwise use the terminal's default color.
- An empty normal request exits successfully; an empty followed request waits.
- Initial and followed lines appear once and in order without a subscription gap.
- `Ctrl+C` closes only the client. Server stop flushes remaining lines and exits successfully.
- Treat a broken stdout pipe as normal termination.
## Failure cases
- No daemon is available: `Server daemon is not running, start it with minegr start`.
- `--last` is outside `1..=10000`.
- Daemon handshake validation fails.
- A slow following client reports `Log stream fell behind; reconnect to continue` and exits non-zero.
## Implementation
The command is a Tokio socket client that writes streamed lines directly to stdout. It uses no TUI or pager.
## Related
- [daemon](daemon.md)
- [console-command](console-command.md)
- [sockets](../architecture/sockets.md)
