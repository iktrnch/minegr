---
type: proposal
status: proposed
created: 2026-08-31
---

# Cross-version stop

Deferred beyond the MVP. Exact-version handshakes mean a newly installed CLI cannot stop an older daemon; manual termination may be required.

A future frozen stop-control exchange could support upgrades without making all IPC backwards compatible. Its framing, identity, completion, and disconnect guarantees need a separate decision before adoption.

Current `stop` continues to require an exact Minegr-version match.

## Related

- [IPC protocol](../architecture/ipc-protocol.md)
- [Stop command](../features/commands/stop.md)

