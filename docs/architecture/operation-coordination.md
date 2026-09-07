---
type: architecture
status: current
created: 2026-08-27
---

# Operation coordination

## Purpose

Define ordering, exclusion, cancellation, and shutdown across Console input, backup, restart, and stop.

## Responsibilities

- Preserve accepted Console input order.
- Make a live backup atomic from the daemon's perspective.
- Prevent backup and restart from overlapping unsafely.
- Give stop a terminal priority without losing already accepted work.
- Give every request an observable terminal response.

## Structure

The daemon owns one unbounded FIFO for accepted Console inputs. The operation coordinator gates consumption of that queue while exclusive lifecycle work runs. Backup is an atomic barrier. Restart is coordinated with that barrier. Stop is a priority lifecycle operation outside the Console queue.

Only one backup may hold the cross-process backup lock. Duplicate lifecycle requests perform no duplicate work and return `AlreadyInProgress` immediately.

## Data and control flow

### Backup

Earlier Console inputs drain before a live backup begins. The daemon runs `save-off`, then `save-all flush`, and pauses queue consumption while the CLI archives the world. It runs `save-on` before queue consumption resumes, including after failure or client disconnect.

A restart requested during an active backup waits for the complete backup. A backup requested while restart is queued or active fails with `RestartInProgress`. A second backup fails with `BackupInProgress`.

### Restart

Restart drains earlier Console inputs, stops Java, starts it again, then resumes the queue. Later Console inputs wait for readiness. Client disconnect does not cancel an accepted restart.

### Stop

Stop waits for an active backup to finish, rejects new work, drains accepted Console inputs, cancels pending restart startup, and shuts down Java. It is never enqueued as Console input. A second termination signal may cancel archiving, remove its temporary file, restore saving, and accelerate shutdown.

## Interfaces

### Inputs

- Accepted Console inputs.
- Backup, restart, and stop Messages.
- Client disconnects and process signals.

### Outputs

- Ordered Console input writes to Java.
- Lifecycle progress and terminal responses.
- Cancellation and busy responses.

## Invariants

- Acknowledged Console inputs are never silently discarded or retried automatically.
- Backup is atomic until the archive is published or its temporary file is removed and saving is restored.
- No Console input executes between `save-off` and `save-on`.
- Restart never overlaps an active backup.
- Once stop is accepted, no new work is accepted.
- Every duplicate request receives `AlreadyInProgress`; no client waits on an ignored request.
- Disconnecting a backup client cancels its archive and triggers recovery. Disconnecting a restart or stop client does not cancel daemon work.

## Related

- [Backup command](../features/commands/backup.md)
- [Restart command](../features/commands/restart.md)
- [Stop command](../features/commands/stop.md)
- [Console command](../features/commands/console.md)
- [Coordinate exclusive operations in the daemon](../decisions/0008-coordinate-exclusive-operations-in-the-daemon.md)
