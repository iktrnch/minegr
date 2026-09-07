---
type: decision
status: accepted
created: 2026-09-07
supersedes: []
superseded_by: []
---

# Coordinate exclusive operations in the daemon

## Context

Console input, live backup, restart, and stop share one Java process. Their ordering must prevent input during a frozen save, overlapping backup and restart, or new work after shutdown begins.

## Options considered

### One daemon operation coordinator

Keep Console input ordering and lifecycle exclusion under one owner, with backup as an atomic barrier and stop as a priority operation.

**Advantages**

- Defines one authoritative order.
- Recovery and shutdown can reject new work consistently.
- Acknowledged Console input is not silently lost.

**Disadvantages**

- The coordinator owns several lifecycle states.
- The accepted Console queue is unbounded.

### Independent command locks

Let each command acquire only the locks it needs.

**Advantages**

- Each command owns less shared state.

**Disadvantages**

- Lock order and queue order can disagree.
- Stop and disconnect recovery become distributed across handlers.

## Decision

Use one daemon operation coordinator. Backup is atomic and pauses Console queue consumption. Restart waits for an active backup; backup fails while restart is queued or active. Stop remains outside the Console queue, waits for backup, rejects new work, drains accepted input, and cancels pending restart startup.

## Rationale

A single owner makes observable ordering and recovery explicit and prevents command-specific locks from creating contradictory results.

## Consequences

### Positive

- Backup, restart, and stop have deterministic precedence.
- Client disconnect recovery is centralized.
- Duplicate lifecycle requests receive a terminal response without duplicate work.

### Negative

- Coordinator correctness is critical to all lifecycle operations.
- An active atomic backup can delay restart and stop.

## Related

- [Operation coordination](../architecture/operation-coordination.md)
- [Backup command](../features/commands/backup.md)
- [Restart command](../features/commands/restart.md)
- [Stop command](../features/commands/stop.md)

