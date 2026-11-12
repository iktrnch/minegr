---
type: glossary
status: current
created: 2026-08-31
---

# Glossary

| Term | Meaning |
| --- | --- |
| Command | A CLI invocation. Each daemon-backed Command owns one tunnel. |
| Tunnel | One full-duplex Unix socket connection for a Command session. |
| Message | A protocol request, response, lifecycle update, or subscription item. |
| Port | A logical Message route such as `Logs`, `Status`, or `Console`. |
| Console input | One complete line accepted into the daemon's shared FIFO. |
| Minegr version | Embedded package version used for exact handshake compatibility; no separate protocol version. |
| Instance | One configuration and server root, including managed files, mutable state, and an optional daemon. |
| Managed file | `server.jar`, `server.properties`, or `eula.txt`; `init` creates missing files without overwriting differing ones. |
| Mutable state | Minecraft data excluded from configuration, such as worlds, player data, access lists, and logs. |
| Sync | One-way replacement of TOML properties from a stopped server's `server.properties`. |

Copying configuration recreates the initial setup, not mutable state.

## Related

- [Commands](architecture/commands.md)
- [IPC protocol](architecture/ipc-protocol.md)
- [Configuration](architecture/configuration.md)
- [Filesystem layout](architecture/filesystem-layout.md)
- [Use one or two IPC tunnels](decisions/0005-use-one-or-two-ipc-tunnels.md)

