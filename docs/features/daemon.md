---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
related_docs: []
---
# daemon

## Summary
The daemon owns one running Minecraft server and provides the live control interface used by `minegr` commands.
## Behaviour
`minegr start` launches one detached daemon per server. The daemon starts Java, detects the standard Minecraft readiness line, and exposes `starting`, `running`, `stopping`, or `failed`. A missing daemon means `stopped`.

Concurrent clients may use `start`, `stop`, `restart`, `status`, `logs`, and `console`. Multiple console clients share one unbounded Console input FIFO.
## Workflow
1. Bind `$XDG_RUNTIME_DIR/minegr/<uuid>.sock` and validate the client handshake.
2. Start Java and report readiness, failure, or cancellation.
3. Serve clients until final stop; replace Java in place during restart.
## Rules
- Every Command handshakes with the Minegr version, server UUID, and canonical configuration path. All Commands, including `stop`, require an exact Minegr-version match.
- The runtime directory and socket are accessible only to their Unix owner.
- The daemon and Java process are unique per server; Java must not outlive the daemon.
- Disconnecting the initiating client before readiness cancels startup. Later client disconnects do not stop Java.
- Restart validates the configuration before stopping Java, retains the socket, and requires the UUID and canonical path to remain unchanged. It shares the Console input queue; later inputs wait for readiness and duplicate restart clients receive `AlreadyInProgress` without joining the operation.
- Stop waits for an active backup to complete, drains accepted Console inputs, cancels restart startup, waits 60 seconds for Java, then escalates to `SIGTERM` and `SIGKILL` after another 10 seconds. New work is rejected once stop is accepted.
- Daemon `SIGTERM` and host shutdown enter the same coordinated stop path. A second termination signal accelerates escalation after restoring backup safety.
- During shutdown the socket remains available only to report `DaemonStopping`; it is removed after Java exits and output drains.
- The daemon tails Minecraft's `logs/latest.log` for history and live clients. It keeps no Minecraft-output history buffer and follows file replacement or truncation across restart.
- Daemon `tracing` logs live under `.minegr/logs/`, retain the newest seven sessions, and exclude Minecraft output, Console input contents, and secrets.
- Slow streaming clients are disconnected rather than allowed to delay the daemon.
- Restarting the server does not restart the daemon.
## Failure cases
- An active daemon, mismatched handshake, or invalid restart configuration is rejected without affecting Java.
- Startup failure returns the last 100 available `latest.log` lines and the Minecraft log path, then removes the initial daemon and socket.
- Unexpected Java exit or restart startup failure enters `failed` while the daemon and socket remain available for status, logs, restart, or final stop.
- Malformed requests or failed clients affect only that connection.
## Implementation
The CLI re-executes itself in a detached internal daemon mode. A versioned protocol over the Unix socket multiplexes control requests and log streams. The concurrency model is defined separately.
## Related
- [Start command](commands/start.md)
- [IPC protocol](../architecture/ipc-protocol.md)
- [Operation coordination](../architecture/operation-coordination.md)
- [Logging](../architecture/logging.md)
- [Sync vs async](../decisions/0002-sync-vs-async.md)
