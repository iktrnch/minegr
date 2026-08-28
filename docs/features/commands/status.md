---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
related_docs:
  - daemon.md
  - minegr-command.md
---
# status-command
## Summary
`minegr status` prints the server state and Java process usage reported by the
daemon.
## Behaviour
The command prints one snapshot and exits:
```text
Status: running
Memory usage: 2048 MiB
CPU usage: 125.4%
```
Supported states are `starting`, `running`, `stopping`, `failed`, and `stopped`.
A missing or unresponsive daemon prints only:
```text
Status: stopped
```
## Workflow
1. Load the configuration and connect to its daemon.
2. Request the current state and Java process metrics.
3. Print the snapshot and exit.
## Rules
- Show metrics whenever Java is alive, including while starting or stopping.
- Memory is resident memory (RSS), rounded to whole MiB.
- CPU is rounded to one decimal; `100%` equals one fully used core and may be exceeded.
- Status requests are read-only and bypass the console-command queue.
- Every reported state exits with code `0`.
## Failure cases
- A protocol or daemon identity mismatch fails with a non-zero exit.
- If a live process metric is unavailable, print `unavailable`, then report its error on stderr after the status block. The command still exits with code `0`:
```text
Status: running
Memory usage: unavailable
CPU usage: 125.4%

Error: failed to read memory usage: <reason>
```
- If both metrics are unavailable, report separate errors in memory-then-CPU order.
## Implementation
The daemon reads the Java process state, RSS, and CPU usage. The client formats the response without sending anything to the Minecraft console. The same IPC port provides a state-only subscription for the console TUI.
## Related
- [Daemon](../daemon.md)
- [Minegr command](minegr.md)
