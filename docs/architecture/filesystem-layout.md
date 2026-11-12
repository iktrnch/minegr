---
type: architecture
status: current
created: 2026-09-07
---

# Filesystem layout

The selected configuration's canonical parent is the server root and Java's working directory.

| Path under server root | Owner and purpose |
| --- | --- |
| `minegr.toml` (default name) | Declarative instance configuration |
| `server.jar`, `server.properties`, `eula.txt` | Managed files materialized by `init` |
| World directories | Minecraft mutable state |
| `logs/` | Minecraft logs |
| `backups/` | World ZIP archives |
| `.minegr/locks/backup.lock` | Advisory backup lock |
| `.minegr/logs/` | Daemon diagnostics |

Minecraft keeps its conventional layout. `init` creates missing managed files without replacing differing files or mutable state. `sync` reads properties into configuration.

## File safety

- Keep managed paths inside the canonical root. Backup symlinks must resolve inside it.
- Configuration, backups, sockets, and Minegr-private files are owner-only.
- Download into a unique owner-only system temporary directory.
- Publish files through an adjacent temporary file, flush, and atomic rename; never expose partial output.
- The operating-system lock determines whether backup is active; the lock file may remain after release.

Backup names use local time: `backup-DD-MM-YYYY-HH-MM-SS±HHMM.zip`. Never overwrite an existing archive.

## Related

- [Configuration](configuration.md)
- [Artifact acquisition](artifact-acquisition.md)
- [Logging](logging.md)
- [Backup command](../features/commands/backup.md)

