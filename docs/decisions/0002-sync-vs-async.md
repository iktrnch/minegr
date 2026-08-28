---
type: decision
status: accepted
created: 2026-08-24
supersedes: []
superseded_by: []
---
# 0002-sync-vs-async

## Context
`minegr` [daemon](../features/daemon.md) requires communication with a client and keeps track of multiple tasks concurrently.

Most of such tasks are IO bound which may be designed as async event-driven or managed by using system threads. 
## Options considered
### Async runtime with tokio
Run socket, process, signal, and timer work as tasks on a Tokio runtime.

**Advantages**
- Many clients and streams do not require one OS thread each.
- Built-in async primitives support timeouts, cancellation, and process I/O.

**Disadvantages**
- Async code and runtime-aware libraries spread through daemon interfaces.
- Blocking work must be isolated from executor threads.

### Sync
Use blocking I/O and dedicated OS threads for concurrent work.

**Advantages**
- Control flow remains linear and uses standard blocking APIs.
- Thread stacks make debugging and failure traces direct.

**Disadvantages**
- Each long-lived client or stream consumes a thread and stack.
- Coordinated shutdown requires explicit cross-thread signalling and joins.

## Decision
Use Tokio for the daemon and its socket clients.

## Rationale
The daemon is primarily concurrent I/O: client connections, Java streams, signals, and timers. Tokio handles these through one runtime and provides consistent cancellation and timeout primitives. This fits the daemon better than coordinating dedicated threads.

## Consequences
### Positive
- Client streams and lifecycle tasks share one concurrency model.
- Shutdown, cancellation, and timeouts use Tokio primitives.
### Negative
- Daemon-facing interfaces become async.
- Blocking work must use dedicated blocking threads.

## Related
- [daemon](../features/daemon.md)
- [ipc-protocol](../architecture/ipc-protocol.md)
