---
type: feature
status: unimplemented
created: 2026-08-24
---

# Daemon

`minegr start` re-executes the binary in a detached internal mode. One daemon owns one Java server and serves concurrent clients over its UUID's Unix socket. Java must not outlive it.

## Lifecycle

| State | Meaning |
| --- | --- |
| `starting` | Java launched; waiting for Minecraft's standard readiness line |
| `running` | Ready for use |
| `stopping` | Coordinated shutdown underway |
| `failed` | Java exited unexpectedly or restart startup failed; daemon remains available |
| `stopped` | No daemon |

Initial startup failure or unrecoverable daemon failure removes the daemon and socket. Unexpected Java exit after readiness or failed restart retains them for status, logs, restart, or final stop. Startup failure reports the last 100 available `latest.log` lines and its path.

Disconnecting the initiating start client before readiness cancels startup. Later client disconnects follow their command's semantics.

## Responsibilities

- Own Java launch, readiness, signals, reaping, and socket cleanup.
- Authenticate and route clients through the [IPC protocol](../architecture/ipc-protocol.md).
- Coordinate Console input, backup, restart, and stop through one [operation coordinator](../architecture/operation-coordination.md).
- Reload and validate restart configuration before stopping Java; keep the UUID, canonical path, daemon, and socket unchanged.
- Tail Minecraft's log and retain separate daemon diagnostics according to [logging](../architecture/logging.md).
- Isolate malformed or slow clients from other connections.

Shutdown allows 60 seconds for Java, then sends `SIGTERM` and, after 10 more seconds, `SIGKILL`. Daemon termination and host shutdown use the same coordinated stop path.

## Related
- [Start command](commands/start.md)
- [IPC protocol](../architecture/ipc-protocol.md)
- [Operation coordination](../architecture/operation-coordination.md)
- [Logging](../architecture/logging.md)
- [Sync vs async](../decisions/0002-sync-vs-async.md)

