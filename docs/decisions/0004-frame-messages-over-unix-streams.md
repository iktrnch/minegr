---
type: decision
status: accepted
created: 2026-08-28
supersedes: []
superseded_by: []
---

# Frame messages over Unix streams
## Context
Minegr sends multiple serialized `Message` values through long-lived Unix stream connections. A stream preserves byte order but not message boundaries, so the protocol must identify where each message ends.

The framing method must support request/response exchanges and unbounded log or console streams. Serialization format is a separate decision.
## Options considered
### Length prefix
Prefix every payload with a fixed-width unsigned length.
**Advantages**
- Works with text and binary serialization.
- Reads exactly one bounded payload at a time.
- Has straightforward Tokio codec support.
**Disadvantages**
- Requires rejecting oversized lengths before allocation.
- A corrupt length desynchronizes the connection.
- Adds a small header to every message.
### Delimiter
Terminate every serialized message with a reserved byte sequence, such as a newline.
**Advantages**
- Simple to inspect and debug with text formats.
- Can use buffered line-oriented reads.
- Adds little framing overhead.

**Disadvantages**
- Payloads must escape or forbid the delimiter.
- Couples framing to serialization rules.
- Requires a size limit while searching for the delimiter.
### Incremental deserialization
Concatenate self-delimiting serialized values and let the decoder find each boundary.

**Advantages**
- Adds no separate framing bytes.
- Keeps boundaries inside the serialization format.

**Disadvantages**
- Requires a format that safely supports concatenated values.
- Distinguishing incomplete from invalid input is format-specific.
- Size limits and recovery are harder to implement.
### Unix sequenced-packet socket
Use `SOCK_SEQPACKET` so the transport preserves message boundaries.
**Advantages**
- Removes application-level boundary detection.
- Preserves reliable, ordered delivery.
- Keeps one packet aligned with one message.

**Disadvantages**
- Replaces the planned Unix stream transport.
- Has less direct Tokio and library support.
- Requires explicit packet-size handling.
## Decision
Prefix each serialized `Message` payload with a four-byte big-endian unsigned length. Reject zero-length payloads and payloads over 16 MiB.
## Rationale
Length-prefix framing works with any serialization format, maps directly to bounded reads, and fits Tokio's stream model without changing transports.
## Consequences
### Positive
- Framing stays independent of serialization.
- Receivers can reject oversized payloads before reading them.
### Negative
- Every message gains a small header.
- Invalid lengths terminate the connection because its boundaries can no longer be trusted.
## Related
- [IPC protocol](../architecture/ipc-protocol.md)
- [Daemon](../features/daemon.md)
- [Sync vs async](0002-sync-vs-async.md)
