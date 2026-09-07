---
type: decision
status: accepted
created: 2026-09-07
supersedes: []
superseded_by: []
---

# Tail Minecraft latest.log

## Context

`logs` and `console` require recent Minecraft output and live updates. Minecraft already persists that output in `logs/latest.log`, so a second Minegr-owned persistent copy or daemon-session buffer would duplicate data and define competing retention semantics.

## Options considered

### Tail `logs/latest.log`

The daemon reads history and follows appended lines from Minecraft's current log file.

**Advantages**

- Uses Minecraft's authoritative persistent output.
- Avoids a separate in-memory history buffer and duplicate file.
- Users can inspect logs with ordinary tools.

**Disadvantages**

- History resets when restart replaces or truncates the file.
- The follower must detect rotation and truncation.

### Keep daemon-session history

Capture Java output in a Minegr-owned buffer spanning Java restarts.

**Advantages**

- Preserves one continuous session history.
- Does not depend on Minecraft's file rotation.

**Disadvantages**

- Duplicates Minecraft output and retention behaviour.
- Requires bounded memory and stdout/stderr merge rules.

## Decision

The daemon tails `<server-root>/logs/latest.log` and serves it through the `Logs` protocol port. Minegr keeps no Minecraft-output history buffer and writes no duplicate Minecraft log file.

## Rationale

Minecraft already owns persistent log production. Reusing it keeps Minegr's responsibility limited to controlled access and live streaming.

## Consequences

### Positive

- One persistent source of Minecraft output exists.
- Minegr log clients and external tools observe the same file.

### Negative

- New history requests after restart cannot see the previous `latest.log` through Minegr.
- Connected followers need explicit file-replacement handling.

## Related

- [Logging](../architecture/logging.md)
- [Logs command](../features/commands/logs.md)
- [Console command](../features/commands/console.md)

