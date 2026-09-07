---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
related_docs: []
---
# logs-command

## Summary
`minegr logs` prints recent lines from the current Minecraft `logs/latest.log` through the daemon.
### CLI flags
| Flag              | Description                                                  | Mandatory |
| ----------------- | ------------------------------------------------------------ | --------- |
| `--last <lines>`  | Initial line count. Defaults to `1000`; accepts `1..=10000`. | No        |
| `--follow`        | Continue printing new lines.                                 | No        |
## Behaviour
`minegr logs` prints the requested tail and exits. `minegr logs --follow` prints the same tail, then streams new lines until final daemon stop or the user presses `Ctrl+C`. It remains attached while the daemon is `failed` so a later restart can resume the stream.

`--follow` may be combined with `--last`. It remains connected across `minegr restart`; history resets when Minecraft replaces or truncates `latest.log`.
## Workflow
1. Load the configuration and connect to its daemon.
2. Request an atomic tail of `logs/latest.log`, optionally followed by a subscription.
3. Print file lines in order while the daemon detects append, replacement, and truncation.
## Rules
- The daemon reads `latest.log`; the CLI never opens Minecraft log files directly.
- A new request after restart sees only the new `latest.log`. Existing followers continue from the beginning of the replacement without replay or a synthetic marker.
- Minegr keeps no separate Minecraft-output history buffer or persistent copy.
- Write only log lines to stdout. Write errors to stderr.
- Use colors only on a terminal and disable them for redirected output or `NO_COLOR`.
- Strip control sequences, mute timestamps, show warnings in yellow and errors in red, and otherwise use the terminal's default color.
- An empty normal request exits successfully; an empty followed request waits.
- Initial and followed lines appear once and in order without a subscription gap.
- `Ctrl+C` closes only the client. Server stop delivers lines already observed from the file and exits successfully.
- Treat a broken stdout pipe as normal termination.
## Failure cases
- No daemon is available: `Server daemon is not running, start it with minegr start`.
- `--last` is outside `1..=10000`.
- Daemon handshake validation fails.
- A slow following client reports `Log stream fell behind; reconnect to continue` and exits non-zero.
## Implementation
The command is a Tokio socket client that writes streamed lines directly to stdout. It uses no TUI or pager.
## Related
- [Daemon](../daemon.md)
- [Console command](console.md)
- [IPC protocol](../../architecture/ipc-protocol.md)
- [Logging](../../architecture/logging.md)
