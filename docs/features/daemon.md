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

Concurrent clients may use `start`, `stop`, `restart`, `status`, and `logs`. Only one `console` client may connect; read-only clients use `logs --follow`.
## Workflow
1. Bind `$XDG_RUNTIME_DIR/minegr/<uuid>.sock` and validate the client handshake.
2. Start Java and report readiness, failure, or cancellation.
3. Serve clients until Java stops, then remove the socket and exit.
## Rules
- The handshake verifies the protocol version, server UUID, and canonical configuration path.
- The runtime directory and socket are accessible only to their Unix owner.
- The daemon and Java process are unique per server; Java must not outlive the daemon.
- Disconnecting the initiating client before readiness cancels startup. Later client disconnects do not stop Java.
- Restart validates the configuration before stopping Java, retains the socket, and requires the UUID and canonical path to remain unchanged.
- Stop sends `stop`, waits 60 seconds, then escalates to `SIGTERM` and `SIGKILL` after a further 10 seconds.
- The daemon keeps the latest 10,000 current-session lines in memory. Historical logs come from Minecraft's `latest.log` and dated logs; minegr does not duplicate them.
- Slow streaming clients are disconnected rather than allowed to delay the daemon.
## Failure cases
- An active daemon, mismatched handshake, invalid restart configuration, or second console connection is rejected without affecting Java.
- Startup failure returns the last 100 session lines and the Minecraft log path.
- Unexpected Java exit is reported as `failed`; the daemon then flushes output, removes its socket, and exits.
- Malformed requests or failed clients affect only that connection.
## Implementation
The CLI re-executes itself in a detached internal daemon mode. A versioned protocol over the Unix socket multiplexes control requests and log streams. The concurrency model is defined separately.
## Related
- [start-command](start-command.md)
- [sockets](../architecture/sockets.md)
- [0002-sync-vs-async](../decisions/0002-sync-vs-async.md)
