---
type: architecture
status: current
created: 2026-08-27
---

# Operation coordination

One daemon coordinator owns lifecycle ordering and one unbounded FIFO of accepted Console inputs. Acknowledgement means enqueueing; accepted input is never silently discarded or automatically retried.

| Operation | Ordering |
| --- | --- |
| Backup | Drain earlier inputs, run `save-off` and `save-all flush`, then pause input consumption while the CLI archives. Restore `save-on` before resuming. |
| Restart | Wait for backup, drain earlier inputs, stop and start Java, then resume queued input after readiness. |
| Stop | Reject new work, wait for backup, drain accepted inputs, cancel restart startup, then shut down Java. Stop stays outside the input queue. |

Only one backup holds the cross-process lock. Backup during queued or active restart fails with `RestartInProgress`; a second backup fails with `BackupInProgress`. Duplicate restart and stop requests return `AlreadyInProgress` without repeating or joining work.

A backup remains atomic until the archive is published or its temporary file is removed and saving restored. No Console input executes between `save-off` and `save-on`.

Disconnecting a backup client cancels archiving and restores saving. Accepted restart and stop continue after client disconnect. Daemon termination uses the same stop path; a second termination signal may cancel archiving, recover saving, and accelerate shutdown.

Each request receives progress as needed and a terminal result, including `CancelledByStop` for a cancelled restart.

## Related

- [Backup command](../features/commands/backup.md)
- [Restart command](../features/commands/restart.md)
- [Stop command](../features/commands/stop.md)
- [Console command](../features/commands/console.md)
- [Coordinate exclusive operations in the daemon](../decisions/0008-coordinate-exclusive-operations-in-the-daemon.md)

