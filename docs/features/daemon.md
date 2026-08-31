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

Concurrent clients may use `start`, `stop`, `restart`, `status`, `logs`, and `console`. Multiple console clients share one Console input FIFO.
## Workflow
1. Bind `$XDG_RUNTIME_DIR/minegr/<uuid>.sock` and validate the client handshake.
2. Start Java and report readiness, failure, or cancellation.
3. Serve clients until final stop; replace Java in place during restart.
## Rules
- Every Command handshakes with the Minegr version, server UUID, and canonical configuration path. All Commands, including `stop`, require an exact Minegr-version match.
- The runtime directory and socket are accessible only to their Unix owner.
- The daemon and Java process are unique per server; Java must not outlive the daemon.
- Disconnecting the initiating client before readiness cancels startup. Later client disconnects do not stop Java.
- Restart validates the configuration before stopping Java, retains the socket, and requires the UUID and canonical path to remain unchanged. It shares the Console input queue; later inputs wait for readiness and duplicate restart clients join the same operation.
- Stop finishes active backup recovery, drains accepted Console inputs, cancels restart startup, waits 60 seconds for Java, then escalates to `SIGTERM` and `SIGKILL` after another 10 seconds. New work is rejected once stop is accepted.
- During shutdown the socket remains available only to report `DaemonStopping`; it is removed after Java exits and output drains.
- The daemon keeps only the latest 10,000 daemon-session lines in memory, including lines across restarts.
- Slow streaming clients are disconnected rather than allowed to delay the daemon.
- Restarting the server does not restart the daemon.
## Failure cases
- An active daemon, mismatched handshake, or invalid restart configuration is rejected without affecting Java.
- Startup failure returns the last 100 session lines and the Minecraft log path.
- Unexpected Java exit is reported as `failed`; the daemon then flushes output, removes its socket, and exits.
- Malformed requests or failed clients affect only that connection.
## Implementation
The CLI re-executes itself in a detached internal daemon mode. A versioned protocol over the Unix socket multiplexes control requests and log streams. The concurrency model is defined separately.
## Related
- [Start command](commands/start.md)
- [IPC protocol](../architecture/ipc-protocol.md)
- [Sync vs async](../decisions/0002-sync-vs-async.md)
