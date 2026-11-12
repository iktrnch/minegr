---
type: feature
status: unimplemented
created: 2026-08-24
---

# Start command

`minegr start` launches the per-server daemon and waits up to five minutes for Minecraft readiness.

1. Load configuration and run pure and host validation.
2. Connect to the UUID socket, removing only a verified stale socket.
3. Launch the internal daemon and wait for readiness or failure.

A matching daemon reports `Server is already running, stop it with minegr stop`. The same UUID at another canonical path reports the conflict and suggests `minegr init --uuid`.

Print `Starting server…`, wait quietly for Minecraft's `Done!` readiness line, then print `Server started successfully.` Failure, timeout, or interruption stops Java; initial startup failure also removes the daemon and socket. Report the reason, last 100 available `latest.log` lines, and log path to stderr.

Missing managed files direct the user to rerun `init`; `start` never downloads or reconstructs them. A pinned Fabric launcher may perform its own bootstrap. Java must be available, compatible, and the only server process for this instance.

## Related
- [Init command](init.md)
- [Daemon](../daemon.md)
- [Configuration](../../architecture/configuration.md)
- [Validation](../../architecture/validation.md)

