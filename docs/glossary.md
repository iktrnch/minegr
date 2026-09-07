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

## Instance

One server root and its declarative `minegr.toml`, generated managed files, mutable Minecraft state, and optional running daemon. Copying `minegr.toml` is sufficient to recreate its configured initial state, but not its mutable state.

## Managed file

A file Minegr can materialize from `minegr.toml`: initially `server.jar`, `server.properties`, and `eula.txt`. `init` creates a missing managed file but never overwrites a differing one.

## Mutable state

Minecraft data intentionally excluded from `minegr.toml`, including worlds, player data, bans, operators, whitelist entries, logs, crash reports, caches, and runtime metadata.

## Sync

The one-way capture of persisted Minecraft configuration into `minegr.toml`. Initially `minegr sync` replaces `[minecraft.properties]` from a stopped server's `server.properties` and changes no other section.

## Related

- [Commands](architecture/commands.md)
- [IPC protocol](architecture/ipc-protocol.md)
- [Configuration](architecture/configuration.md)
- [Filesystem layout](architecture/filesystem-layout.md)
- [Use one or two IPC tunnels](decisions/0005-use-one-or-two-ipc-tunnels.md)
