---
type: feature
status: unimplemented
created: 2026-08-24
---

# Restart command

`minegr restart` replaces Java while retaining its daemon, socket, console clients, and log followers.

1. Reload and validate configuration before touching the running server. UUID and canonical path must stay unchanged.
2. Wait for backup and drain earlier Console inputs.
3. Stop Java with the normal shutdown sequence, then wait up to five minutes for new startup readiness.
4. Resume queued input and report success.

Show stopping and starting progress with Indicatif. Completed lines remain visible:

```text
✔ Server stopped successfully.
✔ Server restarted successfully.
```

Plain output prints each phase and result once. Forced termination warns before startup continues.

Duplicate restart requests return `AlreadyInProgress` successfully. Backup during restart fails with `RestartInProgress`. Stop cancels pending startup with `CancelledByStop`. `Ctrl+C` detaches the client while restart continues.

No daemon reports `Server is not running`. Invalid configuration leaves Java untouched; shutdown failure prevents startup. Startup failure retains the daemon in `failed` for inspection or retry. Report `Failed to restart server: <reason>` and the last 100 available log lines to stderr.

## Related
- [Daemon](../daemon.md)
- [Start command](start.md)
- [Stop command](stop.md)
- [Operation coordination](../../architecture/operation-coordination.md)
- [Logging](../../architecture/logging.md)

