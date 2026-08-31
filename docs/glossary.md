---
type: glossary
status: current
created: 2026-08-31
---

# Glossary

## Command

A user-facing CLI invocation selected through Minegr's command-line parser, such as `minegr start` or `minegr console`. A Command that communicates with the daemon owns one tunnel and may exchange multiple Messages over it.

## Message

One unit of daemon-client protocol communication. Messages carry requests, responses, lifecycle updates, and subscription data over a Command's tunnel.

## Console input

One complete line submitted through a `Console` Message for execution by the Minecraft server. Multiple console clients contribute Console inputs to one daemon-owned FIFO.

## Minegr version

The package version embedded in the binary and carried in each daemon handshake. It currently identifies the wire schema, and Command tunnels require an exact Minegr-version match. Minegr has no separate protocol-version identifier.

## Related

- [Commands](architecture/commands.md)
- [IPC protocol](architecture/ipc-protocol.md)
- [Use one or two IPC tunnels](decisions/0005-use-one-or-two-ipc-tunnels.md)
