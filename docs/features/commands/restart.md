---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
related_docs: []
---
# restart-command
## Summary
`minegr restart` replaces the running Java process without replacing its daemon.
## Behaviour
Indicatif displays `Stopping server…` and `Starting server…` as active spinners, then leaves each completed line visible:
```text
✔ Server stopped successfully.
✔ Server restarted successfully.
```
Redirected output disables animation and prints `Stopping server…`, `Server stopped successfully.`, `Starting server…`, and `Server restarted successfully.` once each.
## Workflow
1. Reload and validate the configuration before showing progress.
2. Wait for an active backup and drain earlier Console inputs.
3. Stop Java using the normal stop sequence while retaining the daemon and socket.
4. Start Java and wait up to five minutes for readiness.
5. Execute Console inputs queued behind restart and report success.
## Rules
- Console inputs submitted during restart remain queued until the server is running.
- Console and `logs --follow` clients remain connected.
- A second restart client receives `AlreadyInProgress` and exits successfully without joining the operation.
- A backup requested while restart is queued or active fails with `RestartInProgress`.
- A stop request cancels pending startup and the restart client reports `CancelledByStop`.
- `Ctrl+C` closes only the client; restart continues.
- Forced shutdown prints a warning and continues once Java exits.
- Existing log followers remain connected, but new history resets to the replacement `logs/latest.log`.
## Failure cases
- No daemon is available: `Server is not running`.
- Invalid reloaded configuration leaves the running server untouched.
- Shutdown failure prevents startup.
- Stop cancels the restart.
- Startup failure leaves the daemon and socket alive in `failed` for inspection or retry.
Failure prints `Failed to restart server: <reason>` and the last 100 available `latest.log` lines to stderr.
## Implementation
The daemon composes its existing stop and start flows while retaining its socket. The client renders phase progress with Indicatif.
## Related
- [Daemon](../daemon.md)
- [Start command](start.md)
- [Stop command](stop.md)
- [Operation coordination](../../architecture/operation-coordination.md)
- [Logging](../../architecture/logging.md)
