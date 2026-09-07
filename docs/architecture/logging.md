---
type: architecture
status: current
created: 2026-09-07
---

# Logging

## Purpose

Define Minecraft log streaming and Minegr daemon diagnostics without duplicating Minecraft's persistent log files.

## Responsibilities

- Serve current Minecraft history and live lines through the daemon.
- Follow `logs/latest.log` safely across replacement and truncation.
- Keep daemon tracing separate from Minecraft output.

## Structure

Minecraft owns `<server-root>/logs/latest.log` and its normal rotations. The daemon tails that file and exposes it through the `Logs` protocol port. Minegr has no separate Minecraft log file and no daemon-session in-memory history buffer.

The daemon writes its own `tracing` diagnostics to `.minegr/logs/`, one file per daemon session. It retains the newest seven files. These files exclude Minecraft output, Console input contents, and secrets.

## Data and control flow

A history request reads up to the requested number of lines from the current `latest.log`, records the follow offset, and then emits appended lines without a gap. A following client remains connected across restart. If Java replaces or truncates `latest.log`, history resets to the new file and following continues from its beginning without replaying old content or injecting a synthetic log line.

If `latest.log` does not exist while Java is starting, a finite history request returns an empty result and a follower waits for the file to appear. A startup failure still reports the expected path even when no lines are available.

New `logs` requests after restart see only the new `latest.log`. Users inspect Minecraft's files directly after the daemon stops; `minegr logs` remains daemon-backed.

Display clients strip terminal control sequences, mute Minecraft timestamps, and apply terminal-only severity colors. Persistent Minecraft and tracing files are not rewritten for display.

## Interfaces

### Inputs

- Minecraft's `logs/latest.log`.
- Daemon lifecycle and protocol tracing events.

### Outputs

- Ordered `Logs` responses for current clients.
- Owner-only daemon tracing files.

## Invariants

- Minegr never persists a duplicate copy of Minecraft output.
- A slow subscriber is disconnected rather than allowed to delay the daemon.
- A log file replacement or truncation cannot replay content already delivered from that file.
- Tracing never records secrets or Console input contents.
- The daemon retains at most seven tracing session files.

## Related

- [IPC protocol](ipc-protocol.md)
- [Logs command](../features/commands/logs.md)
- [Console command](../features/commands/console.md)
- [Tail Minecraft latest.log](../decisions/0007-tail-minecraft-latest-log.md)
