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

Prints `Stopping server…`, submits the first stop request, and waits. After Java exits, output is drained, the daemon reports completion, removes its socket, and the client prints `Server stopped successfully.`

If graceful shutdown exceeds 60 seconds, the daemon sends `SIGTERM`, then `SIGKILL` after another 10 seconds. Successful forced termination prints a warning and still returns success.

## Workflow

1. Load the configuration and connect using a handshake that requires a matching Minegr version.
2. Stop accepting work, finish active backup recovery, and drain accepted Console inputs.
3. Cancel restart startup and stop Java.
4. Wait for Java termination, output draining, and socket cleanup.
5. Report the daemon's final result.

## Rules

- New Console inputs are rejected after stop is queued.
- Later stop clients observe shutdown in progress and wait for socket removal without joining it.
- `Ctrl+C` closes only the client; shutdown continues in the daemon.
- Java must not outlive the daemon.
- A stop during restart cancels pending startup.

## Failure cases

- No daemon is available: `Server is not running`.
- Peer, version, identity, or stop-message validation fails before shutdown is accepted.
- Java remains alive after escalation or daemon cleanup fails.

Failure prints `Failed to stop server: <reason>` and the last 100 daemon-session log lines to stderr.

## Implementation

The CLI submits one stop request and waits over the Unix socket. It does not attempt to stop a daemon using an incompatible protocol. The daemon owns queue ordering, signals, process reaping, and cleanup.

## Related

- [Daemon](../daemon.md)
- [Console command](console.md)
- [Start command](start.md)
