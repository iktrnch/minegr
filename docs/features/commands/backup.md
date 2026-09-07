---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
---
# backup-command

## Summary

`minegr backup` creates a ZIP archive of the server's world folders while the server is stopped or running through minegr.
## Behaviour
Prints `Creating backup…`, creates `backups/backup-DD-MM-YYYY-HH-MM-SS±HHMM.zip`, then prints `Backup created: <path>`.

The timestamp uses local time. Existing backups are never overwritten or removed automatically.
## Workflow
1. Resolve the server root and acquire the operating-system lock on `.minegr/locks/backup.lock`.
2. Read `level-name` from `server.properties`, defaulting to `world`. Include that folder and any matching `_nether` and `_the_end` folders.
3. Validate paths, reject a duplicate filename, and require free space equal to the included data plus 10%.
4. If the daemon is running, request a live snapshot. Otherwise, verify that no world `session.lock` is active.
5. Create a temporary ZIP, rename it atomically, and report its path.

For a live snapshot, earlier Console inputs drain before the daemon runs `save-off` and `save-all flush`, waiting up to five minutes. The operation is then atomic from the daemon's perspective and Console queue consumption pauses. The daemon runs `save-on` before executing later queued input, including after client disconnect or failure.
## Rules
- Only one backup may run per server.
- A restart requested during backup waits for it. A backup requested while restart is queued or active fails with `RestartInProgress`.
- Stop waits for the atomic backup to finish unless a second termination signal cancels archiving and triggers recovery.
- Preserve world directory names at the ZIP root.
- Include all world data except `session.lock`.
- Symlinks may resolve only within the canonical server root.
- Create `backups/` for its owner only and archives with mode `0600`.
- A stopped or daemon-managed running server may be backed up; an unmanaged running server may not.
## Failure cases
- `server.properties` or world paths are invalid.
- A backup with the same timestamp exists or another backup is active.
- Available space is smaller than the estimated input size.
- The live save fails or exceeds five minutes.
- An active `session.lock` belongs to a server not managed by the daemon.
- Archiving is interrupted or fails.

Failure or client disconnect removes the temporary ZIP. A live backup always restores saving before releasing queued Console inputs. A completed archive contains only world files and never `minegr.toml` or software artifacts.
## Implementation
The CLI owns path validation, size estimation, and ZIP creation. The daemon controls the live snapshot barrier and recovers it if the client disconnects. The advisory lock file may remain after the operating-system lock is released.
## Related
- [Daemon](../daemon.md)
- [Operation coordination](../../architecture/operation-coordination.md)
- [Filesystem layout](../../architecture/filesystem-layout.md)
