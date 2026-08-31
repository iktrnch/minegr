---
type: proposal
status: proposed
created: 2026-08-31
---

# Cross-version stop

## Problem

Every Command currently requires an exact Minegr-version match. If Minegr is updated while an older daemon is running, the new CLI cannot ask that daemon to stop and the user may need to terminate the daemon and Java process manually.

## Proposal

Consider a small, frozen control protocol that lets future clients stop older daemons without making the normal IPC protocol backwards compatible.

## Motivation

Cross-version stop would make upgrades safer once Minegr is released and daemon processes commonly survive binary replacement. It is not required for the initial release and does not belong in the current protocol contract.

## Possible approach

Structurally separate a versioned stop-control exchange from normal handshakes and sessions. Its framing, identity fields, acceptance response, terminal result, and disconnect behaviour would form one compatibility boundary retained across releases.

## Alternatives

- Keep requiring manual process termination after an incompatible update.
- Make the complete IPC protocol backwards compatible.

## Open questions

- Which release first needs the compatibility commitment?
- Which identity and terminal fields can remain frozen indefinitely?
- How should a client distinguish accepted shutdown from daemon failure after disconnecting?

## Impact

Adopting this later would affect handshake routing, stop handling, protocol tests, shutdown completion, and upgrade documentation.

## Related

- [IPC protocol](../architecture/ipc-protocol.md)
- [Stop command](../features/commands/stop.md)
