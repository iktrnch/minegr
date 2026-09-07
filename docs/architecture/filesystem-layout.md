---
type: architecture
status: current
created: 2026-09-07
---

# Filesystem layout

## Purpose

Define the files Minegr owns and the boundary between declarative configuration, generated files, and mutable Minecraft state.

## Responsibilities

- Give managed files stable locations.
- Keep Minecraft's normal root layout usable by external tools.
- Constrain paths, permissions, temporary files, and atomic publication.

## Structure

```text
<server-root>/
├── minegr.toml              # selected config; this is the default name
├── server.jar
├── server.properties
├── eula.txt
├── backups/
├── logs/                    # Minecraft-owned
├── <world directories>/    # mutable state
└── .minegr/
    ├── locks/backup.lock
    └── logs/                # daemon tracing logs
```

The canonical parent of the selected `minegr.toml` is `<server-root>`. Java runs with that directory as its working directory. Minegr supplies `./server.jar`; Minecraft retains its conventional paths for worlds, configuration, and logs.

Downloads use a unique owner-only system temporary directory. Files requiring atomic publication are copied to a temporary file beside their destination, flushed, and renamed.

## Data and control flow

`init` materializes missing `server.jar`, `server.properties`, and `eula.txt`. It refuses to overwrite a differing managed file. `sync` reads `server.properties` into `minegr.toml`. Minecraft owns mutable data and its `logs/` directory. The daemon owns `.minegr/logs/` and the backup lock.

Backups are published under `backups/` as `backup-DD-MM-YYYY-HH-MM-SS±HHMM.zip`. A collision fails instead of overwriting or adding a suffix.

## Interfaces

### Inputs

- One canonical configuration path.
- Managed relative paths and Minecraft-created state.

### Outputs

- Stable locations used by commands, Java, and external tools.

## Invariants

- Managed paths may not traverse or resolve outside the canonical server root.
- Symlinks included in backups must resolve within the server root.
- `minegr.toml`, backup archives, runtime sockets, and Minegr-private files are owner-only.
- `.minegr/locks/backup.lock` is an advisory lock location, not proof that a backup is active; the operating-system lock is authoritative.
- Temporary and partial artifacts never appear at a final managed path.
- Mutable state is never removed or overwritten by `init` or `sync`.

## Related

- [Configuration](configuration.md)
- [Artifact acquisition](artifact-acquisition.md)
- [Logging](logging.md)
- [Backup command](../features/commands/backup.md)
