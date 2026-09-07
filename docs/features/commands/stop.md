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
2. Stop accepting work, wait for an atomic active backup, and drain accepted Console inputs.
3. Cancel restart startup and stop Java.
4. Wait for Java termination, output draining, and socket cleanup.
5. Report the daemon's final result.

## Rules

- New Console inputs are rejected after stop is queued.
- Later stop clients interpret the daemon's `DaemonStopping` response as `AlreadyInProgress`, print a short message, and exit successfully without joining it.
- `Ctrl+C` closes only the client; shutdown continues in the daemon.
- Java must not outlive the daemon.
- A stop during restart cancels pending startup.
- Stop is a priority lifecycle operation outside the Console input queue.

## Failure cases

- No daemon is available: `Server is not running`.
- Peer, version, identity, or stop-message validation fails before shutdown is accepted.
- Java remains alive after escalation or daemon cleanup fails.

Failure prints `Failed to stop server: <reason>` and the last 100 available `latest.log` lines to stderr.

## Implementation

The CLI submits one stop request and waits over the Unix socket. It does not attempt to stop a daemon using an incompatible protocol. The daemon owns queue ordering, signals, process reaping, and cleanup.

## Related

- [Daemon](../daemon.md)
- [Console command](console.md)
- [Start command](start.md)
- [Operation coordination](../../architecture/operation-coordination.md)
