---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
related_docs: []
---
# stop-command

## Summary
`minegr stop` gracefully stops a running Minecraft server and its daemon.
## Behaviour

Prints `Stopping server…`, queues the server's `stop` command, and waits. After Java exits, output is drained, the daemon removes its socket, and the client prints `Server stopped successfully.`

If graceful shutdown exceeds 60 seconds, the daemon sends `SIGTERM`, then `SIGKILL` after another 10 seconds. Successful forced termination prints a warning and still returns success.

## Workflow

1. Load the configuration and connect to its daemon.
2. Queue `stop` after active backup work and accepted console commands.
3. Wait for Java termination, output draining, and socket cleanup.
4. Report the daemon's final result.

## Rules

- New console commands are rejected after stop is queued.
- A second stop client joins the shutdown already in progress.
- `Ctrl+C` closes only the client; shutdown continues in the daemon.
- Java must not outlive the daemon.

## Failure cases

- No daemon is available: `Server is not running`.
- Daemon handshake validation fails.
- Java remains alive after escalation or daemon cleanup fails.

Failure prints `Failed to stop server: <reason>` and the last 100 daemon-session log lines to stderr.

## Implementation

The CLI submits one stop request and waits over the Unix socket. The daemon owns queue ordering, signals, process reaping, and cleanup.

## Related

- [daemon](daemon.md)
- [console-command](console-command.md)
- [start-command](start-command.md)
