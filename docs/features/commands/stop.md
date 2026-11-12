---
type: feature
status: unimplemented
created: 2026-08-24
---

# Stop command

`minegr stop` gracefully stops Java and its daemon.

Print `Stopping server…`, submit the request, and wait for Java exit, output draining, and socket cleanup. Then print `Server stopped successfully.`

Allow 60 seconds for graceful shutdown, then send `SIGTERM` and `SIGKILL` after another 10 seconds. Successful forced termination warns and still succeeds.

The daemon follows [operation coordination](../../architecture/operation-coordination.md): finish an active backup, drain accepted input, cancel restart startup, and reject new work. Duplicate stop requests report `AlreadyInProgress` and succeed without joining shutdown. `Ctrl+C` detaches the client; shutdown continues.

No daemon reports `Server is not running`. The handshake requires matching version and identity. Failure prints `Failed to stop server: <reason>` and the last 100 available log lines to stderr. Java must not outlive the daemon.

## Related

- [Daemon](../daemon.md)
- [Console command](console.md)
- [Start command](start.md)
- [Operation coordination](../../architecture/operation-coordination.md)

