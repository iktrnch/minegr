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
2. Queue restart after active backup work and earlier commands.
3. Stop Java using the normal stop sequence while retaining the daemon and socket.
4. Start Java and wait up to five minutes for readiness.
5. Execute console commands queued behind restart and report success.
## Rules
- Commands submitted during restart remain queued until the server is running.
- Console and `logs --follow` clients remain connected.
- A second restart client joins the operation already in progress.
- `Ctrl+C` closes only the client; restart continues.
- Forced shutdown prints a warning and continues once Java exits.
- The daemon log buffer remains continuous across restarts.
## Failure cases
- No daemon is available: `Server is not running`.
- Invalid reloaded configuration leaves the running server untouched.
- Shutdown failure prevents startup.
- Startup failure cleans up Java and the daemon, leaving the server stopped.
Failure prints `Failed to restart server: <reason>` and the last 100 daemon-session lines to stderr.
## Implementation
The daemon composes its existing stop and start flows while retaining its socket. The client renders phase progress with Indicatif.
## Related
- [daemon](daemon.md)
- [start-command](start-command.md)
- [stop-command](stop-command.md)
