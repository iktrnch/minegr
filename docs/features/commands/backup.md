---
type: feature
status: unimplemented
created: 2026-08-24
---

# Backup command

`minegr backup` creates a ZIP of world folders for a stopped or daemon-managed server.

Print `Creating backup…`, then `Backup created: <path>`. Use local time in `backups/backup-DD-MM-YYYY-HH-MM-SS±HHMM.zip`. Never overwrite or automatically delete backups.

## Workflow

1. Acquire the operating-system lock on `.minegr/locks/backup.lock`.
2. Read `level-name` from `server.properties`, defaulting to `world`. Include that folder and matching `_nether` and `_the_end` folders.
3. Validate paths and require free space for the input size plus 10%.
4. For a live server, acquire the daemon's save barrier; otherwise reject active world `session.lock` files.
5. Create a temporary ZIP and publish it atomically.

The CLI validates and archives; the daemon owns live-save coordination. Drain earlier Console inputs, run `save-off` and `save-all flush` with a five-minute timeout, then pause queue consumption during archiving. Always restore `save-on` before resuming input.

## Rules

- Allow only one backup per server.
- Follow [operation coordination](../../architecture/operation-coordination.md) for backup, restart, stop, and disconnect ordering.
- Preserve world directory names at ZIP root; exclude `session.lock`. Include neither configuration nor server software.
- Keep symlink targets inside the server root, `backups/` owner-only, and archives mode `0600`.
- Reject unmanaged running servers, invalid paths, insufficient space, duplicate archive names, and failed or timed-out saves.
- Failure, interruption, or client disconnect removes the temporary ZIP and restores saving before releasing queued input.

## Related
- [Daemon](../daemon.md)
- [Operation coordination](../../architecture/operation-coordination.md)
- [Filesystem layout](../../architecture/filesystem-layout.md)

