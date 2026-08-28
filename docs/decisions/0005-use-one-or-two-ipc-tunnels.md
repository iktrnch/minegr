---
type: decision
status: accepted
created: 2026-08-28
supersedes: []
superseded_by: []
---

# Use one or two IPC tunnels
## Context
Minegr tunnels are bidirectional, but requests and streamed responses must be read and written concurrently without interleaving frames.
## Options considered
### Split one stream
Call `UnixStream::into_split()` and give the read and write halves separate owners.

**Advantages**
- Uses one connection and handshake per command.
- Needs no pairing identifier.
- Supports independent reads and writes directly in Tokio.

**Disadvantages**
- Both halves must coordinate shutdown.
- The write half still needs one owner when several tasks produce responses.
- A failed connection ends both directions.

### Pair two tunnels
The client opens separate request and response connections and pairs them during setup.

**Advantages**
- Each connection has one direction and one owner.
- Request and response backpressure are isolated.
- Either direction can close independently.

**Disadvantages**
- Requires a pairing identifier, restoring correlation state.
- Doubles connections, setup, handshakes, and cleanup.
- Partial connection failure leaves a half-open pair to recover.
- The client must open both connections because the daemon cannot connect back without a client-owned endpoint.

## Decision
Split each accepted `UnixStream` with `into_split()`. The read half receives requests. Response producers send typed `Message` values through a bounded `mpsc` channel to one task that serializes and exclusively owns the write half.

## Rationale
One connection already provides independent full-duplex I/O. Splitting it avoids paired-connection identity and cleanup, while the single writer preserves frame boundaries.

## Consequences
### Positive
- Each command needs one connection and handshake.
- Only one task serializes and writes frames.
- The bounded channel isolates response producers from socket writes.

### Negative
- Read and write tasks must coordinate tunnel shutdown.
- A failed stream ends both directions.
- Backpressure requires an explicit full-channel policy.

## Related
- [IPC protocol](../architecture/ipc-protocol.md)
- [Frame messages over Unix streams](0004-frame-messages-over-unix-streams.md)
- [Sync vs async](0002-sync-vs-async.md)
